import Foundation

/// The state of the receiver as a whole.
///
/// WHAT: the four states the head of the window shows: the receiver is
/// running, it is starting, it is stopped, or it reported an error.
///
/// WHY: The head of the window is the one thing that is visible from
/// across the room, and it has to answer one question: "can I send
/// something to this device right now?". The state of a single protocol
/// cannot answer it — three protocols can advertise themselves while the
/// fourth one has failed. The four states are therefore a summary of the
/// four cards, and every one of them is shown as a word, as a shape and as
/// a colour, so that the state never depends on the colour alone.
///
/// EXAMPLE: a receiver with one advertising protocol and one failed
/// protocol is ``ReceiverStatus/running``: the receiver serves the
/// network, and the failure is shown in the card of the protocol that
/// failed.
public enum ReceiverStatus: String, Sendable, CaseIterable {

    /// At least one service advertises itself or carries a stream.
    case running

    /// A service is on its way: it starts or it stops.
    case starting

    /// No service runs.
    case stopped

    /// No service runs and at least one of them failed.
    case error

    /// The state of the receiver for a given state of the shell.
    ///
    /// WHY: The order of the cases is the order of the questions. A
    /// receiver that serves the network is running, even if another
    /// protocol failed — the failure is then shown in the card of the
    /// protocol that failed, where it belongs. A receiver whose services
    /// are on their way is starting. A receiver whose only failure is a
    /// protocol that never ran is in error, because it cannot serve
    /// anything. Everything else is stopped.
    ///
    /// - Parameter state: the state of the shell.
    /// - Returns: the state of the receiver.
    public init(state: ReceiverUIState) {
        let phases = state.protocols.map(\.phase)
        if phases.contains(where: { $0 == .advertising || $0 == .streaming }) {
            self = .running
        } else if phases.contains(where: { $0 == .starting || $0 == .stopping }) {
            self = .starting
        } else if phases.contains(.failed) {
            self = .error
        } else {
            self = .stopped
        }
    }

    /// The key of the text of the state.
    ///
    /// EXAMPLE: ``ReceiverStatus/running`` has the key `status.running`.
    public var textKey: String {
        "status.\(rawValue)"
    }

    /// The key of the sentence that explains the state.
    ///
    /// EXAMPLE: ``ReceiverStatus/running`` has the key
    /// `status.running.detail`.
    public var detailKey: String {
        "status.\(rawValue).detail"
    }

    /// The name of the symbol that shows the state.
    ///
    /// WHY: The shape is the second channel of the state next to the word.
    /// The four symbols have four different outlines — a circle with a
    /// tick, a circle with arrows, a circle with two bars and a triangle —
    /// so that the state can be read without the colour and without
    /// reading the word. The name is a text and not a framework, so the
    /// kit does not import SwiftUI and can be tested on every system.
    public var symbolName: String {
        switch self {
        case .running:
            return "checkmark.circle.fill"
        case .starting:
            return "arrow.triangle.2.circlepath"
        case .stopped:
            return "pause.circle.fill"
        case .error:
            return "exclamationmark.triangle.fill"
        }
    }

    /// The tone of the state.
    public var tone: ShellTone {
        switch self {
        case .running:
            return .advertising
        case .starting:
            return .transition
        case .stopped:
            return .idle
        case .error:
            return .failure
        }
    }

    /// Whether the state is a problem the user has to see.
    ///
    /// WHY: The head of the window draws the tone of the state. The tone
    /// of a failure is the only one that shouts; every other state is
    /// calm.
    public var isProblem: Bool {
        self == .error
    }
}
