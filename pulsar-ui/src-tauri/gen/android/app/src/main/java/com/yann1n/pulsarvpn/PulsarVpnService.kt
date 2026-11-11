package com.yann1n.pulsarvpn

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.core.app.NotificationCompat
import org.json.JSONObject
import java.io.File

class PulsarVpnService : VpnService() {
    private var tunFd: Int? = null
    private val handler = Handler(Looper.getMainLooper())
    private val preferences by lazy {
        getSharedPreferences(MainActivity.PREFS, Context.MODE_PRIVATE)
    }
    private val healthCheck = object : Runnable {
        override fun run() {
            val config = preferences.getString(MainActivity.KEY_CONFIG, null)
            if (config != null && !nativeIsXrayRunning()) {
                Log.w(TAG, "Xray stopped unexpectedly; restarting")
                runCatching { startTunnel(config) }
                    .onFailure { Log.e(TAG, "Xray restart failed", it) }
            }
            handler.postDelayed(this, HEALTH_CHECK_MS)
        }
    }

    private external fun nativeStartXray(
        binary: String,
        assetDirectory: String,
        config: String,
        tunFd: Int
    ): String?

    private external fun nativeStopXray()
    private external fun nativeIsXrayRunning(): Boolean

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopTunnel(forget = true)
            return START_NOT_STICKY
        }

        val config = intent?.getStringExtra(EXTRA_CONFIG)
            ?: preferences.getString(MainActivity.KEY_CONFIG, null)
        if (config.isNullOrBlank()) {
            stopSelf()
            return START_NOT_STICKY
        }

        startForeground(NOTIFICATION_ID, notification("Connecting"))
        runCatching { startTunnel(config) }
            .onSuccess { updateNotification("Connected") }
            .onFailure {
                Log.e(TAG, "Unable to start Xray", it)
                preferences.edit().putBoolean(MainActivity.KEY_RUNNING, false).apply()
                updateNotification("Connection failed")
                stopRuntime()
                stopSelf()
            }
        return START_STICKY
    }

    private fun startTunnel(config: String) {
        handler.removeCallbacks(healthCheck)
        stopRuntime()
        val hasIpv6 = JSONObject(config)
            .optJSONObject("dns")
            ?.optString("queryStrategy") == "UseIP"
        val builder = Builder()
            .setSession("Pulsar VPN")
            .setMtu(1280)
            .addAddress("172.19.0.1", 30)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("1.1.1.1")
            // Xray is a child of our UID; excluding it prevents its outbound
            // sockets from being captured by its own VPN interface.
            .addDisallowedApplication(packageName)
        if (hasIpv6) {
            builder.addAddress("fdfe:dcba:9876::1", 126)
            builder.addRoute("::", 0)
            builder.addDnsServer("2606:4700:4700::1111")
        }

        val descriptor = builder.establish()
            ?: error("Android refused to create the VPN interface")
        val fd = descriptor.detachFd()
        tunFd = fd

        val assetDirectory = prepareAssets()
        val binary = File(applicationInfo.nativeLibraryDir, "libxray.so")
        check(binary.canExecute()) {
            "Xray runtime is missing for ${Build.SUPPORTED_ABIS.joinToString()}"
        }
        nativeStartXray(binary.absolutePath, assetDirectory.absolutePath, config, fd)
            ?.let { error(it) }

        preferences.edit()
            .putString(MainActivity.KEY_CONFIG, config)
            .putBoolean(MainActivity.KEY_RUNNING, true)
            .apply()
        handler.postDelayed(healthCheck, HEALTH_CHECK_MS)
    }

    private fun prepareAssets(): File {
        val directory = File(filesDir, "xray").apply { mkdirs() }
        listOf("geoip.dat", "geosite.dat").forEach { name ->
            val destination = File(directory, name)
            assets.open("xray/$name").use { input ->
                if (!destination.exists() || destination.length() != input.available().toLong()) {
                    destination.outputStream().use(input::copyTo)
                }
            }
        }
        return directory
    }

    private fun stopTunnel(forget: Boolean) {
        handler.removeCallbacks(healthCheck)
        stopRuntime()
        val editor = preferences.edit().putBoolean(MainActivity.KEY_RUNNING, false)
        if (forget) editor.remove(MainActivity.KEY_CONFIG)
        editor.apply()
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun stopRuntime() {
        nativeStopXray()
        tunFd?.let { fd -> runCatching { ParcelFileDescriptor.adoptFd(fd).close() } }
        tunFd = null
    }

    override fun onRevoke() = stopTunnel(forget = true)

    override fun onDestroy() {
        handler.removeCallbacks(healthCheck)
        stopRuntime()
        preferences.edit().putBoolean(MainActivity.KEY_RUNNING, false).apply()
        super.onDestroy()
    }

    private fun notification(state: String) = NotificationCompat.Builder(this, CHANNEL_ID)
        .setSmallIcon(R.mipmap.ic_launcher)
        .setContentTitle("Pulsar VPN")
        .setContentText(state)
        .setOngoing(true)
        .setCategory(NotificationCompat.CATEGORY_SERVICE)
        .setOnlyAlertOnce(true)
        .setContentIntent(
            PendingIntent.getActivity(
                this,
                0,
                packageManager.getLaunchIntentForPackage(packageName),
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        )
        .build()

    private fun updateNotification(state: String) {
        getSystemService(NotificationManager::class.java)
            .notify(NOTIFICATION_ID, notification(state))
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "VPN connection",
            NotificationManager.IMPORTANCE_LOW
        )
        getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
    }

    companion object {
        const val ACTION_START = "com.yann1n.pulsarvpn.START"
        const val ACTION_STOP = "com.yann1n.pulsarvpn.STOP"
        const val EXTRA_CONFIG = "config"
        private const val TAG = "PulsarVpnService"
        private const val CHANNEL_ID = "pulsar_vpn"
        private const val NOTIFICATION_ID = 1107
        private const val HEALTH_CHECK_MS = 5_000L
    }
}
