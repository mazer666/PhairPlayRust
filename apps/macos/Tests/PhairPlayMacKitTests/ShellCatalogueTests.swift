import XCTest
@testable import PhairPlayMacKit

/// The tests of the catalogue of the texts.
///
/// WHAT: the proof that the English catalogue and the German catalogue
/// carry the same keys, that every key that the kit and the views build
/// exists in both of them, and that no view writes a text of its own.
///
/// WHY: A text that is written into the source of a view cannot be
/// translated and cannot be reviewed by somebody who does not read Swift.
/// A key that exists in one language and not in the other shows the user
/// the key itself on half of the screens. Both mistakes are invisible in
/// the build and obvious in the window, so they are caught here.
///
/// EXAMPLE: the key `shortcut.arrows.title` exists in the English
/// catalogue and in the German catalogue, and the view of the shortcut
/// reads it out of the catalogue instead of writing "Arrow keys" into its
/// own source.
final class ShellCatalogueTests: XCTestCase {

    /// The keys that the views of the shell read out of the catalogue.
    ///
    /// WHY: The views read their texts with `ReceiverStrings.text`, and
    /// the keys of those calls are found by the scan of the sources below.
    /// The keys that the kit builds out of a name are listed here, because
    /// they are not written into the source of a view.
    private static let builtKeys: [String] = [
        "dashboard.title",
        "dashboard.device",
        "dashboard.version",
        "dashboard.running",
        "dashboard.stopped",
        "dashboard.protocols",
        "protocol.ports",
        "protocol.ports.tcp",
        "protocol.ports.none",
        "protocol.ports.unknown",
        "protocol.peers.one",
        "protocol.peers.many",
        "protocol.switch.on",
        "protocol.switch.off",
        "protocol.switch.state.on",
        "protocol.switch.state.off",
        "nowplaying.title",
        "nowplaying.empty",
        "nowplaying.sender.unknown",
        "nowplaying.title.unknown",
        "nowplaying.artist.unknown",
        "nowplaying.album.unknown",
        "nowplaying.video.unknown",
        "nowplaying.audio.unknown",
        "nowplaying.video",
        "nowplaying.audio",
        "nowplaying.uptime",
        "failure.title",
        "failure.code",
        "failure.dismiss",
        "menu.title",
        "menu.running",
        "menu.stopped",
        "menu.refresh",
        "menu.quit",
    ]

    func testTheEnglishAndTheGermanCatalogueCarryTheSameKeys() {
        let english = Set(ShellCatalogueTests.catalogue(language: "en").keys)
        let german = Set(ShellCatalogueTests.catalogue(language: "de").keys)
        XCTAssertFalse(english.isEmpty)
        XCTAssertEqual(
            english.subtracting(german),
            [],
            "these keys are missing in the German catalogue"
        )
        XCTAssertEqual(
            german.subtracting(english),
            [],
            "these keys are missing in the English catalogue"
        )
    }

    func testEveryEntryOfTheCatalogueCarriesAText() {
        for language in ["en", "de"] {
            for (key, text) in ShellCatalogueTests.catalogue(language: language) {
                XCTAssertFalse(
                    text.trimmingCharacters(in: .whitespaces).isEmpty,
                    "the key \(key) of the \(language) catalogue carries no text"
                )
            }
        }
    }

    func testThePlaceholdersOfTheCatalogueAreAllPercentAt() {
        // WHY: The reader of the catalogue replaces the placeholders from
        // left to right. A placeholder of another kind would be left in the
        // text of the window.
        for language in ["en", "de"] {
            for (key, text) in ShellCatalogueTests.catalogue(language: language) {
                for placeholder in ["%d", "%i", "%f", "%@", "%1$@"] where text.contains(placeholder) {
                    XCTAssertEqual(
                        placeholder,
                        "%@",
                        "the key \(key) of the \(language) catalogue uses \(placeholder)"
                    )
                }
            }
        }
    }

    func testEveryKeyThatTheKitBuildsExistsInBothCatalogues() {
        var keys = ShellCatalogueTests.builtKeys
        for identifier in ProtocolId.allCases {
            keys.append("protocol.\(identifier.rawValue)")
        }
        for phase in ProtocolPhase.allCases {
            keys.append(phase.textKey)
            keys.append(phase.reasonKey)
        }
        for status in ReceiverStatus.allCases {
            keys.append(status.textKey)
            keys.append(status.detailKey)
        }
        for shortcut in ShellShortcut.all {
            keys.append(shortcut.titleKey)
            keys.append(shortcut.detailKey)
        }
        for language in ["en", "de"] {
            let catalogue = ShellCatalogueTests.catalogue(language: language)
            for key in keys {
                XCTAssertNotNil(
                    catalogue[key],
                    "the key \(key) is missing in the \(language) catalogue"
                )
            }
        }
    }

    func testEveryKeyThatTheSourcesUseExistsInBothCatalogues() {
        let used = ShellCatalogueTests.usedKeys()
        XCTAssertFalse(used.isEmpty)
        for language in ["en", "de"] {
            let catalogue = ShellCatalogueTests.catalogue(language: language)
            for key in used.sorted() {
                XCTAssertNotNil(
                    catalogue[key],
                    "the key \(key) is missing in the \(language) catalogue"
                )
            }
        }
    }

    func testNoViewWritesATextOfItsOwn() {
        // WHY: A text in the source of a view cannot be translated. The
        // scan looks for the opening of a text with a literal in it.
        for file in ShellCatalogueTests.sources(of: "PhairPlayMacApp") {
            let contents = ShellCatalogueTests.code(of: file)
            XCTAssertFalse(
                contents.contains("Text(\""),
                "the view \(file.lastPathComponent) writes a text of its own"
            )
        }
    }

    /// The catalogue of one language.
    ///
    /// - Parameter language: the name of the language, "en" or "de".
    /// - Returns: the texts of the catalogue, by key.
    private static func catalogue(language: String) -> [String: String] {
        let url = resources().appendingPathComponent("\(language).lproj/Localizable.strings")
        guard let contents = try? String(contentsOf: url, encoding: .utf8) else {
            XCTFail("the catalogue \(language) cannot be read at \(url.path)")
            return [:]
        }
        var texts: [String: String] = [:]
        let pattern = "\"([^\"]+)\"\\s*=\\s*\"((?:[^\"\\\\]|\\\\.)*)\";"
        guard let expression = try? NSRegularExpression(pattern: pattern) else {
            XCTFail("the pattern of the catalogue cannot be built")
            return [:]
        }
        let range = NSRange(contents.startIndex..<contents.endIndex, in: contents)
        for match in expression.matches(in: contents, range: range) {
            guard let keyRange = Range(match.range(at: 1), in: contents),
                  let textRange = Range(match.range(at: 2), in: contents) else {
                continue
            }
            texts[String(contents[keyRange])] = String(contents[textRange])
        }
        return texts
    }

    /// The keys that the sources of the shell read out of the catalogue.
    ///
    /// - Returns: the keys of every call of `ReceiverStrings.text` in the
    ///   kit and in the application.
    private static func usedKeys() -> Set<String> {
        var keys: Set<String> = []
        for directory in ["PhairPlayMacKit", "PhairPlayMacApp"] {
            for file in sources(of: directory) {
                let contents = code(of: file)
                guard let expression = try? NSRegularExpression(
                    pattern: "ReceiverStrings\\.text\\(\\s*\"([^\"]+)\""
                ) else {
                    continue
                }
                let range = NSRange(contents.startIndex..<contents.endIndex, in: contents)
                for match in expression.matches(in: contents, range: range) {
                    guard let keyRange = Range(match.range(at: 1), in: contents) else {
                        continue
                    }
                    let key = String(contents[keyRange])
                    // WHY: A key that is built out of a name carries a
                    // backslash in its source — "protocol.\(name)" — and it
                    // is checked by the test of the built keys above.
                    guard !key.contains("\\") else {
                        continue
                    }
                    keys.insert(key)
                }
            }
        }
        return keys
    }

    /// The source files of one target of the package.
    ///
    /// - Parameter target: the name of the target.
    /// - Returns: the files of the target.
    private static func sources(of target: String) -> [URL] {
        let directory = packageRoot().appendingPathComponent("Sources/\(target)")
        let contents = (try? FileManager.default.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: nil
        )) ?? []
        return contents.filter { $0.pathExtension == "swift" }.sorted { $0.path < $1.path }
    }

    /// The directory of the resources of the kit.
    private static func resources() -> URL {
        packageRoot().appendingPathComponent("Sources/PhairPlayMacKit/Resources")
    }

    /// The root of the package of the shell.
    ///
    /// WHY: The tests read the sources and the catalogues of the package,
    /// because those files are the truth that a translator changes. The
    /// path is built out of the path of this file, so that it is right
    /// wherever the package is checked out.
    private static func packageRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
    }

    /// The content of a file.
    private static func read(_ url: URL) -> String {
        (try? String(contentsOf: url, encoding: .utf8)) ?? ""
    }

    /// The code of a file, without its comments.
    ///
    /// WHY: The comments of the shell explain the code with examples that
    /// name keys of the catalogue. A scan of the code must not read those
    /// examples as a use of a key.
    ///
    /// - Parameter url: the file.
    /// - Returns: the lines of the file that carry code.
    private static func code(of url: URL) -> String {
        read(url)
            .split(separator: "\n", omittingEmptySubsequences: false)
            .filter { line in
                !line.trimmingCharacters(in: .whitespaces).hasPrefix("//")
            }
            .joined(separator: "\n")
    }
}
