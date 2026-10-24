import SwiftUI
import PhairPlayMacKit

/// The view of the session that is running.
///
/// WHAT: the sender, the formats of the stream, the metadata of the track
/// and the artwork that the sender sent.
///
/// WHY: The sender of a stream is the reason a user opens the application
/// while something is playing. The view therefore leads with the name of
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
        VStack(alignment: .leading, spacing: 8) {
            Text(ReceiverStrings.text("nowplaying.title"))
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
            if let session {
                content(of: session)
            } else {
                Text(ReceiverStrings.text("nowplaying.empty"))
                    .font(.system(size: 14))
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(Color(nsColor: .controlBackgroundColor))
        .cornerRadius(10)
    }

    /// The content of the view for a running session.
    private func content(of session: NowPlayingState) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .top, spacing: 12) {
                if let artwork = session.artwork, let image = NSImage(data: artwork) {
                    Image(nsImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 72, height: 72)
                        .cornerRadius(6)
                }
                VStack(alignment: .leading, spacing: 2) {
                    Text(session.title ?? ReceiverStrings.text("nowplaying.title.unknown"))
                        .font(.system(size: 16, weight: .medium))
                    Text(session.artist ?? ReceiverStrings.text("nowplaying.artist.unknown"))
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                    Text(session.album ?? ReceiverStrings.text("nowplaying.album.unknown"))
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
            }
            Text(session.senderName ?? ReceiverStrings.text("nowplaying.sender.unknown"))
                .font(.system(size: 14))
            if let address = session.senderAddress, !address.isEmpty {
                Text(address)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            if let video = session.video {
                Text(ReceiverStrings.text("nowplaying.video", [video]))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            } else {
                Text(ReceiverStrings.text("nowplaying.video.unknown"))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            if let audio = session.audio {
                Text(ReceiverStrings.text("nowplaying.audio", [audio]))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            } else {
                Text(ReceiverStrings.text("nowplaying.audio.unknown"))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            Text(ReceiverStrings.text(
                "nowplaying.uptime",
                [ReceiverMapping.uptimeText(session.uptimeMs)]
            ))
            .font(.system(size: 12))
            .foregroundStyle(.secondary)
        }
    }
}
