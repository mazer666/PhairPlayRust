import Foundation

/// The state of one protocol as the shell shows it.
///
/// WHAT: the eight states a protocol card can carry. They are the seven
/// states of the engine of the core plus the case that the user has
/// switched the protocol off.
///
/// WHY: "switched off in the settings" and "the service is not running"
/// are two different facts, and a user must never have to guess which one
/// they are looking at. A protocol that is switched off is not advertised
/// at all; a protocol that is switched on but not running was stopped.
/// The core does not know about the first case — it only knows the states
/// of the engine — so the shell adds it here.
///
/// EXAMPLE: a card whose phase is ``ProtocolPhase/disabled`` shows
/// "Disabled in the settings", a card whose phase is ``ProtocolPhase/stopped``
/// shows "Service stopped".
public enum ProtocolPhase: String, Sendable, CaseIterable {
    /// The protocol is switched off in the settings; it is not advertised.
    case disabled

    /// The protocol is switched on, but the engine has never run.
    case notStarted

    /// The protocol is switched on, but the service is not running.
    case stopped

    /// The engine is starting up.
    case starting

    /// The engine is running and advertising itself.
    case advertising

    /// A sender is connected and the stream is running.
    case streaming

    /// The engine is shutting down.
    case stopping

    /// The engine failed to start or died unexpectedly.
    case failed

    /// Whether the receiver of the protocol is working right now.
    ///
    /// WHY: The head of the window says whether the receiver is running.
    /// Only the states in which the engine serves the network count: a
    /// protocol that is starting up is on its way, and one that is stopping
    /// is on its way out.
    public var isRunning: Bool {
        switch self {
        case .starting, .advertising, .streaming, .stopping:
            return true
        case .disabled, .notStarted, .stopped, .failed:
            return false
        }
    }
}

/// The four protocols the receiver speaks.
///
/// WHAT: the identifier of a protocol and the name the bridge knows it by.
///
/// WHY: The bridge speaks the names of the core: "airplay", "miracast",
/// "cast" and "dlna". The shell needs those names to start and to stop an
/// engine, and they have to be exactly the names of the core — a different
/// spelling would be answered with the structured failure "unknown
/// protocol". The table below is therefore the contract between the shell
/// and the bridge, and it is written down in one place.
///
/// EXAMPLE: ``ProtocolId/airplay`` is "airplay", and the shell calls
/// `startProtocol(protocol: "airplay")` with it.
public enum ProtocolId: String, Sendable, CaseIterable {
    /// AirPlay 2 — screen mirroring and audio streaming.
    case airplay = "airplay"

    /// Miracast / Wi-Fi Display — screen mirroring.
    case miracast = "miracast"

    /// Google Cast — media playback and mirroring.
    case cast = "cast"

    /// DLNA / UPnP MediaRenderer — media playback.
    case dlna = "dlna"

    /// The protocol with the given name, if the name is a protocol.
    ///
    /// WHY: The name arrives from the configuration of the core, and a name
    /// that the shell does not know must not crash the shell. The call
    /// returns `nil` and the caller leaves the entry alone.
    public static func ofBridgeName(_ name: String) -> ProtocolId? {
        ProtocolId(rawValue: name)
    }
}

/// The state of one protocol, as the card of the dashboard shows it.
///
/// WHAT: the protocol, the phase, the ports the engine listens on and the
/// number of senders that are connected.
///
/// WHY: The card of the dashboard must show the same information on every
/// platform and must never invent a fact. Every field of this structure
/// is a fact that the bridge reported; the shell only chooses the wording
/// of the phase.
///
/// EXAMPLE: a card of a running AirPlay engine has the phase
/// ``ProtocolPhase/advertising`` and the port `7000`.
public struct ProtocolCardState: Sendable, Equatable {
    /// The protocol the card describes.
    public let protocolId: ProtocolId

    /// The phase the card shows.
    public let phase: ProtocolPhase

    /// Whether the protocol is switched on in the settings.
    public let enabled: Bool

    /// The TCP port the engine listens on, if it listens on TCP.
    public let tcpPort: UInt16?

    /// The UDP port the engine listens on, if it listens on UDP.
    public let udpPort: UInt16?

    /// The number of senders that are connected.
    public let peerCount: UInt32

    /// The reason of the last failure, if there was one.
    public let lastError: String?

    /// Creates the card of one protocol.
    public init(
        protocolId: ProtocolId,
        phase: ProtocolPhase,
        enabled: Bool,
        tcpPort: UInt16?,
        udpPort: UInt16?,
        peerCount: UInt32,
        lastError: String?
    ) {
        self.protocolId = protocolId
        self.phase = phase
        self.enabled = enabled
        self.tcpPort = tcpPort
        self.udpPort = udpPort
        self.peerCount = peerCount
        self.lastError = lastError
    }
}

/// The session that is running right now, as the shell shows it.
///
/// WHAT: the sender, the formats of the stream, the metadata of the track
/// and the artwork that the sender sent.
///
/// WHY: The metadata of a track does not arrive with the snapshot of a
/// session — it arrives later, as an event. The shell therefore keeps the
/// snapshot and the metadata of a session together, and the view of the
/// running session always shows the newest of both.
///
/// EXAMPLE: a session of an AirPlay sender that plays a track carries the
/// title of the track, the artist and the album.
public struct NowPlayingState: Sendable, Equatable {
    /// The identifier of the session.
    public let sessionId: UInt64

    /// The stable name of the protocol that carries the session.
    public let protocolName: String

    /// The name of the sender, if it announced one.
    public let senderName: String?

    /// The address of the sender, if it is known.
    public let senderAddress: String?

    /// The description of the video stream, if the session carries one.
    public let video: String?

    /// The description of the audio stream, if the session carries one.
    public let audio: String?

    /// The title of the track, if the sender reported one.
    public let title: String?

    /// The artist of the track, if the sender reported one.
    public let artist: String?

    /// The album of the track, if the sender reported one.
    public let album: String?

    /// The artwork of the track, if the sender sent one.
    public let artwork: Data?

    /// How long the session has been running, in milliseconds.
    public let uptimeMs: UInt64

    /// Creates the state of the running session.
    public init(
        sessionId: UInt64,
        protocolName: String,
        senderName: String?,
        senderAddress: String?,
        video: String?,
        audio: String?,
        title: String?,
        artist: String?,
        album: String?,
        artwork: Data?,
        uptimeMs: UInt64
    ) {
        self.sessionId = sessionId
        self.protocolName = protocolName
        self.senderName = senderName
        self.senderAddress = senderAddress
        self.video = video
        self.audio = audio
        self.title = title
        self.artist = artist
        self.album = album
        self.artwork = artwork
        self.uptimeMs = uptimeMs
    }
}

/// A failure that crossed the bridge.
///
/// WHAT: the stable numeric code of the failure of the core and the
/// explanation that came with it.
///
/// WHY: The bridge never answers with a bare string and never panics. The
/// shell therefore shows the code and the explanation of the failure
/// instead of a generic "something went wrong", because the code is the
/// part that stays the same when the wording of the core changes.
///
/// EXAMPLE: a call with the name of a protocol that does not exist arrives
/// with the code `1` and the explanation "unknown protocol `telepathy`".
public struct BridgeFailure: Sendable, Equatable {
    /// The stable numeric code of the failure.
    public let code: Int32

    /// The explanation of the failure.
    public let details: String

    /// Creates the failure of the bridge.
    public init(code: Int32, details: String) {
        self.code = code
        self.details = details
    }
}

/// The state of the receiver, as the window of the shell shows it.
///
/// WHAT: the four protocol cards, the session that is running, the identity
/// of the device and the last failure of the bridge.
///
/// WHY: The shell has one state and one flow of changes. The window, the
/// item in the menu bar and the tests all read the same state, so they
/// cannot disagree about what the receiver is doing.
///
/// EXAMPLE: the window draws one card per entry of ``protocols`` and the
/// view of the running session from ``nowPlaying``.
public struct ReceiverUIState: Sendable, Equatable {
    /// Whether the receiver of the shell is running.
    public var running: Bool

    /// The name of the device that the receiver advertises.
    public var deviceName: String

    /// The version of the build that carries the bridge.
    public var version: String

    /// The card of every protocol, in the order of the core.
    public var protocols: [ProtocolCardState]

    /// The session that is running, if there is one.
    public var nowPlaying: NowPlayingState?

    /// The last failure of the bridge, if there was one.
    public var failure: BridgeFailure?

    /// The state before the bridge has answered for the first time.
    ///
    /// WHY: The window must have something to draw before the first answer
    /// arrives, and it must say that it does not know yet instead of
    /// showing an empty receiver as if it were the truth.
    public static let empty = ReceiverUIState(
        running: false,
        deviceName: "",
        version: "",
        protocols: [],
        nowPlaying: nil,
        failure: nil
    )

    /// Creates the state of the receiver.
    public init(
        running: Bool,
        deviceName: String,
        version: String,
        protocols: [ProtocolCardState],
        nowPlaying: NowPlayingState?,
        failure: BridgeFailure?
    ) {
        self.running = running
        self.deviceName = deviceName
        self.version = version
        self.protocols = protocols
        self.nowPlaying = nowPlaying
        self.failure = failure
    }
}
