import SwiftUI
import PhairPlayMacKit

/// The colours of the window.
///
/// WHAT: the colours of the surfaces of the window, the colour of a tone
/// of the shell and the colour of the frame that shows the focus.
///
/// WHY: The tones are the tones of the kit, which holds them as numbers so
/// that the window of this shell and the tests of the kit cannot drift
/// apart. The frame of the focus is drawn by the shell and not by the
/// system, because the frame of macOS is thin and dark and is hard to see
/// on a dark background.
///
/// EXAMPLE: a card whose engine advertises itself draws its state in the
/// tone ``ShellTone/advertising``.
enum ShellTheme {

    /// The colour of a tone of the shell.
    ///
    /// - Parameter tone: the tone.
    /// - Returns: the colour of the tone.
    static func color(of tone: ShellTone) -> Color {
        Color(hex: tone.hex)
    }

    /// The colour of the frame that shows the focus.
    ///
    /// WHY: The frame has to be seen on a light and on a dark background,
    /// and it has to be seen from a distance. A blue that is bright enough
    /// for a dark room is still bright enough for a bright one.
    static let focus = Color(hex: "4DA3FF")

    /// The colour of the surface of a card.
    static let surface = Color(nsColor: .controlBackgroundColor)

    /// The colour of the surface of the head of the window.
    static let header = Color(nsColor: .windowBackgroundColor)
}

/// A colour of the shell, written as a hexadecimal number.
///
/// WHY: The colours of the shell are written down as numbers, so that the
/// tones of the kit and the colours of the window cannot drift apart.
extension Color {
    /// The colour of a hexadecimal number of the form `RRGGBB`.
    ///
    /// - Parameter hex: the number, with or without a leading `#`.
    init(hex: String) {
        var value: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&value)
        self.init(
            .sRGB,
            red: Double((value >> 16) & 0xFF) / 255.0,
            green: Double((value >> 8) & 0xFF) / 255.0,
            blue: Double(value & 0xFF) / 255.0,
            opacity: 1.0
        )
    }
}
