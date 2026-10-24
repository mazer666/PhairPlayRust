import Foundation

/// The texts of the shell.
///
/// WHAT: the reader of the string catalogue of the shell.
///
/// WHY: Every text the user can read lives in a string catalogue and
/// nowhere else. A text that is written into the source of a view cannot
/// be translated, and it cannot be reviewed by somebody who does not read
/// Swift. The catalogue of the shell carries an English and a German
/// version; the reader answers the text of the language of the system, and
/// the key itself when a text is missing — a missing text is visible, but
/// it never crashes the application.
///
/// EXAMPLE: `ReceiverStrings.text("protocol.state.disabled")` answers
/// "Disabled in the settings" in English and "In den Einstellungen
/// deaktiviert" in German.
public enum ReceiverStrings {

    /// The text of a key.
    ///
    /// - Parameter key: the key of the catalogue.
    /// - Returns: the text of the catalogue, or the key when the catalogue
    ///   does not carry it.
    public static func text(_ key: String) -> String {
        NSLocalizedString(key, tableName: nil, bundle: .module, value: key, comment: "")
    }

    /// The text of a key with arguments.
    ///
    /// WHY: The placeholders of the catalogue are all `%@`, so that the
    /// reader never has to guess the width of a number on the platform.
    ///
    /// - Parameters:
    ///   - key: the key of the catalogue.
    ///   - arguments: the values of the placeholders.
    /// - Returns: the text of the catalogue with the arguments in it.
    public static func text(_ key: String, _ arguments: [String]) -> String {
        let template = text(key)
        var result = template
        for argument in arguments {
            guard let range = result.range(of: "%@") else {
                break
            }
            result.replaceSubrange(range, with: argument)
        }
        return result
    }
}
