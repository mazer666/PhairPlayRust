import XCTest
@testable import PhairPlayMacKit

/// The tests of the handling of the keys.
///
/// WHAT: the proof that the space bar switches the protocol of the card
/// that holds the focus, that the escape key closes the banner of a
/// failure, that the tab key and the arrow keys move the focus, and that
/// the keys of the keyboard are mapped to the keys of the shell.
///
/// WHY: The window is operated without a mouse, and the keys are the only
/// way to reach the protocols. The mapping of a key of the keyboard to a
/// key of the shell is written down as numbers, and a number that is
/// wrong is a key that does nothing — which cannot be seen in a test of
/// the window, because the window does not exist on the machine that runs
/// the tests.
///
/// EXAMPLE: the space bar on the card of AirPlay asks for the action
/// ``ShellAction/toggleProtocol(_:)`` with the case `.airplay`.
final class ShellKeyboardTests: XCTestCase {

    func testTheSpaceKeySwitchesTheFocusedProtocol() {
        let outcome = ShellKeyboard.handle(
            .space,
            focus: .card(.airplay),
            state: state()
        )
        XCTAssertEqual(outcome.action, .toggleProtocol(.airplay))
        XCTAssertEqual(outcome.focus, .card(.airplay))
    }

    func testTheSpaceKeyOnTheBannerClosesIt() {
        let outcome = ShellKeyboard.handle(
            .space,
            focus: .dismissFailure,
            state: state(failure: true)
        )
        XCTAssertEqual(outcome.action, .dismissFailure)
    }

    func testTheSpaceKeyDoesNothingWithoutAFocus() {
        let outcome = ShellKeyboard.handle(.space, focus: nil, state: state())
        XCTAssertNil(outcome.action)
    }

    func testTheEscapeKeyClosesTheBanner() {
        let outcome = ShellKeyboard.handle(
            .escape,
            focus: .card(.cast),
            state: state(failure: true)
        )
        XCTAssertEqual(outcome.action, .dismissFailure)
        XCTAssertEqual(outcome.focus, .card(.cast))
    }

    func testTheEscapeKeyDoesNothingWithoutAFailure() {
        let outcome = ShellKeyboard.handle(
            .escape,
            focus: .card(.cast),
            state: state(failure: false)
        )
        XCTAssertNil(outcome.action)
    }

    func testTheTabKeyMovesTheFocusToTheNextCard() {
        let outcome = ShellKeyboard.handle(
            .tab,
            focus: .card(.airplay),
            state: state()
        )
        XCTAssertEqual(outcome.focus, .card(.miracast))
        XCTAssertNil(outcome.action)
    }

    func testTheShiftTabKeyMovesTheFocusToThePreviousCard() {
        let outcome = ShellKeyboard.handle(
            .shiftTab,
            focus: .card(.miracast),
            state: state()
        )
        XCTAssertEqual(outcome.focus, .card(.airplay))
    }

    func testTheArrowKeysMoveTheFocusInsideTheGrid() {
        let outcome = ShellKeyboard.handle(
            .down,
            focus: .card(.airplay),
            state: state()
        )
        XCTAssertEqual(outcome.focus, .card(.cast))
    }

    func testTheKeysOfTheKeyboardAreMapped() {
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 48, shift: false), .tab)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 48, shift: true), .shiftTab)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 49, shift: false), .space)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 53, shift: false), .escape)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 123, shift: false), .left)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 124, shift: false), .right)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 126, shift: false), .up)
        XCTAssertEqual(ShellKeyMap.key(forKeyCode: 125, shift: false), .down)
    }

    func testAKeyThatTheShellDoesNotUseIsAnsweredWithNothing() {
        XCTAssertNil(ShellKeyMap.key(forKeyCode: 0, shift: false))
        XCTAssertNil(ShellKeyMap.key(forKeyCode: 36, shift: false))
    }

    /// A state of the receiver with the four cards.
    private func state(failure: Bool = false) -> ReceiverUIState {
        ReceiverUIState(
            running: true,
            deviceName: "Attic",
            version: "0.1.0",
            protocols: ProtocolId.allCases.map { identifier in
                ProtocolCardState(
                    protocolId: identifier,
                    phase: .advertising,
                    enabled: true,
                    tcpPort: 7000,
                    udpPort: nil,
                    peerCount: 0,
                    lastError: nil
                )
            },
            nowPlaying: nil,
            failure: failure
                ? BridgeFailure(code: 1, details: "unknown protocol")
                : nil
        )
    }
}
