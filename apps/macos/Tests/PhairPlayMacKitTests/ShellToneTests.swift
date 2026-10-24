import XCTest
@testable import PhairPlayMacKit

/// The tests of the tones of the shell.
///
/// WHAT: the proof that every tone is a colour that a screen can show,
/// that no two tones carry the same colour, and that every phase and every
/// state of the receiver has a tone.
///
/// WHY: The tones are the five slots of the product, and they are written
/// down as numbers so that the window of this shell and the tests of the
/// kit cannot drift apart. A number that is not a colour is a colour that
/// the shell cannot draw, and two states with the same colour are two
/// states that the user cannot tell apart.
///
/// EXAMPLE: the tone ``ShellTone/advertising`` is the colour `1F8A5B`.
final class ShellToneTests: XCTestCase {

    func testEveryToneIsAColourOfSixDigits() {
        for tone in ShellTone.allCases {
            let hex = tone.hex
            XCTAssertEqual(hex.count, 6, "the tone \(tone.rawValue) is not a colour")
            XCTAssertTrue(
                hex.allSatisfy { $0.isHexDigit },
                "the tone \(tone.rawValue) carries a digit that is not hexadecimal"
            )
        }
    }

    func testNoTwoTonesCarryTheSameColour() {
        let colours = Set(ShellTone.allCases.map(\.hex))
        XCTAssertEqual(colours.count, ShellTone.allCases.count)
    }

    func testEveryPhaseHasATone() {
        for phase in ProtocolPhase.allCases {
            XCTAssertTrue(ShellTone.allCases.contains(phase.tone))
        }
    }

    func testTheTonesOfThePhasesAreTheOnesOfTheProduct() {
        XCTAssertEqual(ProtocolPhase.disabled.tone, .idle)
        XCTAssertEqual(ProtocolPhase.notStarted.tone, .idle)
        XCTAssertEqual(ProtocolPhase.stopped.tone, .idle)
        XCTAssertEqual(ProtocolPhase.starting.tone, .transition)
        XCTAssertEqual(ProtocolPhase.stopping.tone, .transition)
        XCTAssertEqual(ProtocolPhase.advertising.tone, .advertising)
        XCTAssertEqual(ProtocolPhase.streaming.tone, .streaming)
        XCTAssertEqual(ProtocolPhase.failed.tone, .failure)
    }

    func testEveryStateOfTheReceiverHasATone() {
        for status in ReceiverStatus.allCases {
            XCTAssertTrue(ShellTone.allCases.contains(status.tone))
        }
    }

    func testTheFailureIsTheOnlyToneThatIsRed() {
        XCTAssertEqual(ReceiverStatus.error.tone, .failure)
        for status in ReceiverStatus.allCases where status != .error {
            XCTAssertNotEqual(status.tone, .failure)
        }
    }

    func testTheRunningStateCarriesTheToneOfAServiceThatAdvertisesItself() {
        XCTAssertEqual(ReceiverStatus.running.tone, .advertising)
    }
}
