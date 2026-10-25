import XCTest
import PhairPlayFFI

/// The tests of the generated bindings against the library of the bridge.
///
/// WHAT: the proof that the bindings that are checked into the repository
/// belong to the library that is built from the sources of the same
/// revision.
///
/// WHY: the bindings are generated, and a binding that is edited by hand
/// drifts from the library it is loaded against. UniFFI protects itself
/// against exactly that: every exported function carries a checksum of its
/// signature, and the binding refuses to start when the checksum of the
/// library does not match the one of the binding. A drift that is not
/// caught here shows up on the device of a user as a crash at the start of
/// the shell — and the crash names a checksum, not a cause.
///
/// WHY the constructor: the checksum of the constructor of the receiver
/// covers its documentation, because the documentation is part of the
/// metadata of the function. A change of a comment in the Rust source
/// therefore changes the checksum — and a binding that was edited by hand
/// keeps the old number.
final class BindingChecksumTests: XCTestCase {

    /// The configuration that the receiver is built with.
    private func configuration() -> FfiAppConfig {
        FfiAppConfig(
            device: FfiDeviceConfig(
                name: "Attic",
                model: "PhairPlay Rust Receiver",
                manufacturer: "PhairplayRust",
                protocolVersion: "1.0"
            ),
            protocols: [],
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
            ),
            session: FfiSessionConfig(
                multiPeerProtocols: ["cast"]
            )
        )
    }

    /// The binding and the library agree.
    ///
    /// WHY: the test fails with the message of UniFFI when they do not —
    /// "UniFFI API checksum mismatch" — and the name of the checksum says
    /// which function drifted.
    func testTheBindingsMatchTheLibrary() throws {
        let receiver = try Receiver(config: configuration())
        XCTAssertNotNil(receiver)
    }
}
