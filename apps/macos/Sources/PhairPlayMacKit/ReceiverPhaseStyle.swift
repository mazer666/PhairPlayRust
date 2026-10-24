import Foundation

/// The appearance of a state of a protocol.
///
/// WHAT: the word, the sentence, the shape and the tone of every phase a
/// protocol card can carry.
///
/// WHY: The phase of the engine is a fact of the core, and the way it is
/// shown is a fact of the shell. The two are kept apart here, so that the
/// card does not have to know where a state comes from — and so that every
/// platform of the product says the same thing in the same way.
///
/// The state is never shown as a colour alone. Every card carries the word
/// of the state, the sentence that explains it, and a symbol whose outline
/// is different for every state, so that the card can be read by a user
/// who cannot tell the colours apart.
///
/// EXAMPLE: the phase ``ProtocolPhase/disabled`` reads "Disabled in the
/// settings", explains that the service is not advertised at all, shows
/// the symbol `minus.circle.fill` and the tone ``ShellTone/idle``.
extension ProtocolPhase {

    /// The key of the word of the state.
    ///
    /// EXAMPLE: ``ProtocolPhase/streaming`` has the key
    /// `protocol.state.streaming`.
    public var textKey: String {
        "protocol.state.\(rawValue)"
    }

    /// The key of the sentence that explains the state.
    ///
    /// EXAMPLE: ``ProtocolPhase/streaming`` has the key
    /// `protocol.state.streaming.reason`.
    public var reasonKey: String {
        "protocol.state.\(rawValue).reason"
    }

    /// The name of the symbol that shows the state.
    ///
    /// WHY: The outlines of the symbols are different for every state: a
    /// dash for a protocol that is switched off, a dashed circle for a
    /// protocol that never ran, two bars for a service that is stopped, an
    /// arrow for a service that starts or stops, waves for a service that
    /// advertises itself, a triangle with a bar for a running stream and a
    /// triangle with a sign of warning for a failure. The name is a text
    /// and not a framework, so the kit stays free of SwiftUI.
    public var symbolName: String {
        switch self {
        case .disabled:
            return "minus.circle.fill"
        case .notStarted:
            return "circle.dashed"
        case .stopped:
            return "pause.circle.fill"
        case .starting:
            return "arrow.up.circle.fill"
        case .advertising:
            return "dot.radiowaves.left.and.right"
        case .streaming:
            return "play.circle.fill"
        case .stopping:
            return "arrow.down.circle.fill"
        case .failed:
            return "exclamationmark.triangle.fill"
        }
    }

    /// The tone of the state.
    public var tone: ShellTone {
        switch self {
        case .disabled, .notStarted, .stopped:
            return .idle
        case .starting, .stopping:
            return .transition
        case .advertising:
            return .advertising
        case .streaming:
            return .streaming
        case .failed:
            return .failure
        }
    }
}
