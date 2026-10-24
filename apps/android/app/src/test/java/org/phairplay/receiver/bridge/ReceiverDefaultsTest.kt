package org.phairplay.receiver.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The tests of the reader of the default configuration.
 *
 * WHAT: the proof that the reader of the file with the defaults accepts a
 * file that is written like the one of the repository, and that it refuses
 * a file with a missing key instead of inventing a value.
 *
 * WHY: The limits of the receiver are the defence against a peer that
 * tries to exhaust the memory of the device. A limit that the shell fills
 * in on its own is a limit that nobody reviewed — and a limit that is
 * missing is a receiver that accepts a message of any size.
 */
class ReceiverDefaultsTest {

    private val values = ReceiverDefaults.parse(
        """
        # A comment.
        device.name = PhairPlay
        device.model = PhairPlay Rust Receiver

        limits.video_queue_frames = 90
        protocol.airplay.enabled = true
        """.trimIndent(),
    )

    @Test
    fun theReaderSkipsTheCommentsAndTheEmptyLines() {
        assertEquals(4, values.size)
    }

    @Test
    fun theReaderKeepsTheValueOfAKey() {
        assertEquals("PhairPlay", values["device.name"])
        assertEquals("PhairPlay Rust Receiver", values["device.model"])
        assertEquals("90", values["limits.video_queue_frames"])
        assertEquals("true", values["protocol.airplay.enabled"])
    }

    @Test
    fun theReaderRefusesALineThatIsNotAKeyAndAValue() {
        val failure = runCatching {
            ReceiverDefaults.parse("device.name PhairPlay")
        }
        assertTrue(failure.isFailure)
    }

    @Test
    fun theConfigurationIsBuiltOutOfTheValues() {
        val text = """
            device.name = PhairPlay
            device.model = PhairPlay Rust Receiver
            device.manufacturer = PhairPlayRust
            device.protocol_version = 1.0
            protocol.airplay.enabled = true
            protocol.airplay.port = 0
            protocol.airplay.require_pin = false
            protocol.miracast.enabled = true
            protocol.miracast.port = 0
            protocol.miracast.require_pin = false
            protocol.cast.enabled = true
            protocol.cast.port = 0
            protocol.cast.require_pin = false
            protocol.dlna.enabled = true
            protocol.dlna.port = 0
            protocol.dlna.require_pin = false
            limits.video_queue_frames = 90
            limits.video_queue_bytes = 67108864
            limits.audio_queue_packets = 200
            limits.max_pairing_attempts = 5
            limits.max_artwork_bytes = 8388608
            limits.max_photo_bytes = 20971520
            limits.max_media_bytes = 536870912
            limits.max_header_count = 64
            limits.max_header_bytes = 8192
            performance.first_frame_latency_ms = 100
            performance.minimum_fps = 25
            performance.max_memory_mb = 150
            performance.max_cpu_percent = 30
            performance.idle_timeout_ms = 30000
        """.trimIndent()
        val config = ReceiverDefaults.toConfiguration(ReceiverDefaults.parse(text))

        assertEquals("PhairPlay", config.device.name)
        assertEquals("PhairPlayRust", config.device.manufacturer)
        assertEquals(4, config.protocols.size)
        assertEquals("airplay", config.protocols[0].name)
        assertEquals("dlna", config.protocols[3].name)
        assertEquals(90uL, config.limits.videoQueueFrames)
        assertEquals(67108864uL, config.limits.videoQueueBytes)
        assertEquals(5u, config.limits.maxPairingAttempts)
        assertEquals(30000uL, config.performance.idleTimeoutMs)
        assertEquals(25u, config.performance.minimumFps)
    }

    @Test
    fun theNameOfTheDeviceWinsOverTheNameOfTheFile() {
        val text = """
            device.name = PhairPlay
            device.model = PhairPlay Rust Receiver
            device.manufacturer = PhairPlayRust
            device.protocol_version = 1.0
            protocol.airplay.enabled = true
            protocol.airplay.port = 0
            protocol.airplay.require_pin = false
            protocol.miracast.enabled = true
            protocol.miracast.port = 0
            protocol.miracast.require_pin = false
            protocol.cast.enabled = true
            protocol.cast.port = 0
            protocol.cast.require_pin = false
            protocol.dlna.enabled = true
            protocol.dlna.port = 0
            protocol.dlna.require_pin = false
            limits.video_queue_frames = 90
            limits.video_queue_bytes = 67108864
            limits.audio_queue_packets = 200
            limits.max_pairing_attempts = 5
            limits.max_artwork_bytes = 8388608
            limits.max_photo_bytes = 20971520
            limits.max_media_bytes = 536870912
            limits.max_header_count = 64
            limits.max_header_bytes = 8192
            performance.first_frame_latency_ms = 100
            performance.minimum_fps = 25
            performance.max_memory_mb = 150
            performance.max_cpu_percent = 30
            performance.idle_timeout_ms = 30000
        """.trimIndent()
        val config = ReceiverDefaults.toConfiguration(
            values = ReceiverDefaults.parse(text),
            deviceName = "Pixel 8",
        )
        assertEquals("Pixel 8", config.device.name)
    }

    @Test
    fun aMissingKeyIsAnErrorAndNotAValueOfTheShell() {
        val failure = runCatching {
            ReceiverDefaults.toConfiguration(values)
        }
        assertTrue(failure.isFailure)
    }

    @Test
    fun aValueThatIsNotANumberIsAnError() {
        val broken = values + ("limits.video_queue_frames" to "many")
        val failure = runCatching {
            ReceiverDefaults.toConfiguration(broken)
        }
        assertTrue(failure.isFailure)
    }
}
