import SwiftUI
import PhairPlayMacKit

/// The head of the window.
///
/// WHAT: the state of the receiver as a big word with a big shape, the
/// sentence that explains it, and the identity of the device.
///
/// WHY: The head of the window is the part that is read from across the
/// room. It answers the one question a user has when the window opens:
/// "can I send something to this device right now?". The state is shown
/// as a word and as a shape, so that it can be read without the colour.
///
/// EXAMPLE: a receiver that advertises itself shows a tick in a circle,
/// the word "Running" and the sentence that says that the receiver is
/// visible to the senders of the local network.
struct StatusHeaderView: View {

    /// The state of the receiver.
    let status: ReceiverStatus

    /// The name of the device that the receiver advertises.
    let deviceName: String

    /// The version of the build that carries the bridge.
    let version: String

    var body: some View {
        HStack(alignment: .center, spacing: 20) {
            Image(systemName: status.symbolName)
                .font(.system(size: ShellTypography.headlineSymbol, weight: .semibold))
                .foregroundStyle(ShellTheme.color(of: status.tone))
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 4) {
                Text(ShellFormatting.statusText(status))
                    .font(.system(size: ShellTypography.headline, weight: .bold))
                    .foregroundStyle(ShellTheme.color(of: status.tone))
                Text(ShellFormatting.statusDetail(status))
                    .font(.system(size: ShellTypography.detail))
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 16)
            VStack(alignment: .trailing, spacing: 4) {
                Text(ShellFormatting.device(deviceName))
                    .font(.system(size: ShellTypography.detail, weight: .medium))
                if !version.isEmpty {
                    Text(ShellFormatting.version(version))
                        .font(.system(size: ShellTypography.label))
                        .foregroundStyle(.secondary)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, ShellTypography.contentPadding)
        .padding(.vertical, 18)
        .background(ShellTheme.header)
    }
}
