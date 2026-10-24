import XCTest
@testable import PhairPlayMacKit

/// The tests of the shortcuts of the shell.
///
/// WHAT: the proof that every key of the shell belongs to exactly one
/// group of shortcuts, and that every group carries a name and a sentence
/// out of the catalogue.
///
/// WHY: The hint at the foot of the window is built out of the groups of
/// the kit. A key that the shell answers to but that no group names would
/// be a key that the user cannot find, and a group without a text would be
/// a hole in the hint. Both mistakes are cheap to make and cheap to catch.
///
/// EXAMPLE: the group of the arrow keys covers the four arrow keys of the
/// shell and is named "Arrow keys" in the catalogue.
final class ShellShortcutTests: XCTestCase {

    func testEveryKeyOfTheShellBelongsToExactlyOneGroup() {
        for key in ShellKey.allCases {
            let groups = ShellShortcut.all.filter { $0.keys.contains(key) }
            XCTAssertEqual(
                groups.count,
                1,
                "the key \(key) belongs to \(groups.count) groups of shortcuts"
            )
        }
    }

    func testEveryGroupCarriesAtLeastOneKey() {
        for shortcut in ShellShortcut.all {
            XCTAssertFalse(shortcut.keys.isEmpty)
        }
    }

    func testEveryKeyOfTheShellIsCoveredByAGroup() {
        let covered = Set(ShellShortcut.all.flatMap(\.keys))
        for key in ShellKey.allCases {
            XCTAssertTrue(covered.contains(key))
        }
    }

    func testTheGroupOfAKeyIsFound() {
        XCTAssertEqual(ShellShortcut.group(of: .tab), .tab)
        XCTAssertEqual(ShellShortcut.group(of: .shiftTab), .shiftTab)
        XCTAssertEqual(ShellShortcut.group(of: .left), .arrows)
        XCTAssertEqual(ShellShortcut.group(of: .right), .arrows)
        XCTAssertEqual(ShellShortcut.group(of: .up), .arrows)
        XCTAssertEqual(ShellShortcut.group(of: .down), .arrows)
        XCTAssertEqual(ShellShortcut.group(of: .space), .space)
        XCTAssertEqual(ShellShortcut.group(of: .escape), .escape)
    }

    func testEveryGroupHasANameAndASentence() {
        // WHY: A missing text is visible — the reader answers the key
        // itself — but it is a mistake. A test that looks for the key
        // catches a group whose text was never written into the catalogue.
        for shortcut in ShellShortcut.all {
            XCTAssertNotEqual(
                ShellFormatting.shortcutTitle(shortcut),
                shortcut.titleKey
            )
            XCTAssertNotEqual(
                ShellFormatting.shortcutDetail(shortcut),
                shortcut.detailKey
            )
        }
    }

    func testEveryGroupHasItsOwnName() {
        let titles = Set(ShellShortcut.all.map { ShellFormatting.shortcutTitle($0) })
        XCTAssertEqual(titles.count, ShellShortcut.all.count)
    }

    func testTheHintNamesEveryGroupInTheOrderOfTheKit() {
        let texts = ShellFormatting.shortcutTexts()
        XCTAssertEqual(texts.count, ShellShortcut.all.count)
        XCTAssertEqual(texts.map(\.shortcut), ShellShortcut.all)
        for shortcut in texts {
            XCTAssertFalse(shortcut.title.isEmpty)
            XCTAssertFalse(shortcut.detail.isEmpty)
        }
    }

    func testTheTextOfAShortcutCarriesItsGroup() {
        let text = ShellFormatting.shortcutText(.arrows)
        XCTAssertEqual(text.shortcut, .arrows)
        XCTAssertEqual(text.title, ShellFormatting.shortcutTitle(.arrows))
        XCTAssertEqual(text.detail, ShellFormatting.shortcutDetail(.arrows))
    }
}
