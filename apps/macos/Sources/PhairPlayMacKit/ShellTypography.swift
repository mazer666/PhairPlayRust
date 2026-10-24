import Foundation

/// The sizes and the measures of the window.
///
/// WHAT: the size of every kind of text of the shell and the measures of
/// the layout: the padding of the window, the distance between the cards,
/// the width of the frame of the focus and the smallest size of the
/// window.
///
/// WHY: The shell is read from a distance — from a sofa, from a table, and
/// sometimes from the other end of the room. The text therefore has a
/// minimum size, and the sizes are written down here instead of being
/// spread over the views, so that a card, the head of the window and the
/// area of the session all speak with the same voice.
///
/// The window has to hold the grid of the four cards without a horizontal
/// scroll bar, so the smallest width is the width of two cards and their
/// distance.
///
/// EXAMPLE: the name of a protocol is drawn with `cardTitle` (22 points),
/// the word of its state with `state` (20 points) and the sentences
/// underneath with `detail` (16 points).
public enum ShellTypography {

    /// The smallest size of the title of the window, in points.
    public static let title: CGFloat = 28

    /// The size of the word of the state of the receiver, in points.
    ///
    /// WHY: The word of the state of the receiver is the largest text of
    /// the window: it is the answer to the question "can I send something
    /// to this device right now?".
    public static let headline: CGFloat = 34

    /// The size of the name of a protocol on its card, in points.
    public static let cardTitle: CGFloat = 22

    /// The size of the word of the state of a protocol, in points.
    public static let state: CGFloat = 20

    /// The size of the details — the sentences, the ports and the number
    /// of senders, in points.
    public static let detail: CGFloat = 16

    /// The size of the short labels, in points.
    public static let label: CGFloat = 16

    /// The distance between the cards of the grid, in points.
    public static let gridSpacing: CGFloat = 16

    /// The padding inside a card, in points.
    public static let cardPadding: CGFloat = 16

    /// The padding around the content of the window, in points.
    public static let contentPadding: CGFloat = 20

    /// The radius of the corners of a card, in points.
    public static let cornerRadius: CGFloat = 14

    /// The width of the frame that shows the focus, in points.
    ///
    /// WHY: The frame of the focus of macOS is thin and dark, and it is
    /// hard to see on a dark background. The shell draws its own frame,
    /// and it is thick enough to be seen from a distance.
    public static let focusRingWidth: CGFloat = 3

    /// The smallest width of the window, in points.
    public static let minimumWindowWidth: CGFloat = 760

    /// The smallest height of the window, in points.
    public static let minimumWindowHeight: CGFloat = 620

    /// The size of the shape of the state in the head of the window, in
    /// points.
    ///
    /// WHY: The shape of the state is read from across the room, like the
    /// word of the state, and it has to be as large as the word.
    public static let headlineSymbol: CGFloat = 46

    /// The width of the switch of a card, in points.
    public static let switchWidth: CGFloat = 54

    /// The height of the switch of a card, in points.
    public static let switchHeight: CGFloat = 30

    /// The diameter of the knob of the switch of a card, in points.
    public static let switchKnob: CGFloat = 24

    /// The distance between the knob of the switch and its edge, in points.
    public static let switchInset: CGFloat = 3

    /// The radius of the corners of the button of the banner, in points.
    public static let buttonCornerRadius: CGFloat = 8

    /// The padding around the text of the button of the banner, in points.
    public static let buttonPadding: CGFloat = 10

    /// The size of the artwork of a track, in points.
    public static let artworkSize: CGFloat = 96

    /// The radius of the corners of the artwork of a track, in points.
    public static let artworkCornerRadius: CGFloat = 8
}
