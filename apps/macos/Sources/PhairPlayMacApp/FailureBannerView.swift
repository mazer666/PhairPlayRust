import SwiftUI
import PhairPlayMacKit

/// The banner of a failure of the bridge.
///
/// WHAT: the explanation of the failure, the code of the failure and the
/// button that closes the banner.
///
/// WHY: The bridge reports a failure with a code and an explanation. The
/// banner shows both, because the code is the part that a bug report needs
/// and the explanation is the part that the user needs.
///
/// The banner is a surface of the focus: the tab key walks to it, and the
/// escape key closes it. The escape key is the fast way out — a failure
/// that is in the way of the grid must not have to be closed with the
/// mouse.
///
/// EXAMPLE: `FailureBannerView(failure: failure, isFocused: false, onDismiss: …)`
/// draws the banner of a failure of the bridge.
struct FailureBannerView: View {

    /// The failure that the bridge reported.
    let failure: BridgeFailure

    /// Whether the banner holds the focus.
    let isFocused: Bool

    /// The call that hides the banner.
    let onDismiss: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 16) {
            Image(systemName: ReceiverStatus.error.symbolName)
                .font(.system(size: ShellTypography.state, weight: .semibold))
                .foregroundStyle(ShellTheme.color(of: .failure))
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 4) {
                Text(ReceiverStrings.text("failure.title"))
                    .font(.system(size: ShellTypography.state, weight: .semibold))
                    .foregroundStyle(ShellTheme.color(of: .failure))
                Text(ReceiverStrings.text("failure.code", ["\(failure.code)"]))
                    .font(.system(size: ShellTypography.detail, weight: .medium))
                    .foregroundStyle(ShellTheme.color(of: .failure))
                Text(failure.details)
                    .font(.system(size: ShellTypography.detail))
                    .fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 12)
            DismissButton(
                label: ReceiverStrings.text("failure.dismiss"),
                onPress: onDismiss
            )
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(ShellTypography.cardPadding)
        .background(ShellTheme.color(of: .failure).opacity(0.10))
        .clipShape(RoundedRectangle(cornerRadius: ShellTypography.cornerRadius))
        .contentShape(RoundedRectangle(cornerRadius: ShellTypography.cornerRadius))
        .focusFrame(isFocused)
    }
}

/// The button that closes the banner of a failure.
///
/// WHAT: a drawn button with the name of the action.
///
/// WHY: A button of the system takes the focus of the window when it is
/// clicked, and the focus of this window belongs to the surfaces of the
/// ring of the focus. The button is therefore drawn and not taken from the
/// system, and it carries the traits of a button, so that it is still
/// announced as a button by the screen reader.
///
/// EXAMPLE: `DismissButton(label: "Dismiss", onPress: …)` draws the button
/// and reports every click.
struct DismissButton: View {

    /// The name of the action.
    let label: String

    /// The call that reports that the user pressed the button.
    let onPress: () -> Void

    var body: some View {
        Text(label)
            .font(.system(size: ShellTypography.detail, weight: .semibold))
            .foregroundStyle(ShellTheme.color(of: .failure))
            .padding(.horizontal, ShellTypography.buttonPadding)
            .padding(.vertical, ShellTypography.buttonPadding / 2)
            .background(
                RoundedRectangle(cornerRadius: ShellTypography.buttonCornerRadius)
                    .fill(ShellTheme.color(of: .failure).opacity(0.18))
            )
            .overlay(
                RoundedRectangle(cornerRadius: ShellTypography.buttonCornerRadius)
                    .strokeBorder(
                        ShellTheme.color(of: .failure).opacity(0.65),
                        lineWidth: 1
                    )
            )
            .contentShape(Rectangle())
            .onTapGesture { onPress() }
            .help(label)
            .accessibilityElement()
            .accessibilityLabel(label)
            .accessibilityAddTraits(.isButton)
    }
}
