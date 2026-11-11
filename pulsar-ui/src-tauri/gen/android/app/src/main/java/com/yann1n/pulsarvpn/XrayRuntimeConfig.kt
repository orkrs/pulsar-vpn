package com.yann1n.pulsarvpn

internal object XrayRuntimeConfig {
    fun command(binary: String, config: String): List<String> =
        listOf(binary, "run", "-config", config)

    fun environment(tunFd: Int, assetDirectory: String): Map<String, String> {
        require(tunFd >= 0) { "TUN file descriptor must be valid" }
        return mapOf(
            "XRAY_TUN_FD" to tunFd.toString(),
            "XRAY_LOCATION_ASSET" to assetDirectory
        )
    }
}
