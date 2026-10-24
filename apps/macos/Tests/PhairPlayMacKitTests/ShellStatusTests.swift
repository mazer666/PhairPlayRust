import XCTest
@testable import PhairPlayMacKit

/// The tests of the state of the receiver.
///
/// WHAT: the proof that the head of the window shows the right one of the
/// four states, and that every state carries a word, a shape and a tone.
///
/// WHY: The head of the window is the one thing that is read from a
/// distance. It has to answer one question — "can I send something to this
/// device right now?" — and it has to answer it with more than a colour,
/// because a colour is the one channel that a user with a weak screen or
/// with a colour blindness cannot use.
///
/// EXAMPLE: a receiver with one advertising protocol is
/// ``ReceiverStatus/running`` and reads "Running" with the symbol
/// `checkmark.circle.fill`.
final class ShellStatusTests: XCTestCase {

    func testAReceiverThatAdvertisesItselfIsRunning() {
        let status = ReceiverStatus(state: state(phases: [.advertising, .stopped]))
        XCTAssertEqual(status, .running)
    }

    func testAReceiverThatCarriesAStreamIsRunning() {
        let status = ReceiverStatus(state: state(phases: [.streaming]))
        XCTAssertEqual(status, .running)
    }

    func testAReceiverWhoseServiceIsStartingIsStarting() {
        let status = ReceiverStatus(state: state(phases: [.starting, .stopped]))
        XCTAssertEqual(status, .starting)
    }

    func testAReceiverWhoseServiceIsStoppingIsStarting() {
        let status = ReceiverStatus(state: state(phases: [.stopping]))
        XCTAssertEqual(status, .starting)
    }

    func testAReceiverWithoutAnyServiceIsStopped() {
        let status = ReceiverStatus(state: state(phases: [.disabled, .notStarted, .stopped]))
        XCTAssertEqual(status, .stopped)
    }

    func testAReceiverWhoseServiceFailedIsInError() {
        let status = ReceiverStatus(state: state(phases: [.failed, .stopped]))
        XCTAssertEqual(status, .error)
    }

    func testARunningServiceWinsOverAFailedProtocol() {
        // WHY: The receiver serves the network as long as one service
        // advertises itself. The failure belongs to the card of the
        // protocol that failed and not to the head of the window.
        let status = ReceiverStatus(state: state(phases: [.advertising, .failed]))
        XCTAssertEqual(status, .running)
    }

    func testEveryStateHasItsOwnWord() {
        let words = Set(ReceiverStatus.allCases.map { ShellFormatting.statusText($0) })
        XCTAssertEqual(words.count, ReceiverStatus.allCases.count)
    }

    func testEveryStateHasItsOwnShape() {
        let shapes = Set(ReceiverStatus.allCases.map(\.symbolName))
        XCTAssertEqual(shapes.count, ReceiverStatus.allCases.count)
    }

    func testEveryStateHasATone() {
        for status in ReceiverStatus.allCases {
            XCTAssertFalse(status.tone.hex.isEmpty)
        }
    }

    func testOnlyTheErrorIsAProblem() {
        XCTAssertTrue(ReceiverStatus.error.isProblem)
        XCTAssertFalse(ReceiverStatus.running.isProblem)
        XCTAssertFalse(ReceiverStatus.starting.isProblem)
        XCTAssertFalse(ReceiverStatus.stopped.isProblem)
    }

    func testTheWordOfAStateIsNoKey() {
        // WHY: A missing text is visible — the reader answers the key
        // itself — but it is a mistake. A test that looks for the key
        // catches a text that was never written into the catalogue.
        for status in ReceiverStatus.allCases {
            XCTAssertNotEqual(ShellFormatting.statusText(status), status.textKey)
            XCTAssertNotEqual(ShellFormatting.statusDetail(status), status.detailKey)
        }
    }

    /// A state of the receiver with the given phases.
    private func state(phases: [ProtocolPhase]) -> ReceiverUIState {
        ReceiverUIState(
            running: phases.contains { $0.isRunning },
            deviceName: "Attic",
            version: "0.1.0",
            protocols: zip(ProtocolId.allCases, phases).map { identifier, phase in
                ProtocolCardState(
                    protocolId: identifier,
                    phase: phase,
                    enabled: phase != .disabled,
                    tcpPort: nil,
                    udpPort: nil,
                    peerCount: 0,
                    lastError: nil
                )
            },
            nowPlaying: nil,
            failure: nil
        )
    }
}
