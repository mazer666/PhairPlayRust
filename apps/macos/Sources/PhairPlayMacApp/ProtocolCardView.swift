import SwiftUI
import PhairPlayMacKit

/// The card of one protocol.
///
/// WHAT: the name of the protocol, the state of its engine as a word and
/// as a shape, the sentence that explains the state, the ports it listens
/// on, the number of senders that are connected and the switch that
/// switches the protocol on and off.
///
/// WHY: A user has to be able to see at a glance which protocols are
/// running and which ones are switched off. The card therefore shows the
/// state as a word and not only as a colour, and the state "switched off
/// in the settings" is a state of its own — it is not the same as "the
/// service is stopped".
///
/// The card is the surface that holds the focus while the user walks
/// through the grid with the keyboard. It draws its own frame, because the
/// frame of macOS is too weak on a dark background.
///
/// EXAMPLE: `ProtocolCardView(card: card, isFocused: true, onToggle: …)`
/// draws the card of the AirPlay engine and reports every change of the
/// switch.
struct ProtocolCardView: View {

    /// The state of the protocol.
    let card: ProtocolCardState

    /// Whether the card holds the focus.
    let isFocused: Bool

    /// The call that reports a change of the switch.
    let onToggle: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            state
            reason
            details
            failure
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(ShellTypography.cardPadding)
        .background(ShellTheme.surface)
        .clipShape(RoundedRectangle(cornerRadius: ShellTypography.cornerRadius))
        .contentShape(RoundedRectangle(cornerRadius: ShellTypography.cornerRadius))
        .focusFrame(isFocused)
    }

    /// The name of the protocol and the switch that switches it on and off.
    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            Text(ShellFormatting.protocolName(card.protocolId))
                .font(.system(size: ShellTypography.cardTitle, weight: .semibold))
                .lineLimit(1)
                .minimumScaleFactor(0.8)
            Spacer(minLength: 8)
            ProtocolSwitch(
                isOn: card.enabled,
                label: ShellFormatting.switchAction(of: card.protocolId, enabled: card.enabled),
                onToggle: onToggle
            )
        }
    }

    /// The state of the engine as a word and as a shape.
    private var state: some View {
        HStack(spacing: 8) {
            Image(systemName: card.phase.symbolName)
                .font(.system(size: ShellTypography.state, weight: .medium))
                .foregroundStyle(ShellTheme.color(of: card.phase.tone))
                .accessibilityHidden(true)
            Text(ShellFormatting.stateText(card.phase))
                .font(.system(size: ShellTypography.state, weight: .semibold))
                .foregroundStyle(ShellTheme.color(of: card.phase.tone))
                .lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)
        }
    }

    /// The sentence that explains the state.
    private var reason: some View {
        Text(ShellFormatting.reasonText(card.phase))
            .font(.system(size: ShellTypography.detail))
            .foregroundStyle(.secondary)
            .fixedSize(horizontal: false, vertical: true)
    }

    /// The ports and the number of senders.
    private var details: some View {
        HStack(spacing: 16) {
            Text(ShellFormatting.ports(of: card))
                .font(.system(size: ShellTypography.detail))
                .foregroundStyle(.secondary)
            if card.peerCount > 0 {
                Text(ShellFormatting.peers(of: card))
                    .font(.system(size: ShellTypography.detail))
                    .foregroundStyle(.secondary)
            }
        }
    }

    /// The reason of the last failure of the engine.
    @ViewBuilder
    private var failure: some View {
        if let reason = card.lastError, !reason.isEmpty {
            Text(reason)
                .font(.system(size: ShellTypography.detail))
                .foregroundStyle(ShellTheme.color(of: .failure))
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}

/// The switch that switches a protocol on and off.
///
/// WHAT: a drawn switch with a knob, and the call that reports that the
/// user pressed it.
///
/// WHY: The switch of the shell is drawn and not taken from the system,
/// because a system switch takes the focus of the window when it is
/// clicked — and the focus of this window belongs to the grid of the
/// cards. The switch therefore carries the traits of a button and its
/// label, so that it is still announced as a switch by the screen reader.
///
/// EXAMPLE: `ProtocolSwitch(isOn: true, label: …, onToggle: …)` draws a
/// switch that is on and reports every click.
struct ProtocolSwitch: View {

    /// Whether the protocol is switched on.
    let isOn: Bool

    /// The sentence that says what the switch does.
    let label: String

    /// The call that reports that the user pressed the switch.
    let onToggle: () -> Void

    var body: some View {
        ZStack(alignment: isOn ? .trailing : .leading) {
            Capsule()
                .fill(isOn ? ShellTheme.color(of: .advertising) : Color.secondary.opacity(0.35))
                .frame(width: ShellTypography.switchWidth, height: ShellTypography.switchHeight)
            Circle()
                .fill(Color.white)
                .frame(width: ShellTypography.switchKnob, height: ShellTypography.switchKnob)
                .shadow(radius: 1)
                .padding(.horizontal, ShellTypography.switchInset)
        }
        .contentShape(Rectangle())
        .onTapGesture { onToggle() }
        .help(label)
        .accessibilityElement()
        .accessibilityLabel(label)
        .accessibilityValue(
            isOn
                ? ReceiverStrings.text("protocol.switch.state.on")
                : ReceiverStrings.text("protocol.switch.state.off")
        )
        .accessibilityAddTraits(.isButton)
    }
}
