package com.yann1n.pulsarvpn

import android.app.Activity
import android.Manifest
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.VpnService
import android.os.Build
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
    private var pendingConfig: String? = null

    private val vpnPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            if (result.resultCode == Activity.RESULT_OK) {
                pendingConfig?.let { config ->
                    val intent = Intent(this, PulsarVpnService::class.java)
                        .setAction(PulsarVpnService.ACTION_START)
                        .putExtra(PulsarVpnService.EXTRA_CONFIG, config)
                    ContextCompat.startForegroundService(this, intent)
                }
            }
            pendingConfig = null
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        instance = this
        if (Build.VERSION.SDK_INT >= 33 && checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1107)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (instance === this) instance = null
    }

    companion object {
        const val STARTED = 0
        const val PERMISSION_REQUESTED = 1
        private var instance: MainActivity? = null

        @JvmStatic
        fun requestVpn(context: Context, config: String): Int {
            val ctx = instance ?: context
            if (VpnService.prepare(ctx) == null) {
                val intent = Intent(ctx, PulsarVpnService::class.java)
                    .setAction(PulsarVpnService.ACTION_START)
                    .putExtra(PulsarVpnService.EXTRA_CONFIG, config)
                ContextCompat.startForegroundService(ctx, intent)
                return STARTED
            }
            instance?.let { act ->
                act.pendingConfig = config
                val intent = VpnService.prepare(act) ?: return@let
                act.vpnPermissionLauncher.launch(intent)
            }
            return PERMISSION_REQUESTED
        }

        @JvmStatic
        fun stopVpn(context: Context) {
            val ctx = instance ?: context
            ctx.startService(
                Intent(ctx, PulsarVpnService::class.java)
                    .setAction(PulsarVpnService.ACTION_STOP)
            )
        }

        @JvmStatic
        fun isVpnRunning(context: Context): Boolean =
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
                .getBoolean(KEY_RUNNING, false)

        internal const val PREFS = "pulsar_vpn_service"
        internal const val KEY_CONFIG = "config"
        internal const val KEY_RUNNING = "running"

        @JvmStatic
        fun getClipboardText(context: Context): String {
            val ctx = instance ?: context
            val clipboard = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                ?: return ""
            val clip = clipboard.primaryClip
            if (clip != null && clip.itemCount > 0) {
                val text = clip.getItemAt(0).text
                return text?.toString() ?: ""
            }
            return ""
        }
    }
}
