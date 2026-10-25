package org.phairplay.receiver.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.phairplay_ffi.FfiAppConfig
import uniffi.phairplay_ffi.FfiAudioCodec
import uniffi.phairplay_ffi.FfiAudioFormat
import uniffi.phairplay_ffi.FfiDeviceConfig
import uniffi.phairplay_ffi.FfiLimitsConfig
import uniffi.phairplay_ffi.FfiPerformanceConfig
import uniffi.phairplay_ffi.FfiProtocolConfig
import uniffi.phairplay_ffi.FfiVideoCodec
import uniffi.phairplay_ffi.FfiVideoFormat

/**
 * The tests of the translation of the objects of the bridge.
 *
 * WHAT: the proof that the configuration of the bridge keeps the setting of
 * a protocol, that a protocol that is not mentioned is switched on — like
 * the core does — and that the descriptions of the streams name the codec.
 *
 * WHY: The settings of the protocols are the only thing the user changes
 * from the interface, and a mistake there is a protocol that is
 * advertised although the user switched it off.
 */
class ReceiverMappingTest {

    private fun configuration(protocols: List<FfiProtocolConfig>): FfiAppConfig = FfiAppConfig(
        device = FfiDeviceConfig(
            name = "Attic",
            model = "PhairPlay Rust Receiver",
            manufacturer = "PhairPlayRust",
            protocolVersion = "1.0",
        ),
        protocols = protocols,
        limits = FfiLimitsConfig(
            videoQueueFrames = 90uL,
            videoQueueBytes = 67108864uL,
            audioQueuePackets = 200uL,
            audioQueueBytes = 4194304uL,
            maxPairingAttempts = 5u,
            maxArtworkBytes = 8388608uL,
            maxPhotoBytes = 20971520uL,
            maxMediaBytes = 536870912uL,
            maxHeaderCount = 64uL,
            maxHeaderBytes = 8192uL,
        ),
        performance = FfiPerformanceConfig(
            firstFrameLatencyMs = 100uL,
            minimumFps = 25u,
            maxMemoryMb = 150u,
            maxCpuPercent = 30u,
            idleTimeoutMs = 30000uL,
        ),
    )

    @Test
    fun aProtocolThatIsNotMentionedIsSwitchedOn() {
        // The core treats a protocol that is not in the configuration as
        // switched on, because the default of the protocol is "enabled".
        val config = configuration(emptyList())
        assertTrue(config.isProtocolEnabled("airplay"))
    }

    @Test
    fun aProtocolThatIsSwitchedOffIsSwitchedOff() {
        val config = configuration(
            listOf(FfiProtocolConfig("airplay", enabled = false, port = 0u, requirePin = false, maxMessageBytes = 1024uL)),
        )
        assertFalse(config.isProtocolEnabled("airplay"))
    }

    @Test
    fun switchingAProtocolOffWritesTheSetting() {
        val config = configuration(
            listOf(FfiProtocolConfig("airplay", enabled = true, port = 7000u, requirePin = true, maxMessageBytes = 4096uL)),
        )
        val updated = config.withProtocolEnabled("airplay", false)
        assertFalse(updated.isProtocolEnabled("airplay"))
    }

    @Test
    fun switchingAProtocolOffKeepsTheRestOfTheEntry() {
        val config = configuration(
            listOf(FfiProtocolConfig("airplay", enabled = true, port = 7000u, requirePin = true, maxMessageBytes = 4096uL)),
        )
        val updated = config.withProtocolEnabled("airplay", false)
        val entry = updated.protocols.first { it.name == "airplay" }
        assertEquals(7000.toUShort(), entry.port)
        assertTrue(entry.requirePin)
        assertEquals(4096uL, entry.maxMessageBytes)
    }

    @Test
    fun switchingAProtocolOnAddsItWithTheDefaultOfTheCore() {
        val config = configuration(emptyList())
        val updated = config.withProtocolEnabled("cast", true)
        val entry = updated.protocols.first { it.name == "cast" }
        assertTrue(entry.enabled)
        assertEquals(0.toUShort(), entry.port)
        assertFalse(entry.requirePin)
        assertEquals(DEFAULT_MAX_MESSAGE_BYTES, entry.maxMessageBytes)
    }

    @Test
    fun switchingAProtocolKeepsTheOtherProtocols() {
        val config = configuration(
            listOf(
                FfiProtocolConfig("airplay", enabled = true, port = 0u, requirePin = false, maxMessageBytes = 1024uL),
                FfiProtocolConfig("cast", enabled = true, port = 0u, requirePin = false, maxMessageBytes = 1024uL),
            ),
        )
        val updated = config.withProtocolEnabled("cast", false)
        assertEquals(2, updated.protocols.size)
        assertTrue(updated.isProtocolEnabled("airplay"))
        assertFalse(updated.isProtocolEnabled("cast"))
    }

    @Test
    fun theDescriptionOfTheVideoNamesTheCodecAndTheSize() {
        val format = FfiVideoFormat(
            codec = FfiVideoCodec.H264,
            width = 1920u,
            height = 1080u,
            frameRate = 30u,
            sps = null,
            pps = null,
        )
        assertEquals("H.264 1920x1080 @30", videoDescription(format))
    }

    @Test
    fun theDescriptionOfTheAudioNamesTheCodecAndTheRate() {
        val format = FfiAudioFormat(
            codec = FfiAudioCodec.AAC_LC,
            sampleRate = 44100u,
            channels = 2u,
            samplesPerPacket = 1024u,
        )
        assertEquals("AAC-LC 44100 Hz stereo", audioDescription(format))
    }

    @Test
    fun aSessionWithoutAVideoHasNoDescription() {
        assertEquals(null, videoDescription(null))
        assertEquals(null, audioDescription(null))
    }
}
