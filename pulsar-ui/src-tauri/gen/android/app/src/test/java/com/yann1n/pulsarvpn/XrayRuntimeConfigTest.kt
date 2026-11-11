package com.yann1n.pulsarvpn

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class XrayRuntimeConfigTest {
    @Test
    fun commandUsesExplicitConfigFile() {
        assertEquals(
            listOf("/native/libxray.so", "run", "-config", "/files/xray/config.json"),
            XrayRuntimeConfig.command("/native/libxray.so", "/files/xray/config.json")
        )
    }

    @Test
    fun environmentPassesTunDescriptorAndAssetDirectory() {
        val env = XrayRuntimeConfig.environment(42, "/files/xray")
        assertEquals("42", env["XRAY_TUN_FD"])
        assertEquals("/files/xray", env["XRAY_LOCATION_ASSET"])
        assertFalse(env.containsKey("PATH"))
    }

    @Test(expected = IllegalArgumentException::class)
    fun rejectsInvalidTunDescriptor() {
        XrayRuntimeConfig.environment(-1, "/files/xray")
    }
}
