import XCTest
import PhairPlayFFI
@testable import PhairPlayMacKit

/// The tests of the model of the shell.
///
/// WHAT: the proof that the shell tells "switched off in the settings" and
/// "the service is not running" apart, that every state of the engine
/// finds its phase, and that the configuration keeps the setting of a
/// protocol.
///
/// WHY: The two cases look the same on a screen and mean something
/// completely different. A protocol that is switched off is not advertised
/// at all; a protocol that is switched on but not running was stopped, and
/// the user can start it. A test that does not check the difference cannot
/// catch a mistake in it.
final class ReceiverModelTests: XCTestCase {

    func testAProtocolThatIsSwitchedOffIsDisabled() {
        let phase = ReceiverMapping.protocolPhase(enabled: false, state: .advertising)
        XCTAssertEqual(phase, .disabled)
    }

    func testAProtocolThatIsSwitchedOffIsDisabledEvenWhileItStops() {
        let phase = ReceiverMapping.protocolPhase(enabled: false, state: .stopping)
        XCTAssertEqual(phase, .disabled)
    }

    func testAStoppedProtocolIsStopped() {
        let phase = ReceiverMapping.protocolPhase(enabled: true, state: .stopped)
        XCTAssertEqual(phase, .stopped)
    }

    func testStoppedAndDisabledAreNotTheSame() {
        let stopped = ReceiverMapping.protocolPhase(enabled: true, state: .stopped)
        let disabled = ReceiverMapping.protocolPhase(enabled: false, state: .stopped)
        XCTAssertNotEqual(stopped, disabled)
    }

    func testAProtocolThatNeverRanIsNotStarted() {
        let phase = ReceiverMapping.protocolPhase(enabled: true, state: .idle)
        XCTAssertEqual(phase, .notStarted)
    }

    func testEveryStateOfTheEngineFindsItsPhase() {
        let expected: [FfiProtocolState: ProtocolPhase] = [
            .idle: .notStarted,
            .starting: .starting,
            .advertising: .advertising,
            .streaming: .streaming,
            .stopping: .stopping,
            .stopped: .stopped,
            .failed: .failed,
        ]
        for (state, phase) in expected {
            XCTAssertEqual(ReceiverMapping.protocolPhase(enabled: true, state: state), phase)
        }
    }

    func testOnlyTheStatesThatServeTheNetworkCountAsRunning() {
        XCTAssertFalse(ProtocolPhase.disabled.isRunning)
        XCTAssertFalse(ProtocolPhase.notStarted.isRunning)
        XCTAssertFalse(ProtocolPhase.stopped.isRunning)
        XCTAssertFalse(ProtocolPhase.failed.isRunning)
        XCTAssertTrue(ProtocolPhase.starting.isRunning)
        XCTAssertTrue(ProtocolPhase.advertising.isRunning)
        XCTAssertTrue(ProtocolPhase.streaming.isRunning)
        XCTAssertTrue(ProtocolPhase.stopping.isRunning)
    }

    func testAProtocolThatIsNotMentionedIsSwitchedOn() {
        // The core treats a protocol that is not in the configuration as
        // switched on, because the default of the protocol is "enabled".
        let config = configuration(protocols: [])
        XCTAssertTrue(ReceiverMapping.isProtocolEnabled(config, bridgeName: "airplay"))
    }

    func testAProtocolThatIsSwitchedOffIsSwitchedOff() {
        let config = configuration(protocols: [
            FfiProtocolConfig(
                name: "airplay",
                enabled: false,
                port: 0,
                requirePin: false,
                maxMessageBytes: 1024
            )
        ])
        XCTAssertFalse(ReceiverMapping.isProtocolEnabled(config, bridgeName: "airplay"))
    }

    func testSwitchingAProtocolOffWritesTheSetting() {
        let config = configuration(protocols: [
            FfiProtocolConfig(
                name: "airplay",
                enabled: true,
                port: 7000,
                requirePin: true,
                maxMessageBytes: 4096
            )
        ])
        let updated = ReceiverMapping.withProtocolEnabled(
            config,
            bridgeName: "airplay",
            enabled: false
        )
        XCTAssertFalse(ReceiverMapping.isProtocolEnabled(updated, bridgeName: "airplay"))
    }

    func testSwitchingAProtocolOffKeepsTheRestOfTheEntry() {
        let config = configuration(protocols: [
            FfiProtocolConfig(
                name: "airplay",
                enabled: true,
                port: 7000,
                requirePin: true,
                maxMessageBytes: 4096
            )
        ])
        let updated = ReceiverMapping.withProtocolEnabled(
            config,
            bridgeName: "airplay",
            enabled: false
        )
        let entry = updated.protocols.first { $0.name == "airplay" }
        XCTAssertEqual(entry?.port, 7000)
        XCTAssertEqual(entry?.requirePin, true)
        XCTAssertEqual(entry?.maxMessageBytes, 4096)
    }

    func testSwitchingAProtocolOnAddsItWithTheDefaultOfTheCore() {
        let config = configuration(protocols: [])
        let updated = ReceiverMapping.withProtocolEnabled(
            config,
            bridgeName: "cast",
            enabled: true
        )
        let entry = updated.protocols.first { $0.name == "cast" }
        XCTAssertEqual(entry?.enabled, true)
        XCTAssertEqual(entry?.port, 0)
        XCTAssertEqual(entry?.requirePin, false)
        XCTAssertEqual(entry?.maxMessageBytes, ReceiverDefaults.defaultMaxMessageBytes)
    }

    func testSwitchingAProtocolKeepsTheOtherProtocols() {
        let config = configuration(protocols: [
            FfiProtocolConfig(
                name: "airplay",
                enabled: true,
                port: 0,
                requirePin: false,
                maxMessageBytes: 1024
            ),
            FfiProtocolConfig(
                name: "cast",
                enabled: true,
                port: 0,
                requirePin: false,
                maxMessageBytes: 1024
            )
        ])
        let updated = ReceiverMapping.withProtocolEnabled(
            config,
            bridgeName: "cast",
            enabled: false
        )
        XCTAssertEqual(updated.protocols.count, 2)
        XCTAssertTrue(ReceiverMapping.isProtocolEnabled(updated, bridgeName: "airplay"))
        XCTAssertFalse(ReceiverMapping.isProtocolEnabled(updated, bridgeName: "cast"))
    }

    func testTheDescriptionOfTheVideoNamesTheCodecAndTheSize() {
        let format = FfiVideoFormat(
            codec: .h264,
            width: 1920,
            height: 1080,
            frameRate: 30,
            sps: nil,
            pps: nil
        )
        XCTAssertEqual(ReceiverMapping.videoDescription(format), "H.264 1920x1080 @30")
    }

    func testTheDescriptionOfTheAudioNamesTheCodecAndTheRate() {
        let format = FfiAudioFormat(
            codec: .aacLc,
            sampleRate: 44100,
            channels: 2,
            samplesPerPacket: 1024
        )
        XCTAssertEqual(ReceiverMapping.audioDescription(format), "AAC-LC 44100 Hz stereo")
    }

    func testASessionWithoutAVideoHasNoDescription() {
        XCTAssertNil(ReceiverMapping.videoDescription(nil))
        XCTAssertNil(ReceiverMapping.audioDescription(nil))
    }

    func testTheRunningTimeIsWrittenWithTwoDigits() {
        XCTAssertEqual(ReceiverMapping.uptimeText(0), "0:00")
        XCTAssertEqual(ReceiverMapping.uptimeText(5_000), "0:05")
        XCTAssertEqual(ReceiverMapping.uptimeText(65_000), "1:05")
        XCTAssertEqual(ReceiverMapping.uptimeText(3_725_000), "1:02:05")
    }

    /// A configuration with the given protocols.
    private func configuration(protocols: [FfiProtocolConfig]) -> FfiAppConfig {
        FfiAppConfig(
            device: FfiDeviceConfig(
                name: "Attic",
                model: "PhairPlay Rust Receiver",
                manufacturer: "PhairPlayRust",
                protocolVersion: "1.0"
            ),
            protocols: protocols,
            limits: FfiLimitsConfig(
                videoQueueFrames: 90,
                videoQueueBytes: 67_108_864,
                audioQueuePackets: 200,
                audioQueueBytes: 4_194_304,
                maxPairingAttempts: 5,
                maxArtworkBytes: 8_388_608,
                maxPhotoBytes: 20_971_520,
                maxMediaBytes: 536_870_912,
                maxHeaderCount: 64,
                maxHeaderBytes: 8192
            ),
            performance: FfiPerformanceConfig(
                firstFrameLatencyMs: 100,
                minimumFps: 25,
                maxMemoryMb: 150,
                maxCpuPercent: 30,
                idleTimeoutMs: 30_000
            )
        )
    }
}
