import SwiftUI
import PhairPlayMacKit

/// The card of one protocol.
///
/// WHAT: the name of the protocol, the state of its engine, the ports it
/// listens on, the number of senders that are connected and the switch
/// that switches the protocol on and off.
///
/// WHY: A user has to be able to see at a glance which protocols are
/// running and which ones are switched off. The card therefore shows the
/// state as a word and not only as a colour, and the state "switched off
/// in the settings" is a state of its own — it is not the same as "the
/// service is stopped".
///
/// EXAMPLE: `ProtocolCardView(card: card, onToggle: onToggle)` draws the
/// card of the AirPlay engine and reports every change of the switch.
struct ProtocolCardView: View {

    /// The state of the protocol.
    let card: ProtocolCardState

    /// The call that reports a change of the switch.
    let onToggle: (ProtocolId, Bool) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(name)
                    .font(.system(size: 17, weight: .medium))
                Spacer()
                Toggle(
                    "",
                    isOn: Binding(
                        get: { card.enabled },
                        set: { wanted in onToggle(card.protocolId, wanted) }
                    )
                )
                .labelsHidden()
                .help(ReceiverStrings.text(
                    card.enabled ? "protocol.switch.off" : "protocol.switch.on",
                    [name]
                ))
            }
            HStack(spacing: 6) {
                Circle()
                    .fill(Color(hex: tone))
                    .frame(width: 10, height: 10)
                Text(state)
                    .font(.system(size: 14, weight: .medium))
                    .foregroundStyle(Color(hex: tone))
            }
            Text(reason)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            HStack(spacing: 16) {
                Text(ports)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                if card.peerCount > 0 {
                    Text(peers)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
            }
            if let failure = card.lastError, !failure.isEmpty {
                Text(failure)
                    .font(.system(size: 12))
                    .foregroundStyle(Color.red)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(Color(nsColor: .controlBackgroundColor))
        .cornerRadius(10)
    }

    /// The name of the protocol that the user reads.
    private var name: String {
        ReceiverStrings.text("protocol.\(card.protocolId.rawValue)")
    }

    /// The state of the protocol as the user reads it.
    private var state: String {
        ReceiverStrings.text("protocol.state.\(card.phase.rawValue)")
    }

    /// The sentence that explains the state.
    private var reason: String {
        ReceiverStrings.text("protocol.state.\(card.phase.rawValue).reason")
    }

    /// The colour of the state.
    private var tone: String {
        switch card.phase {
        case .disabled, .notStarted, .stopped:
            return "5B6672"
        case .starting, .stopping:
            return "8A6D1F"
        case .advertising:
            return "1F8A5B"
        case .streaming:
            return "0F7C8A"
        case .failed:
            return "B3261E"
        }
    }

    /// The sentence about the ports of the protocol.
    private var ports: String {
        switch (card.tcpPort, card.udpPort) {
        case (.some(let tcp), .some(let udp)):
            return ReceiverStrings.text("protocol.ports", ["\(tcp)", "\(udp)"])
        case (.some(let tcp), .none):
            return ReceiverStrings.text("protocol.ports.tcp", ["\(tcp)"])
        case (.none, .some(let udp)):
            return ReceiverStrings.text("protocol.ports", ["—", "\(udp)"])
        case (.none, .none):
            return ReceiverStrings.text("protocol.ports.none")
        }
    }

    /// The sentence about the senders that are connected.
    private var peers: String {
        let count = "\(card.peerCount)"
        return ReceiverStrings.text(
            card.peerCount == 1 ? "protocol.peers.one" : "protocol.peers.many",
            [count]
        )
    }
}

/// A colour of the shell, written as a hexadecimal number.
///
/// WHY: The colours of the shell are the ones of the Android shell, and
/// they are written down as numbers so that the two platforms cannot
/// drift apart.
extension Color {
    /// The colour of a hexadecimal number of the form `RRGGBB`.
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
