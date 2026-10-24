import SwiftUI
import PhairPlayMacKit

/// The visible frame that shows that a surface holds the focus.
///
/// WHAT: a frame of the colour of the focus, three points wide, with a
/// soft light around it.
///
/// WHY: The frame of macOS is thin and dark, and it is hard to see on a
/// dark background. The shell is read from a distance — from a sofa, from
/// a table — and a focus that cannot be seen is a focus that does not
/// exist. The shell therefore draws its own frame, and it switches the
/// frame of the system off where the system offers that.
///
/// EXAMPLE: `.focusFrame(isFocused: true)` draws the frame around a card
/// of the grid of the protocols.
struct FocusFrame: ViewModifier {

    /// Whether the surface holds the focus.
    let isFocused: Bool

    /// The radius of the corners of the frame.
    let cornerRadius: CGFloat

    /// The content with the frame of the focus around it.
    func body(content: Content) -> some View {
        content
            .overlay(
                RoundedRectangle(cornerRadius: cornerRadius)
                    .strokeBorder(
                        isFocused ? ShellTheme.focus : Color.clear,
                        lineWidth: ShellTypography.focusRingWidth
                    )
                    .shadow(
                        color: isFocused ? ShellTheme.focus.opacity(0.55) : Color.clear,
                        radius: 6
                    )
            )
    }
}

extension View {

    /// Draws the frame of the focus around the view.
    ///
    /// WHY: Every surface of the focus draws the frame the same way, so
    /// that the user sees one kind of focus in the whole window — the
    /// banner of a failure and the cards of the grid look the same when
    /// they hold the focus.
    ///
    /// - Parameters:
    ///   - isFocused: whether the surface holds the focus.
    ///   - cornerRadius: the radius of the corners of the frame.
    /// - Returns: the view with the frame around it.
    func focusFrame(
        _ isFocused: Bool,
        cornerRadius: CGFloat = ShellTypography.cornerRadius
    ) -> some View {
        modifier(FocusFrame(isFocused: isFocused, cornerRadius: cornerRadius))
    }
}
