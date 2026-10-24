import SwiftUI
import PhairPlayMacKit

/// The area of the session that is running.
///
/// WHAT: the sender, the metadata of the track, the formats of the stream
/// and the artwork that the sender sent.
///
/// WHY: The sender of a stream is the reason a user opens the application
/// while something is playing. The area therefore leads with the name of
/// the sender and with the track, and it shows the technical details
/// underneath — a user who does not care about the sample rate does not
/// have to read it.
///
/// EXAMPLE: `NowPlayingView(session: session)` draws the session of an
/// AirPlay sender that plays a track.
struct NowPlayingView: View {

    /// The session that is running, if there is one.
    let session: NowPlayingState?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(ReceiverStrings.text("nowplaying.title"))
                .font(.system(size: ShellTypography.cardTitle, weight: .semibold))
            if let session {
                content(of: session)
            } else {
                Text(ReceiverStrings.text("nowplaying.empty"))
                    .font(.system(size: ShellTypography.state))
                    .foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(ShellTypography.cardPadding)
        .background(ShellTheme.surface)
        .clipShape(RoundedRectangle(cornerRadius: ShellTypography.cornerRadius))
    }

    /// The content of the area for a running session.
    private func content(of session: NowPlayingState) -> some View {
        HStack(alignment: .top, spacing: 16) {
            artwork(of: session)
            VStack(alignment: .leading, spacing: 6) {
                Text(session.title ?? ReceiverStrings.text("nowplaying.title.unknown"))
                    .font(.system(size: ShellTypography.cardTitle, weight: .medium))
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)
                Text(session.artist ?? ReceiverStrings.text("nowplaying.artist.unknown"))
                    .font(.system(size: ShellTypography.state))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                Text(session.album ?? ReceiverStrings.text("nowplaying.album.unknown"))
                    .font(.system(size: ShellTypography.detail))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                Text(session.senderName ?? ReceiverStrings.text("nowplaying.sender.unknown"))
                    .font(.system(size: ShellTypography.state, weight: .medium))
                    .padding(.top, 4)
                if let address = session.senderAddress, !address.isEmpty {
                    Text(address)
                        .font(.system(size: ShellTypography.detail))
                        .foregroundStyle(.secondary)
                }
                HStack(spacing: 16) {
                    Text(video(of: session))
                        .font(.system(size: ShellTypography.detail))
                        .foregroundStyle(.secondary)
                    Text(audio(of: session))
                        .font(.system(size: ShellTypography.detail))
                        .foregroundStyle(.secondary)
                    Text(ShellFormatting.uptime(session.uptimeMs))
                        .font(.system(size: ShellTypography.detail))
                        .foregroundStyle(.secondary)
                }
                .padding(.top, 4)
            }
        }
    }

    /// The artwork of the track, if the sender sent one.
    @ViewBuilder
    private func artwork(of session: NowPlayingState) -> some View {
        if let artwork = session.artwork, let image = NSImage(data: artwork) {
            Image(nsImage: image)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: ShellTypography.artworkSize, height: ShellTypography.artworkSize)
                .clipShape(
                    RoundedRectangle(cornerRadius: ShellTypography.artworkCornerRadius)
                )
        }
    }

    /// The sentence about the video of the stream.
    private func video(of session: NowPlayingState) -> String {
        guard let video = session.video else {
            return ReceiverStrings.text("nowplaying.video.unknown")
        }
        return ReceiverStrings.text("nowplaying.video", [video])
    }

    /// The sentence about the audio of the stream.
    private func audio(of session: NowPlayingState) -> String {
        guard let audio = session.audio else {
            return ReceiverStrings.text("nowplaying.audio.unknown")
        }
        return ReceiverStrings.text("nowplaying.audio", [audio])
    }
}
