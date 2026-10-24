import SwiftUI
import PhairPlayMacKit

/// The window of the shell.
///
/// WHAT: the one window of the application. It shows the head with the
/// identity of the receiver, the card of every protocol, the view of the
/// running session and the banner of a failure.
///
/// WHY: The window is the one place where the user changes something: the
/// switch of a protocol. The state behind it comes from the client, and
/// the client is the only object that talks to the bridge.
///
/// EXAMPLE: `DashboardView(client: client)` draws the whole window out of
/// the state of the client.
struct DashboardView: View {

    /// The client that carries the state of the receiver.
    @ObservedObject var client: ReceiverClient

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .frame(minWidth: 520, minHeight: 460)
    }

    /// The head of the window.
    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(ReceiverStrings.text("dashboard.title"))
                .font(.system(size: 20, weight: .semibold))
            Text(ReceiverStrings.text("dashboard.device", [client.state.deviceName]))
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
            Text(
                client.state.running
                    ? ReceiverStrings.text("dashboard.running")
                    : ReceiverStrings.text("dashboard.stopped")
            )
            .font(.system(size: 13))
            .foregroundStyle(client.state.running ? Color.green : Color.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
    }

    /// The body of the window.
    private var content: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let failure = client.state.failure {
                    FailureBanner(
                        failure: failure,
                        onDismiss: { client.dismissFailure() }
                    )
                }
                NowPlayingView(session: client.state.nowPlaying)
                ForEach(client.state.protocols, id: \.protocolId) { card in
                    ProtocolCardView(card: card) { protocolId, enabled in
                        client.setProtocolEnabled(protocolId, enabled: enabled)
                    }
                }
                if !client.state.version.isEmpty {
                    Text(ReceiverStrings.text("dashboard.version", [client.state.version]))
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
            }
            .padding(20)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

/// The banner of a failure of the bridge.
///
/// WHY: The bridge reports a failure with a code and an explanation. The
/// banner shows both, because the code is the part that a bug report needs
/// and the explanation is the part that the user needs.
struct FailureBanner: View {

    /// The failure that the bridge reported.
    let failure: BridgeFailure

    /// The call that hides the banner.
    let onDismiss: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(ReceiverStrings.text("failure.title"))
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Color.red)
            Text(ReceiverStrings.text("failure.code", ["\(failure.code)"]))
                .font(.system(size: 12))
                .foregroundStyle(Color.red)
            Text(failure.details)
                .font(.system(size: 12))
                .fixedSize(horizontal: false, vertical: true)
            Button(ReceiverStrings.text("failure.dismiss"), action: onDismiss)
                .buttonStyle(.link)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color.red.opacity(0.08))
        .cornerRadius(8)
    }
}
