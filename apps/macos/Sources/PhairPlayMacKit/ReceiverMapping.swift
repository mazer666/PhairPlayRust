import Foundation
import PhairPlayFFI

/// The translation of the objects of the bridge into the model of the shell.
///
/// WHAT: the pure functions that turn the data transfer objects of the
/// bridge into the structures that the window shows, and the configuration
/// of the receiver into the settings of the protocols.
///
/// WHY: The window must not know the DTOs of the bridge: a change of the
/// bridge would then ripple into every screen. The functions of this file
/// are the only place where the two vocabularies meet, and they are pure,
/// so that they can be tested without a running receiver.
///
/// EXAMPLE: `protocolPhase(enabled: false, state: .advertising)` answers
/// ``ProtocolPhase/disabled``, because a protocol that is switched off is
/// not advertised at all.
public enum ReceiverMapping {

    /// The phase of a protocol.
    ///
    /// - Parameters:
    ///   - enabled: whether the protocol is switched on in the settings.
    ///   - state: the state that the engine reported.
    /// - Returns: the phase that the card shows.
    public static func protocolPhase(enabled: Bool, state: FfiProtocolState) -> ProtocolPhase {
        if !enabled {
            return .disabled
        }
        switch state {
        case .idle:
            return .notStarted
        case .starting:
            return .starting
        case .advertising:
            return .advertising
        case .streaming:
            return .streaming
        case .stopping:
            return .stopping
        case .stopped:
            return .stopped
        case .failed:
            return .failed
        }
    }

    /// The card of one protocol.
    ///
    /// - Parameters:
    ///   - kind: the protocol the status belongs to.
    ///   - enabled: whether the protocol is switched on in the settings.
    ///   - status: the status that the engine reported.
    /// - Returns: the card, or `nil` when the protocol is not one of the
    ///   four that the shell knows.
    public static func protocolCard(
        kind: FfiProtocolKind,
        enabled: Bool,
        status: FfiProtocolStatus
    ) -> ProtocolCardState? {
        guard let identifier = protocolId(of: kind) else {
            return nil
        }
        return ProtocolCardState(
            protocolId: identifier,
            phase: protocolPhase(enabled: enabled, state: status.state),
            enabled: enabled,
            tcpPort: status.tcpPort,
            udpPort: status.udpPort,
            peerCount: status.peerCount,
            lastError: status.lastError
        )
    }

    /// The protocol of a kind of the bridge.
    ///
    /// WHY: The enumeration of the bridge and the one of the shell have the
    /// same members. The function is written anyway, so that a new member
    /// of the bridge cannot reach the screens unnoticed: the shell would
    /// leave the card out instead of drawing a protocol that it cannot
    /// name.
    public static func protocolId(of kind: FfiProtocolKind) -> ProtocolId? {
        switch kind {
        case .airPlay:
            return .airplay
        case .miracast:
            return .miracast
        case .cast:
            return .cast
        case .dlna:
            return .dlna
        // WHY: the shell names every protocol of the bridge. A member that
        // is missing here would leave its card out of the grid.
        case .smartView:
            return .smartview
        }
    }

    /// Whether a protocol is switched on in the configuration.
    ///
    /// WHY: The core treats a protocol that is not mentioned in the
    /// configuration as switched on, because the default of the protocol is
    /// "enabled". The shell has to read the configuration the same way, or
    /// the switch of the card and the behaviour of the engine would
    /// disagree.
    public static func isProtocolEnabled(_ config: FfiAppConfig, bridgeName: String) -> Bool {
        config.protocols.first { $0.name == bridgeName }?.enabled ?? true
    }

    /// The configuration in which one protocol is switched on or off.
    ///
    /// WHY: The configuration is the only place where the user can change a
    /// protocol without touching the source. A protocol that is not
    /// mentioned in it yet is added with the default of the core: no
    /// pinned port, no PIN, and a limit of one megabyte for a single
    /// message.
    public static func withProtocolEnabled(
        _ config: FfiAppConfig,
        bridgeName: String,
        enabled: Bool
    ) -> FfiAppConfig {
        let existing = config.protocols.first { $0.name == bridgeName }
        let entry = existing ?? FfiProtocolConfig(
            name: bridgeName,
            enabled: enabled,
            port: 0,
            requirePin: false,
            maxMessageBytes: ReceiverDefaults.defaultMaxMessageBytes
        )
        var updated = entry
        updated.enabled = enabled
        var protocols = config.protocols.filter { $0.name != bridgeName }
        protocols.append(updated)
        var result = config
        result.protocols = protocols
        return result
    }

    /// The session that is running, as the shell shows it.
    ///
    /// - Parameter session: the snapshot of the session.
    /// - Returns: the state of the running session.
    public static func nowPlaying(of session: FfiSessionSnapshot) -> NowPlayingState {
        NowPlayingState(
            sessionId: session.id,
            protocolName: protocolId(of: session.protocol)?.rawValue ?? String(describing: session.protocol),
            senderName: session.senderName,
            senderAddress: session.senderAddress,
            video: videoDescription(session.video),
            audio: audioDescription(session.audio),
            title: nil,
            artist: nil,
            album: nil,
            artwork: nil,
            uptimeMs: session.uptimeMs
        )
    }

    /// The description of a video stream, if the session carries one.
    ///
    /// WHY: The name of a codec and the size of a picture are the names of
    /// the standards, and they are the same in every language. Only the
    /// sentence around them belongs to the resources of the shell.
    public static func videoDescription(_ format: FfiVideoFormat?) -> String? {
        guard let format else {
            return nil
        }
        let frameRate = format.frameRate.map { " @\($0)" } ?? ""
        return "\(videoCodecName(format.codec)) \(format.width)x\(format.height)\(frameRate)"
    }

    /// The description of an audio stream, if the session carries one.
    public static func audioDescription(_ format: FfiAudioFormat?) -> String? {
        guard let format else {
            return nil
        }
        let channels = format.channels == 1 ? "mono" : "stereo"
        return "\(audioCodecName(format.codec)) \(format.sampleRate) Hz \(channels)"
    }

    /// The name of a video codec.
    public static func videoCodecName(_ codec: FfiVideoCodec) -> String {
        switch codec {
        case .h264:
            return "H.264"
        case .hevc:
            return "HEVC"
        case .vp8:
            return "VP8"
        case .vp9:
            return "VP9"
        case .av1:
            return "AV1"
        }
    }

    /// The name of an audio codec.
    public static func audioCodecName(_ codec: FfiAudioCodec) -> String {
        switch codec {
        case .aacLc:
            return "AAC-LC"
        case .aacEld:
            return "AAC-ELD"
        case .alac:
            return "ALAC"
        case .lpcm:
            return "LPCM"
        case .opus:
            return "Opus"
        case .mp3:
            return "MP3"
        case .flac:
            return "FLAC"
        }
    }

    /// The formatting of the running time of a session.
    ///
    /// WHY: The shell shows the age of a session, and the age is the only
    /// information about a session that the core keeps in a form that
    /// survives a change of the clock of the system. The format is a plain
    /// number and no word, so that the text of the shell does not have to
    /// know the language of the user.
    public static func uptimeText(_ milliseconds: UInt64) -> String {
        let totalSeconds = milliseconds / 1_000
        let seconds = totalSeconds % 60
        let minutes = (totalSeconds / 60) % 60
        let hours = totalSeconds / 3_600
        if hours > 0 {
            return "\(hours):\(padded(minutes)):\(padded(seconds))"
        }
        return "\(minutes):\(padded(seconds))"
    }

    /// A number with a leading zero, if it is smaller than ten.
    private static func padded(_ value: UInt64) -> String {
        value < 10 ? "0\(value)" : "\(value)"
    }
}
