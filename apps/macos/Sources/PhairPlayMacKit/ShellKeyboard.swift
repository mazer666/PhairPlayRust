import Foundation

/// A key of the window.
///
/// WHAT: the keys the shell reacts to: the tab key with and without the
/// shift key, the four arrow keys, the space bar and the escape key.
///
/// WHY: The window is a panel that is read from a distance and operated
/// without a mouse. The keys of the shell are therefore named here, and
/// the mapping of a key of the keyboard to one of these names is a pure
/// function that can be tested without a window.
public enum ShellKey: Sendable, Equatable, CaseIterable {

    /// The tab key: the focus goes to the next surface.
    case tab

    /// The tab key together with the shift key: the focus goes to the
    /// previous surface.
    case shiftTab

    /// The arrow key to the left.
    case left

    /// The arrow key to the right.
    case right

    /// The arrow key up.
    case up

    /// The arrow key down.
    case down

    /// The space bar: the protocol of the card that holds the focus is
    /// switched on or off.
    case space

    /// The escape key: the banner of a failure is closed.
    case escape
}

/// The mapping of the keys of the keyboard to the keys of the shell.
///
/// WHY: The numbers of the keys of the virtual keyboard of macOS do not
/// change, and they are the only thing that AppKit hands over that is the
/// same on every keyboard layout. The mapping lives here and not in the
/// application, so that it can be tested without AppKit.
///
/// EXAMPLE: the key code `48` is the tab key, and the key code `48` with
/// the shift key is ``ShellKey/shiftTab``.
public enum ShellKeyMap {

    /// The numbers of the keys of the virtual keyboard of macOS.
    public enum KeyCode {
        /// The tab key.
        public static let tab: Int = 48

        /// The space bar.
        public static let space: Int = 49

        /// The escape key.
        public static let escape: Int = 53

        /// The arrow key to the left.
        public static let left: Int = 123

        /// The arrow key to the right.
        public static let right: Int = 124

        /// The arrow key down.
        public static let down: Int = 125

        /// The arrow key up.
        public static let up: Int = 126
    }

    /// The key of the shell for a key of the keyboard.
    ///
    /// - Parameters:
    ///   - code: the number of the key of the virtual keyboard.
    ///   - shift: whether the shift key is pressed.
    /// - Returns: the key of the shell, or `nil` when the key of the
    ///   keyboard is not one of the keys of the shell.
    public static func key(forKeyCode code: Int, shift: Bool) -> ShellKey? {
        switch code {
        case KeyCode.tab:
            return shift ? .shiftTab : .tab
        case KeyCode.space:
            return .space
        case KeyCode.escape:
            return .escape
        case KeyCode.left:
            return .left
        case KeyCode.right:
            return .right
        case KeyCode.up:
            return .up
        case KeyCode.down:
            return .down
        default:
            return nil
        }
    }
}

/// What the shell does with a key.
///
/// WHY: The window decides what happens; the kit only says what has to
/// happen. A key never changes the state of the receiver directly — it
/// asks the client to do it — so that the receiver is only ever changed
/// through the bridge.
public enum ShellAction: Sendable, Equatable {

    /// The protocol has to be switched on or off.
    case toggleProtocol(ProtocolId)

    /// The banner of the failure has to be closed.
    case dismissFailure
}

/// The answer of the shell to a key.
///
/// WHAT: the surface that holds the focus after the key and the action
/// that the key asks for.
///
/// EXAMPLE: the space bar on the card of AirPlay answers with the focus
/// on the card of AirPlay and the action ``ShellAction/toggleProtocol(_:)``
/// with the case `.airplay`.
public struct ShellKeyOutcome: Sendable, Equatable {

    /// The surface that holds the focus after the key.
    public let focus: FocusTarget?

    /// The action that the key asks for, if it asks for one.
    public let action: ShellAction?

    /// Creates the answer of the shell to a key.
    ///
    /// - Parameters:
    ///   - focus: the surface that holds the focus after the key.
    ///   - action: the action that the key asks for.
    public init(focus: FocusTarget?, action: ShellAction? = nil) {
        self.focus = focus
        self.action = action
    }
}

/// The handling of the keys of the window.
///
/// WHAT: the pure function that turns a key and the state of the receiver
/// into the new focus and the action that has to happen.
///
/// WHY: The order of the focus and the meaning of the keys are the
/// behaviour of the shell, and they are checked here instead of being
/// checked by clicking through a window. The function knows nothing about
/// SwiftUI: it answers with data, and the window draws it.
///
/// EXAMPLE: the escape key answers with the action
/// ``ShellAction/dismissFailure`` when the receiver reported a failure,
/// and with no action at all when it did not.
public enum ShellKeyboard {

    /// The answer of the shell to a key.
    ///
    /// - Parameters:
    ///   - key: the key that was pressed.
    ///   - focus: the surface that holds the focus before the key.
    ///   - state: the state of the receiver.
    /// - Returns: the new focus and the action of the key.
    public static func handle(
        _ key: ShellKey,
        focus: FocusTarget?,
        state: ReceiverUIState
    ) -> ShellKeyOutcome {
        let ring = FocusRing(state: state)
        switch key {
        case .tab:
            return ShellKeyOutcome(focus: ring.next(after: focus))
        case .shiftTab:
            return ShellKeyOutcome(focus: ring.previous(before: focus))
        case .left:
            return ShellKeyOutcome(focus: ring.move(.left, from: focus))
        case .right:
            return ShellKeyOutcome(focus: ring.move(.right, from: focus))
        case .up:
            return ShellKeyOutcome(focus: ring.move(.up, from: focus))
        case .down:
            return ShellKeyOutcome(focus: ring.move(.down, from: focus))
        case .space:
            return ShellKeyOutcome(focus: focus, action: toggleAction(for: focus))
        case .escape:
            let action: ShellAction? = state.failure == nil ? nil : .dismissFailure
            return ShellKeyOutcome(focus: focus, action: action)
        }
    }

    /// The action of the space bar for a surface that holds the focus.
    ///
    /// WHY: The space bar switches the protocol of the card that holds the
    /// focus. On the button of the banner it closes the banner, because
    /// that button is the only thing that the space bar can do there.
    ///
    /// - Parameter focus: the surface that holds the focus.
    /// - Returns: the action, or `nil` when nothing holds the focus.
    private static func toggleAction(for focus: FocusTarget?) -> ShellAction? {
        switch focus {
        case .some(.card(let identifier)):
            return .toggleProtocol(identifier)
        case .some(.dismissFailure):
            return .dismissFailure
        case nil:
            return nil
        }
    }
}
