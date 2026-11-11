package com.yann1n.pulsarvpn

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat

class PulsarVpnBootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        val config = context.getSharedPreferences(MainActivity.PREFS, Context.MODE_PRIVATE)
            .getString(MainActivity.KEY_CONFIG, null)
            ?: return
        ContextCompat.startForegroundService(
            context,
            Intent(context, PulsarVpnService::class.java)
                .setAction(PulsarVpnService.ACTION_START)
                .putExtra(PulsarVpnService.EXTRA_CONFIG, config)
        )
    }
}
