import XCTest
import PhairPlayFFI
@testable import PhairPlayMacKit

/// The tests of the reader of the default configuration.
///
/// WHAT: the proof that the reader of the file with the defaults accepts a
/// file that is written like the one of the repository, and that it refuses
/// a file with a missing key instead of inventing a value.
///
/// WHY: The limits of the receiver are the defence against a peer that
/// tries to exhaust the memory of the device. A limit that the shell fills
/// in on its own is a limit that nobody reviewed — and a limit that is
/// missing is a receiver that accepts a message of any size.
final class ReceiverDefaultsTests: XCTestCase {

    func testTheReaderSkipsTheCommentsAndTheEmptyLines() throws {
        let values = try ReceiverDefaults.parse(
            """
            # A comment.
            device.name = PhairPlay
            device.model = PhairPlay Rust Receiver

            limits.video_queue_frames = 90
            protocol.airplay.enabled = true
            """
        )
        XCTAssertEqual(values.count, 4)
    }

    func testTheReaderKeepsTheValueOfAKey() throws {
        let values = try ReceiverDefaults.parse(
            """
            device.name = PhairPlay
            limits.video_queue_frames = 90
            """
        )
        XCTAssertEqual(values["device.name"], "PhairPlay")
        XCTAssertEqual(values["limits.video_queue_frames"], "90")
    }

    func testTheReaderRefusesALineThatIsNotAKeyAndAValue() {
        XCTAssertThrowsError(try ReceiverDefaults.parse("device.name PhairPlay"))
    }

    func testTheConfigurationIsBuiltOutOfTheValues() throws {
        let config = try ReceiverDefaults.configuration(from: try values())
        XCTAssertEqual(config.device.name, "PhairPlay")
        XCTAssertEqual(config.device.manufacturer, "PhairPlayRust")
        XCTAssertEqual(config.protocols.count, 4)
        XCTAssertEqual(config.protocols[0].name, "airplay")
        XCTAssertEqual(config.protocols[3].name, "dlna")
        XCTAssertEqual(config.limits.videoQueueFrames, 90)
        XCTAssertEqual(config.limits.videoQueueBytes, 67_108_864)
        XCTAssertEqual(config.limits.maxPairingAttempts, 5)
        XCTAssertEqual(config.performance.idleTimeoutMs, 30_000)
        XCTAssertEqual(config.performance.minimumFps, 25)
    }

    func testTheNameOfTheDeviceWinsOverTheNameOfTheFile() throws {
        let config = try ReceiverDefaults.configuration(from: try values(), deviceName: "Mac mini")
        XCTAssertEqual(config.device.name, "Mac mini")
    }

    func testAMissingKeyIsAnErrorAndNotAValueOfTheShell() {
        XCTAssertThrowsError(try ReceiverDefaults.configuration(from: ["device.name": "PhairPlay"]))
    }

    func testAValueThatIsNotANumberIsAnError() throws {
        let broken = try values().merging(["limits.video_queue_frames": "many"]) { _, new in new }
        XCTAssertThrowsError(try ReceiverDefaults.configuration(from: broken))
    }

    /// The values of the defaults file of the shell.
    private func values() throws -> [String: String] {
        try ReceiverDefaults.parse(
            """
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
            """
        )
    }
}
