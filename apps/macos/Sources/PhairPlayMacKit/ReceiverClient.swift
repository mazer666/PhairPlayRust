import Foundation
import PhairPlayFFI

/// The bridge that hands the events of the receiver to the shell.
///
/// WHY: The callback of the bridge arrives on a thread of the Rust
/// library, not on the thread of the application. The bridge hands the
/// event over to the main queue of the application and changes nothing
/// else, so the unchecked conformance is safe: the closure is called from
/// one place and never from two at the same time.
final class EventBridge: ReceiverEventSink, @unchecked Sendable {
    /// The closure that receives the event.
    private let handler: (FfiReceiverEvent) -> Void

    /// Creates the bridge with the closure that receives the events.
    init(handler: @escaping (FfiReceiverEvent) -> Void) {
        self.handler = handler
    }

    /// The call of the bridge that delivers the events.
    func onEvent(event: FfiReceiverEvent) {
        handler(event)
    }
}

/// The client of the receiver.
///
/// WHAT: the one object of the shell that talks to the bridge. It owns the
/// receiver, it reads the status of the protocols, it starts and stops the
/// engines, it writes the configuration, and it turns the events of the
/// bridge into the state of the window.
///
/// WHY: Every view of the shell and the item in the menu bar read the
/// same state, and this client is the only place where that state is
/// produced. It is also the only place where a failure of the bridge is
/// caught: the bridge reports its failures with an error that carries a
/// code, and the client turns it into a ``BridgeFailure`` that the window
/// can show. A failure must never reach a view as a crash.
///
/// The client knows nothing about the user interface. It announces the
/// changes of the state through ``onChange``, and the shell of the
/// platform decides how they are drawn. That is what makes the client
/// testable without a window.
///
/// EXAMPLE: the window observes the client and draws one card per
/// protocol; the switch of a card calls `setProtocolEnabled`.
public final class ReceiverClient {

    /// The state of the receiver.
    public private(set) var state: ReceiverUIState = .empty

    /// The call that announces a change of the state.
    ///
    /// WHY: The client does not know the user interface, so it cannot
    /// publish the state itself. The call is made on the main queue of the
    /// application, which is the queue the views of the shell live on.
    public var onChange: ((ReceiverUIState) -> Void)?

    /// The receiver of the bridge.
    private var receiver: Receiver?

    /// The metadata of the sessions, by the identifier of the session.
    private var metadata: [UInt64: SessionMetadata] = [:]

    /// Creates the client for a receiver.
    ///
    /// - Parameters:
    ///   - receiver: the receiver of the bridge, or `nil` when the
    ///     receiver could not be created.
    ///   - initialFailure: the failure that the window shows, if the
    ///     receiver could not be created.
    public init(receiver: Receiver?, initialFailure: BridgeFailure? = nil) {
        self.receiver = receiver
        if let initialFailure {
            self.state.failure = initialFailure
        }
        receiver?.setEventSink(sink: EventBridge { [weak self] event in
            DispatchQueue.main.async {
                self?.apply(event)
            }
        })
        refresh()
    }

    /// Reads the state of the receiver from the bridge.
    ///
    /// WHY: The events of the bridge do not carry everything: the identity
    /// of the device and the version of the build arrive only once, and the
    /// list of the protocols has to be built from the status of the four
    /// engines. The window therefore asks the bridge for the truth
    /// whenever it is shown.
    public func refresh() {
        onMain { self.refreshLocked() }
    }

    /// Switches one protocol on or off.
    ///
    /// WHY: The configuration of the bridge is the only place where the
    /// user can change a protocol. Switching a protocol on writes the
    /// configuration first and starts the engine afterwards, because the
    /// bridge refuses to start a protocol that is switched off. Switching
    /// one off stops the engine first, so that the sender is disconnected
    /// before the service disappears.
    ///
    /// - Parameters:
    ///   - protocolId: the protocol that the user switched.
    ///   - enabled: whether the protocol should be switched on.
    public func setProtocolEnabled(_ protocolId: ProtocolId, enabled: Bool) {
        onMain {
            guard let receiver = self.receiver else {
                return
            }
            if enabled {
                guard let config = self.readConfig(receiver) else {
                    return
                }
                let updated = ReceiverMapping.withProtocolEnabled(
                    config,
                    bridgeName: protocolId.rawValue,
                    enabled: true
                )
                do {
                    try receiver.setConfig(config: updated)
                } catch {
                    self.report(error)
                    return
                }
                self.start(receiver, protocolId)
            } else {
                self.stop(receiver, protocolId)
                guard let config = self.readConfig(receiver) else {
                    return
                }
                let updated = ReceiverMapping.withProtocolEnabled(
                    config,
                    bridgeName: protocolId.rawValue,
                    enabled: false
                )
                do {
                    try receiver.setConfig(config: updated)
                } catch {
                    self.report(error)
                    return
                }
            }
            self.refreshLocked()
        }
    }

    /// Removes the failure that the window shows.
    public func dismissFailure() {
        onMain {
            self.state.failure = nil
            self.announce()
        }
    }

    /// Stops the receiver and gives the resources back.
    ///
    /// WHY: The receiver owns a runtime of its own and the engines that
    /// listen on sockets. The application releases them when it quits, so
    /// that a restart does not run into a port that is still bound.
    public func stop() {
        onMain {
            self.receiver?.clearEventSink()
            self.receiver = nil
            self.metadata.removeAll()
            self.state = .empty
            self.announce()
        }
    }

    /// Reads the whole state from the bridge.
    ///
    /// WHY: The method assumes that it runs on the main queue. Every caller
    /// of it is a closure that the queue of the application runs.
    private func refreshLocked() {
        guard let receiver else {
            return
        }
        // WHY: Only the calls that can fail are wrapped in a `do` block.
        // The reading of the state never fails — the bridge answers it
        // with a snapshot — and a `catch` that can never be reached would
        // be a lie about the contract of the bridge.
        let status: [FfiProtocolStatus] = receiver.status()
        let config: FfiAppConfig = receiver.config()
        let sessions: [FfiSessionSnapshot] = receiver.sessions()
        let identity: FfiDeviceIdentity = receiver.identity()
        let build: FfiBuildInfo = receiver.buildInfo()

        let cards: [ProtocolCardState] = status.compactMap { entry in
            let name = ReceiverMapping.protocolId(of: entry.protocol)?.rawValue
                ?? String(describing: entry.protocol)
            let enabled = ReceiverMapping.isProtocolEnabled(config, bridgeName: name)
            return ReceiverMapping.protocolCard(
                kind: entry.protocol,
                enabled: enabled,
                status: entry
            )
        }
        let nowPlaying = sessions.first.map { session in
            let base = ReceiverMapping.nowPlaying(of: session)
            let extra = self.metadata[base.sessionId]
            return NowPlayingState(
                sessionId: base.sessionId,
                protocolName: base.protocolName,
                senderName: base.senderName,
                senderAddress: base.senderAddress,
                video: base.video,
                audio: base.audio,
                title: extra?.title,
                artist: extra?.artist,
                album: extra?.album,
                artwork: extra?.artwork,
                uptimeMs: base.uptimeMs
            )
        }
        state.running = cards.contains { $0.phase.isRunning }
        state.deviceName = identity.name
        state.version = build.version
        state.protocols = cards
        state.nowPlaying = nowPlaying
        announce()
    }

    /// Starts the engine of a protocol.
    private func start(_ receiver: Receiver, _ protocolId: ProtocolId) {
        do {
            _ = try receiver.startProtocol(protocol: protocolId.rawValue)
        } catch {
            report(error)
        }
    }

    /// Stops the engine of a protocol.
    private func stop(_ receiver: Receiver, _ protocolId: ProtocolId) {
        do {
            _ = try receiver.stopProtocol(protocol: protocolId.rawValue)
        } catch {
            report(error)
        }
    }

    /// The configuration of the receiver.
    private func readConfig(_ receiver: Receiver) -> FfiAppConfig? {
        receiver.config()
    }

    /// Applies one event of the bridge to the state.
    ///
    /// WHY: Every event leads to a new reading of the state. The bridge is
    /// the only source of the truth, and reading it again is cheaper than
    /// rebuilding the whole state out of the event — and it cannot drift
    /// away from the bridge.
    private func apply(_ event: FfiReceiverEvent) {
        switch event {
        case .sessionEnded(let sessionId, _):
            metadata.removeValue(forKey: sessionId)
        case .metadataChanged(let sessionId, let title, let artist, let album):
            let known = metadata[sessionId] ?? SessionMetadata()
            metadata[sessionId] = SessionMetadata(
                title: title ?? known.title,
                artist: artist ?? known.artist,
                album: album ?? known.album,
                artwork: known.artwork
            )
        case .artworkChanged(let sessionId, let artwork):
            let known = metadata[sessionId] ?? SessionMetadata()
            metadata[sessionId] = SessionMetadata(
                title: known.title,
                artist: known.artist,
                album: known.album,
                artwork: artwork ?? known.artwork
            )
        default:
            break
        }
        refreshLocked()
    }

    /// Announces the new state of the receiver.
    private func announce() {
        onChange?(state)
    }

    /// Shows a failure of the bridge in the window.
    ///
    /// WHY: The bridge reports every failure with an error that carries the
    /// code of the core. The shell must not let it escape: a view that
    /// throws is a view that crashes the application.
    private func report(_ error: Error) {
        switch error {
        case let failure as FfiError:
            switch failure {
            case .Failed(let code, let details):
                state.failure = BridgeFailure(code: code, details: details)
            }
        default:
            state.failure = BridgeFailure(
                code: ReceiverClient.internalCode,
                details: error.localizedDescription
            )
        }
        announce()
    }

    /// Runs a closure on the main queue of the application.
    private func onMain(_ work: @escaping () -> Void) {
        if Thread.isMainThread {
            work()
        } else {
            DispatchQueue.main.async(execute: work)
        }
    }

    /// The metadata of one session.
    private struct SessionMetadata {
        let title: String?
        let artist: String?
        let album: String?
        let artwork: Data?

        /// Creates the metadata of a session.
        init(
            title: String? = nil,
            artist: String? = nil,
            album: String? = nil,
            artwork: Data? = nil
        ) {
            self.title = title
            self.artist = artist
            self.album = album
            self.artwork = artwork
        }
    }

    /// The code that the shell uses when the bridge fails without a code of
    /// its own.
    ///
    /// WHY: The number is the code of the core for an unspecified internal
    /// failure. The shell never invents a code of its own.
    public static let internalCode: Int32 = 9
}
