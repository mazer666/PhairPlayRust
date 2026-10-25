package org.phairplay.receiver.bridge

import org.phairplay.receiver.model.DEFAULT_MAX_MESSAGE_BYTES
import uniffi.phairplay_ffi.FfiAppConfig
import uniffi.phairplay_ffi.FfiDeviceConfig
import uniffi.phairplay_ffi.FfiLimitsConfig
import uniffi.phairplay_ffi.FfiPerformanceConfig
import uniffi.phairplay_ffi.FfiProtocolConfig

/**
 * The default configuration of the receiver.
 *
 * WHAT: the reader of the file `receiver-defaults.properties` and the
 * builder of the configuration of the bridge out of it.
 *
 * WHY: The bridge takes the configuration of the receiver as an argument
 * and offers no default of its own, so the shell has to build one. The
 * values live in a data file and not in the source: the file is the mirror
 * of `config/app-defaults.toml` and of `config/performance-budgets.toml`,
 * and a change of a value in one of those files has to be made in the
 * mirror in the same change.
 *
 * The reader is strict. A key that is missing is an error and not a
 * reason to fall back to a value of the shell: a receiver that runs with
 * a limit that the shell invented is a receiver whose limits nobody
 * reviewed.
 *
 * EXAMPLE: `ReceiverDefaults.toConfiguration(values, "Pixel 8")` builds the
 * configuration that the receiver starts with.
 */
object ReceiverDefaults {

    /**
     * The name of the file with the default values.
     */
    const val FILE_NAME: String = "receiver-defaults.properties"

    /**
     * Reads the key and value pairs of a properties file.
     *
     * WHY: The format is the one of a properties file of Java: a key, an
     * equals sign, the value, and a number sign that starts a comment. The
     * format is small enough to be read without a library, and the reader
     * is therefore testable without a device.
     *
     * @param text the content of the file
     * @return the values of the file
     * @throws IllegalArgumentException when a line is not a key and a value
     */
    fun parse(text: String): Map<String, String> {
        val values = mutableMapOf<String, String>()
        for (number in text.lines().withIndex()) {
            val line = number.value.trim()
            if (line.isEmpty() || line.startsWith("#")) {
                continue
            }
            val separator = line.indexOf('=')
            if (separator <= 0) {
                throw IllegalArgumentException(
                    "line ${number.index + 1} of the defaults is not a key and a value: $line",
                )
            }
            val key = line.substring(0, separator).trim()
            val value = line.substring(separator + 1).trim()
            values[key] = value
        }
        return values
    }

    /**
     * Builds the configuration of the bridge out of the values.
     *
     * @param values the values of the defaults file
     * @param deviceName the name that the platform reports for the device,
     *   or `null` to keep the name of the defaults file
     * @return the configuration of the receiver
     * @throws IllegalArgumentException when a key is missing or a value is
     *   not a number
     */
    fun toConfiguration(values: Map<String, String>, deviceName: String? = null): FfiAppConfig {
        val name = deviceName?.takeIf { it.isNotBlank() } ?: string(values, "device.name")
        return FfiAppConfig(
            device = FfiDeviceConfig(
                name = name,
                model = string(values, "device.model"),
                manufacturer = string(values, "device.manufacturer"),
                protocolVersion = string(values, "device.protocol_version"),
            ),
            protocols = listOf(
                protocol(values, "airplay"),
                protocol(values, "miracast"),
                protocol(values, "cast"),
                protocol(values, "dlna"),
                protocol(values, "smartview"),
            ),
            limits = FfiLimitsConfig(
                videoQueueFrames = ulong(values, "limits.video_queue_frames"),
                videoQueueBytes = ulong(values, "limits.video_queue_bytes"),
                audioQueuePackets = ulong(values, "limits.audio_queue_packets"),
                audioQueueBytes = ulong(values, "limits.audio_queue_bytes"),
                maxPairingAttempts = uint(values, "limits.max_pairing_attempts"),
                maxArtworkBytes = ulong(values, "limits.max_artwork_bytes"),
                maxPhotoBytes = ulong(values, "limits.max_photo_bytes"),
                maxMediaBytes = ulong(values, "limits.max_media_bytes"),
                maxHeaderCount = ulong(values, "limits.max_header_count"),
                maxHeaderBytes = ulong(values, "limits.max_header_bytes"),
            ),
            performance = FfiPerformanceConfig(
                firstFrameLatencyMs = ulong(values, "performance.first_frame_latency_ms"),
                minimumFps = uint(values, "performance.minimum_fps"),
                maxMemoryMb = uint(values, "performance.max_memory_mb"),
                maxCpuPercent = uint(values, "performance.max_cpu_percent"),
                idleTimeoutMs = ulong(values, "performance.idle_timeout_ms"),
            ),
        )
    }

    /**
     * The configuration of one protocol.
     */
    private fun protocol(values: Map<String, String>, name: String): FfiProtocolConfig =
        FfiProtocolConfig(
            name = name,
            enabled = boolean(values, "protocol.$name.enabled"),
            port = ushort(values, "protocol.$name.port"),
            requirePin = boolean(values, "protocol.$name.require_pin"),
            maxMessageBytes = DEFAULT_MAX_MESSAGE_BYTES,
        )

    /**
     * The value of a key.
     */
    private fun string(values: Map<String, String>, key: String): String =
        values[key] ?: throw IllegalArgumentException("the defaults do not contain the key `$key`")

    /**
     * The value of a key that is a number of 64 bits.
     */
    private fun ulong(values: Map<String, String>, key: String): ULong =
        string(values, key).toULongOrNull()
            ?: throw IllegalArgumentException("the value of `$key` is not a number: ${values[key]}")

    /**
     * The value of a key that is a number of 32 bits.
     */
    private fun uint(values: Map<String, String>, key: String): UInt =
        string(values, key).toUIntOrNull()
            ?: throw IllegalArgumentException("the value of `$key` is not a number: ${values[key]}")

    /**
     * The value of a key that is a number of 16 bits.
     */
    private fun ushort(values: Map<String, String>, key: String): UShort =
        string(values, key).toUShortOrNull()
            ?: throw IllegalArgumentException("the value of `$key` is not a number: ${values[key]}")

    /**
     * The value of a key that is a yes or a no.
     */
    private fun boolean(values: Map<String, String>, key: String): Boolean =
        when (string(values, key).lowercase()) {
            "true" -> true
            "false" -> false
            else -> throw IllegalArgumentException(
                "the value of `$key` is not a yes or a no: ${values[key]}",
            )
        }

}
