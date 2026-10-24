import Foundation

/// A group of keys of the shell that do the same thing.
///
/// WHAT: the five groups of keys that the window answers to — the tab key,
/// the tab key with the shift key, the four arrow keys, the space bar and
/// the escape key — and the texts of the catalogue that name them.
///
/// WHY: The shell is operated without a mouse, and the hint at the foot of
/// the window has to name exactly the keys that the shell answers to. The
/// groups live in the kit, next to the keys themselves, so that a key that
/// is added to the shell cannot be left out of the hint. The test of the
/// kit checks that every key belongs to exactly one group.
///
/// EXAMPLE: the group ``ShellShortcut/arrows`` covers the keys
/// ``ShellKey/left``, ``ShellKey/right``, ``ShellKey/up`` and
/// ``ShellKey/down``, and it is named "Arrow keys" in the catalogue.
public enum ShellShortcut: String, Sendable, CaseIterable {

    /// The tab key: the focus goes to the next surface.
    case tab

    /// The tab key together with the shift key: the focus goes to the
    /// previous surface.
    case shiftTab

    /// The four arrow keys: the focus moves inside the grid of the cards.
    case arrows

    /// The space bar: the protocol of the card that holds the focus is
    /// switched on or off.
    case space

    /// The escape key: the banner of a failure is closed.
    case escape

    /// The groups in the order in which the hint names them.
    ///
    /// WHY: The order is the order in which the user meets the keys: the
    /// key that walks through the window comes first, the key that moves
    /// inside the grid comes next, and the two keys that change something
    /// come last.
    public static let all: [ShellShortcut] = [
        .tab,
        .shiftTab,
        .arrows,
        .space,
        .escape,
    ]

    /// The keys of the shell that belong to the group.
    public var keys: [ShellKey] {
        switch self {
        case .tab:
            return [.tab]
        case .shiftTab:
            return [.shiftTab]
        case .arrows:
            return [.left, .right, .up, .down]
        case .space:
            return [.space]
        case .escape:
            return [.escape]
        }
    }

    /// The group that a key belongs to.
    ///
    /// WHY: The answer is `nil` for a key that the shell does not use. The
    /// caller then leaves the key alone instead of guessing a group for
    /// it.
    ///
    /// - Parameter key: the key of the shell.
    /// - Returns: the group of the key, or `nil` when the key belongs to no
    ///   group.
    public static func group(of key: ShellKey) -> ShellShortcut? {
        all.first { $0.keys.contains(key) }
    }

    /// The key of the name of the group in the catalogue.
    ///
    /// EXAMPLE: ``ShellShortcut/arrows`` has the key
    /// `shortcut.arrows.title`.
    public var titleKey: String {
        "shortcut.\(rawValue).title"
    }

    /// The key of the sentence that says what the group does.
    ///
    /// EXAMPLE: ``ShellShortcut/arrows`` has the key
    /// `shortcut.arrows.detail`.
    public var detailKey: String {
        "shortcut.\(rawValue).detail"
    }
}

/// A shortcut of the shell together with the texts of the catalogue.
///
/// WHAT: the group of the keys, the name of the group and the sentence
/// that says what the group does.
///
/// WHY: The view draws the name and the sentence, and it must not read the
/// catalogue itself — a view that reads the catalogue builds sentences
/// that cannot be translated. The kit resolves the texts and hands them
/// over, so that the hint of the window is built out of the same words
/// everywhere.
///
/// EXAMPLE: the shortcut of the arrow keys is named "Arrow keys" and says
/// "move in the grid".
public struct ShellShortcutText: Sendable, Equatable {

    /// The group of the keys.
    public let shortcut: ShellShortcut

    /// The name of the group.
    public let title: String

    /// The sentence that says what the group does.
    public let detail: String

    /// Creates the texts of a shortcut.
    ///
    /// - Parameters:
    ///   - shortcut: the group of the keys.
    ///   - title: the name of the group.
    ///   - detail: the sentence that says what the group does.
    public init(shortcut: ShellShortcut, title: String, detail: String) {
        self.shortcut = shortcut
        self.title = title
        self.detail = detail
    }
}
