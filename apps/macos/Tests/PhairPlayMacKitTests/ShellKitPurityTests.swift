import XCTest
@testable import PhairPlayMacKit

/// The tests of the purity of the kit.
///
/// WHAT: the proof that the kit imports nothing but Foundation and the
/// bindings of the bridge, and that no file of the shell grows beyond the
/// size that a reviewer can hold in their head.
///
/// WHY: The kit holds the logic of the shell — the order of the focus, the
/// handling of the keys, the wording of the states and the sizes of the
/// text. A kit that imports SwiftUI could only be built on a Mac, and the
/// logic of the shell would only be tested on the machine of the
/// developer who happens to own one. The kit therefore stays free of the
/// frameworks of the user interface, and the test watches that it does.
///
/// EXAMPLE: a file of the kit that imports SwiftUI fails the test of this
/// file, and the logic of the shell is tested again on every machine.
final class ShellKitPurityTests: XCTestCase {

    /// The frameworks that the kit is allowed to import.
    ///
    /// WHY: The bindings of the bridge are a module of this package and
    /// not a framework of the system. Everything else — SwiftUI, AppKit,
    /// Combine — would tie the kit to a Mac.
    private static let allowedImports: Set<String> = [
        "Foundation",
        "PhairPlayFFI",
    ]

    /// The frameworks that the kit must never import.
    private static let forbiddenImports: Set<String> = [
        "SwiftUI",
        "AppKit",
        "Combine",
        "UIKit",
        "WebKit",
    ]

    /// The biggest file of the shell, in lines.
    ///
    /// WHY: A file that nobody can read is a file that nobody reviews. The
    /// limit is the soft limit of the product; the test keeps it.
    private static let maximumLines = 400

    func testTheKitImportsNoFrameworkOfTheUserInterface() {
        for file in ShellKitPurityTests.sources(of: "PhairPlayMacKit") {
            for module in ShellKitPurityTests.imports(of: file) {
                XCTAssertFalse(
                    ShellKitPurityTests.forbiddenImports.contains(module),
                    "the file \(file.lastPathComponent) imports \(module)"
                )
            }
        }
    }

    func testTheKitImportsNothingButFoundationAndTheBindings() {
        for file in ShellKitPurityTests.sources(of: "PhairPlayMacKit") {
            for module in ShellKitPurityTests.imports(of: file) {
                XCTAssertTrue(
                    ShellKitPurityTests.allowedImports.contains(module),
                    "the file \(file.lastPathComponent) imports \(module)"
                )
            }
        }
    }

    func testNoFileOfTheShellIsTooLong() {
        for target in ["PhairPlayMacKit", "PhairPlayMacApp"] {
            for file in ShellKitPurityTests.sources(of: target) {
                let lines = ShellKitPurityTests.lines(of: file)
                XCTAssertLessThanOrEqual(
                    lines,
                    ShellKitPurityTests.maximumLines,
                    "the file \(file.lastPathComponent) has \(lines) lines"
                )
            }
        }
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

    /// The modules that a file imports.
    ///
    /// - Parameter file: the file.
    /// - Returns: the names of the imported modules.
    private static func imports(of file: URL) -> [String] {
        let contents = (try? String(contentsOf: file, encoding: .utf8)) ?? ""
        return contents
            .split(separator: "\n")
            .compactMap { line -> String? in
                let trimmed = line.trimmingCharacters(in: .whitespaces)
                guard trimmed.hasPrefix("import ") else {
                    return nil
                }
                return String(trimmed.dropFirst("import ".count))
                    .trimmingCharacters(in: .whitespaces)
            }
    }

    /// The number of lines of a file.
    private static func lines(of file: URL) -> Int {
        let contents = (try? String(contentsOf: file, encoding: .utf8)) ?? ""
        return contents.split(separator: "\n", omittingEmptySubsequences: false).count
    }

    /// The root of the package of the shell.
    private static func packageRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
    }
}
