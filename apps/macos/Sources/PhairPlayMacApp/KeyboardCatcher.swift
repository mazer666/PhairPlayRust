import AppKit
import SwiftUI
import PhairPlayMacKit

/// The view that reads the keys of the window.
///
/// WHAT: an AppKit view that becomes the first responder of the window and
/// hands the keys of the shell over to the window.
///
/// WHY: macOS moves the focus with the tab key only when the full
/// keyboard access is switched on, and it offers no key at all for the
/// space bar in a window that is not a form. The shell therefore reads the
/// keys itself: the order of the focus is the one of the kit, and the
/// space bar switches the protocol of the card that holds the focus.
///
/// The view does not take part in the mouse: it never takes a click, so
/// that the gestures of the cards keep working.
///
/// EXAMPLE: `KeyboardCatcher { key in handle(key) }` at the bottom of the
/// window reports every key of the shell.
struct KeyboardCatcher: NSViewRepresentable {

    /// The call that receives the keys of the shell.
    let onKey: (ShellKey) -> Void

    /// Creates the view.
    func makeNSView(context: Context) -> ShellKeyView {
        let view = ShellKeyView()
        view.onKey = onKey
        return view
    }

    /// Hands the current call over to the view.
    func updateNSView(_ nsView: ShellKeyView, context: Context) {
        nsView.onKey = onKey
        nsView.claimFirstResponder()
    }
}

/// The AppKit view that reads the keys of the window.
///
/// WHY: The mapping of the keys of the keyboard to the keys of the shell
/// lives in the kit and is tested there; this view only asks AppKit for
/// the number of the key and for the shift key.
final class ShellKeyView: NSView {

    /// The call that receives the keys of the shell.
    var onKey: ((ShellKey) -> Void)?

    /// The view takes the focus, so that the keys arrive here.
    override var acceptsFirstResponder: Bool {
        true
    }

    /// The view never takes a click.
    ///
    /// WHY: The view lies behind the content of the window. It has to be
    /// invisible for the mouse, so that the clicks on the cards and on the
    /// banner keep reaching them.
    override func hitTest(_ point: NSPoint) -> NSView? {
        nil
    }

    /// The view has moved into a window and asks for the focus.
    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        claimFirstResponder()
    }

    /// The view asks the window for the focus.
    ///
    /// WHY: The window has no field of text, so the keys of the shell
    /// always belong to this view. A control that took the focus — a
    /// button of the banner, for example — gives it back as soon as the
    /// state of the receiver changes.
    func claimFirstResponder() {
        guard let window, window.firstResponder !== self else {
            return
        }
        window.makeFirstResponder(self)
    }

    /// A key of the keyboard was pressed.
    override func keyDown(with event: NSEvent) {
        let shift = event.modifierFlags.contains(.shift)
        guard let key = ShellKeyMap.key(forKeyCode: Int(event.keyCode), shift: shift) else {
            super.keyDown(with: event)
            return
        }
        onKey?(key)
    }
}
