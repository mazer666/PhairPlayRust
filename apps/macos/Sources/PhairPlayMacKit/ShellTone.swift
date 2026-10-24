import Foundation

/// The tone of a state of the shell.
///
/// WHAT: the five colours that the shell uses for the states of a protocol
/// and for the state of the receiver: a calm grey for a state that does
/// nothing, an amber for a state that is on its way, a green for a service
/// that advertises itself, a teal for a running stream and a red for a
/// failure.
///
/// WHY: The receiver is an application that runs in the background of a
/// living room. The colours are calm on purpose: a bright red would shine
/// through a dark room and would suggest an error where there is none. The
/// red is reserved for a real failure.
///
/// The five tones are the five slots of the product — the shell of Android
/// and the dashboard of the web carry the same slots under the same names
/// ("disabled", "transition", "advertising", "streaming", "error"). The
/// **slot** is shared, so that a state means the same thing on every
/// platform; the **colour** is the one of this shell. The three palettes
/// are not the same today, and the question of one palette for all of them
/// is an open point of the audit of this shell.
///
/// The tone is only ever one of several channels: every state is shown as a
/// word and as a shape as well. A user who cannot tell the colours apart
/// still reads the state.
///
/// EXAMPLE: the phase ``ProtocolPhase/advertising`` has the tone
/// ``ShellTone/advertising``, which is the colour `1F8A5B`.
public enum ShellTone: String, Sendable, CaseIterable {

    /// A state that does nothing: switched off, never started, stopped.
    case idle

    /// A state that is on its way: starting or stopping.
    case transition

    /// A service that advertises itself and waits for a sender.
    case advertising

    /// A sender is connected and the stream is running.
    case streaming

    /// A failure of the service.
    case failure

    /// The colour of the tone as a hexadecimal number of the form `RRGGBB`.
    ///
    /// WHY: The number is written down here and not in the view, so that
    /// the tones of this shell are compared in one place and not in two.
    public var hex: String {
        switch self {
        case .idle:
            return "5B6672"
        case .transition:
            return "8A6D1F"
        case .advertising:
            return "1F8A5B"
        case .streaming:
            return "0F7C8A"
        case .failure:
            return "B3261E"
        }
    }
}
