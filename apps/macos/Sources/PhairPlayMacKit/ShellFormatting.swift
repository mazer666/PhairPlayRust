import Foundation

/// The texts of the window of the shell.
///
/// WHAT: the sentences that the window shows, built out of the string
/// catalogue of the shell.
///
/// WHY: A view that builds its own sentences cannot be translated and
/// cannot be reviewed by somebody who does not read Swift. Every text of
/// the window therefore comes from the catalogue through this file, and
/// the sentences that depend on the state of the receiver — the ports, the
/// number of senders, the running time — are built here, where they can
/// be tested.
///
/// EXAMPLE: the ports of a card that listens on TCP `7000` and on no UDP
/// port read "TCP 7000".
public enum ShellFormatting {

    /// The name of a protocol that the user reads.
    ///
    /// - Parameter identifier: the protocol.
    /// - Returns: the name of the protocol in the language of the system.
    public static func protocolName(_ identifier: ProtocolId) -> String {
        ReceiverStrings.text("protocol.\(identifier.rawValue)")
    }

    /// The state of a protocol as the user reads it.
    ///
    /// - Parameter phase: the phase of the protocol.
    /// - Returns: the word of the state in the language of the system.
    public static func stateText(_ phase: ProtocolPhase) -> String {
        ReceiverStrings.text(phase.textKey)
    }

    /// The sentence that explains the state of a protocol.
    ///
    /// - Parameter phase: the phase of the protocol.
    /// - Returns: the sentence in the language of the system.
    public static func reasonText(_ phase: ProtocolPhase) -> String {
        ReceiverStrings.text(phase.reasonKey)
    }

    /// The state of the receiver as the user reads it.
    ///
    /// - Parameter status: the state of the receiver.
    /// - Returns: the word of the state in the language of the system.
    public static func statusText(_ status: ReceiverStatus) -> String {
        ReceiverStrings.text(status.textKey)
    }

    /// The sentence that explains the state of the receiver.
    ///
    /// - Parameter status: the state of the receiver.
    /// - Returns: the sentence in the language of the system.
    public static func statusDetail(_ status: ReceiverStatus) -> String {
        ReceiverStrings.text(status.detailKey)
    }

    /// The sentence about the ports of a protocol.
    ///
    /// WHY: A protocol that listens on TCP and on UDP shows both ports, a
    /// protocol that listens on one of them shows that one, and a protocol
    /// that listens on none says so — a port that is not shown looks like
    /// a port that was forgotten.
    ///
    /// - Parameter card: the card of the protocol.
    /// - Returns: the sentence in the language of the system.
    public static func ports(of card: ProtocolCardState) -> String {
        switch (card.tcpPort, card.udpPort) {
        case (.some(let tcp), .some(let udp)):
            return ReceiverStrings.text("protocol.ports", ["\(tcp)", "\(udp)"])
        case (.some(let tcp), .none):
            return ReceiverStrings.text("protocol.ports.tcp", ["\(tcp)"])
        case (.none, .some(let udp)):
            return ReceiverStrings.text(
                "protocol.ports",
                [ReceiverStrings.text("protocol.ports.unknown"), "\(udp)"]
            )
        case (.none, .none):
            return ReceiverStrings.text("protocol.ports.none")
        }
    }

    /// The sentence about the senders that are connected.
    ///
    /// - Parameter card: the card of the protocol.
    /// - Returns: the sentence in the language of the system.
    public static func peers(of card: ProtocolCardState) -> String {
        let count = "\(card.peerCount)"
        return ReceiverStrings.text(
            card.peerCount == 1 ? "protocol.peers.one" : "protocol.peers.many",
            [count]
        )
    }

    /// The sentence about the running time of a session.
    ///
    /// - Parameter milliseconds: how long the session has been running.
    /// - Returns: the sentence in the language of the system.
    public static func uptime(_ milliseconds: UInt64) -> String {
        ReceiverStrings.text(
            "nowplaying.uptime",
            [ReceiverMapping.uptimeText(milliseconds)]
        )
    }

    /// The sentence that names the device of the receiver.
    ///
    /// - Parameter name: the name of the device.
    /// - Returns: the sentence in the language of the system.
    public static func device(_ name: String) -> String {
        ReceiverStrings.text("dashboard.device", [name])
    }

    /// The sentence that names the version of the build.
    ///
    /// - Parameter version: the version of the build.
    /// - Returns: the sentence in the language of the system.
    public static func version(_ version: String) -> String {
        ReceiverStrings.text("dashboard.version", [version])
    }

    /// The sentence that says what the space bar does with a protocol.
    ///
    /// - Parameters:
    ///   - identifier: the protocol.
    ///   - enabled: whether the protocol is switched on.
    /// - Returns: the sentence in the language of the system.
    public static func switchAction(
        of identifier: ProtocolId,
        enabled: Bool
    ) -> String {
        ReceiverStrings.text(
            enabled ? "protocol.switch.off" : "protocol.switch.on",
            [protocolName(identifier)]
        )
    }

    /// The name of a shortcut of the shell.
    ///
    /// - Parameter shortcut: the group of the keys.
    /// - Returns: the name of the group in the language of the system.
    public static func shortcutTitle(_ shortcut: ShellShortcut) -> String {
        ReceiverStrings.text(shortcut.titleKey)
    }

    /// The sentence that says what a shortcut of the shell does.
    ///
    /// - Parameter shortcut: the group of the keys.
    /// - Returns: the sentence in the language of the system.
    public static func shortcutDetail(_ shortcut: ShellShortcut) -> String {
        ReceiverStrings.text(shortcut.detailKey)
    }

    /// The shortcut with its texts.
    ///
    /// - Parameter shortcut: the group of the keys.
    /// - Returns: the group and the texts of the catalogue.
    public static func shortcutText(_ shortcut: ShellShortcut) -> ShellShortcutText {
        ShellShortcutText(
            shortcut: shortcut,
            title: shortcutTitle(shortcut),
            detail: shortcutDetail(shortcut)
        )
    }

    /// The shortcuts of the shell, in the order of the hint.
    ///
    /// WHY: The hint at the foot of the window is built out of these
    /// texts, so that the hint names exactly the keys that the shell
    /// answers to. The order is the order of the groups of the kit.
    ///
    /// - Returns: the texts of every shortcut of the shell.
    public static func shortcutTexts() -> [ShellShortcutText] {
        ShellShortcut.all.map(shortcutText)
    }
}
