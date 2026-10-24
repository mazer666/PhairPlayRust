import XCTest
@testable import PhairPlayMacKit

/// The tests of the reader of the string catalogue.
///
/// WHAT: the proof that the reader answers the text of the catalogue, that
/// it answers the key itself when a text is missing, and that it replaces
/// the placeholders from left to right.
///
/// WHY: A missing text is visible but harmless — the reader answers the key
/// itself, and the application does not crash. A text that is read but
/// never translated is worse: the user sees the key in the middle of a
/// sentence. The reader is therefore tested on both of its answers, and
/// the tests of the catalogue look for the key as the sign of a missing
/// text.
///
/// EXAMPLE: `ReceiverStrings.text("protocol.airplay")` answers "AirPlay 2"
/// in English, and `ReceiverStrings.text("no.such.key")` answers
/// "no.such.key".
final class ReceiverStringsTests: XCTestCase {

    func testTheReaderAnswersTheTextOfTheCatalogue() {
        XCTAssertNotEqual(ReceiverStrings.text("protocol.airplay"), "protocol.airplay")
    }

    func testTheReaderAnswersTheKeyWhenTheTextIsMissing() {
        // WHY: A text that was never written must not crash the window —
        // it has to be visible instead, and the key is the shortest way to
        // make it visible.
        XCTAssertEqual(ReceiverStrings.text("no.such.key.at.all"), "no.such.key.at.all")
    }

    func testTheReaderReplacesThePlaceholdersFromLeftToRight() {
        let text = ReceiverStrings.text("failure.code", ["7"])
        XCTAssertTrue(text.contains("7"))
        XCTAssertFalse(text.contains("%@"))
    }

    func testTheReaderFillsEveryPlaceholderOfTheText() {
        let text = ReceiverStrings.text("protocol.ports", ["7000", "7001"])
        XCTAssertTrue(text.contains("7000"))
        XCTAssertTrue(text.contains("7001"))
        XCTAssertFalse(text.contains("%@"))
    }

    func testTheReaderLeavesTheTextAloneWhenThereIsNoPlaceholder() {
        XCTAssertEqual(
            ReceiverStrings.text("protocol.airplay", ["7000"]),
            ReceiverStrings.text("protocol.airplay")
        )
    }

    func testTheReaderUsesTheArgumentsOnlyAsOftenAsThereArePlaceholders() {
        // WHY: The reader replaces the placeholders from left to right. A
        // second argument without a second placeholder is dropped — it
        // must not appear in the text of the window.
        let text = ReceiverStrings.text("protocol.ports.tcp", ["7000", "7001"])
        XCTAssertTrue(text.contains("7000"))
        XCTAssertFalse(text.contains("7001"))
    }
}
