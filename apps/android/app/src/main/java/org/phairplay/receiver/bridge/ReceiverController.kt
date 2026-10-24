package org.phairplay.receiver.bridge

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.phairplay.receiver.model.BridgeFailure
import org.phairplay.receiver.model.ProtocolCardState
import org.phairplay.receiver.model.ProtocolId
import org.phairplay.receiver.model.ReceiverUiState
import org.phairplay.receiver.model.isProtocolEnabled
import org.phairplay.receiver.model.nowPlayingOf
import org.phairplay.receiver.model.protocolCard
import org.phairplay.receiver.model.protocolIdOf
import org.phairplay.receiver.model.withProtocolEnabled
import uniffi.phairplay_ffi.FfiAppConfig
import uniffi.phairplay_ffi.FfiException
import uniffi.phairplay_ffi.FfiProtocolStatus
import uniffi.phairplay_ffi.FfiReceiverEvent
import uniffi.phairplay_ffi.Receiver
import uniffi.phairplay_ffi.ReceiverEventSink

/**
 * The controller of the receiver.
 *
 * WHAT: the one object of the shell that talks to the bridge. It owns the
 * receiver, it reads the status of the protocols, it starts and stops the
 * engines, it writes the configuration, and it turns the events of the
 * bridge into the state of the interface.
 *
 * WHY: Every screen of the shell and the notification of the service read
 * the same state, and this controller is the only place where that state
 * is produced. It is also the only place where a failure of the bridge is
 * caught: the bridge reports its failures with an exception that carries a
 * code, and the controller turns it into a [BridgeFailure] that the screens
 * can show. A failure must never reach a screen as a crash.
 *
 * EXAMPLE: the dashboard collects `controller.state` and draws one card per
 * protocol; the switch of a card calls `setProtocolEnabled`.
 */
class ReceiverController(
    private val scope: CoroutineScope,
    private val receiver: Receiver,
) : ReceiverEventSink {

    /** The lock that guards the state. */
    private val mutex = Mutex()

    /** The metadata of the sessions, by the identifier of the session. */
    private val metadata = mutableMapOf<ULong, SessionMetadata>()

    /** The state that the screens read. */
    private val mutableState = MutableStateFlow(ReceiverUiState.EMPTY)

    /** The state that the screens read. */
    val state: StateFlow<ReceiverUiState> = mutableState.asStateFlow()

    init {
        receiver.setEventSink(this)
        refresh()
    }

    /**
     * Reads the state of the receiver from the bridge.
     *
     * WHY: The events of the bridge do not carry everything: the identity
     * of the device and the version of the build arrive only once, and the
     * list of the protocols has to be built from the status of the four
     * engines. The screens therefore ask the bridge for the truth whenever
     * they are shown.
     */
    fun refresh() {
        scope.launch {
            mutex.withLock { refreshLocked() }
        }
    }

    /**
     * Switches one protocol on or off.
     *
     * WHY: The configuration of the bridge is the only place where the user
     * can change a protocol. Switching a protocol on writes the
     * configuration first and starts the engine afterwards, because the
     * bridge refuses to start a protocol that is switched off. Switching one
     * off stops the engine first, so that the sender is disconnected before
     * the service disappears.
     */
    fun setProtocolEnabled(protocol: ProtocolId, enabled: Boolean) {
        scope.launch {
            mutex.withLock {
                if (enabled) {
                    val config = readConfig() ?: return@withLock
                    val written = call {
                        receiver.setConfig(config.withProtocolEnabled(protocol.bridgeName, true))
                    }
                    if (written == null) {
                        // The failure was already reported by the call.
                        return@withLock
                    }
                    call { receiver.startProtocol(protocol.bridgeName) }
                } else {
                    call { receiver.stopProtocol(protocol.bridgeName) }
                    val config = readConfig() ?: return@withLock
                    val written = call {
                        receiver.setConfig(config.withProtocolEnabled(protocol.bridgeName, false))
                    }
                    if (written == null) {
                        return@withLock
                    }
                }
                refreshLocked()
            }
        }
    }

    /** Removes the failure that the interface shows. */
    fun dismissFailure() {
        scope.launch {
            mutex.withLock {
                mutableState.value = mutableState.value.copy(failure = null)
            }
        }
    }

    /**
     * Closes the receiver and gives the resources back.
     *
     * WHY: The receiver owns a runtime of its own and the engines that
     * listen on sockets. The service releases them when it stops, so that a
     * restart of the service does not run into a port that is still bound.
     */
    fun close() {
        receiver.clearEventSink()
        receiver.close()
    }

    /**
     * The call of the bridge that delivers the events.
     *
     * WHY: The call happens on a thread of the bridge, not on the thread of
     * the shell. The event is therefore handed over to the scope of the
     * controller, and the state is only ever written there.
     */
    override fun onEvent(event: FfiReceiverEvent) {
        scope.launch {
            mutex.withLock { applyEvent(event) }
        }
    }

    /**
     * Reads the whole state from the bridge.
     *
     * WHY: The method assumes that the lock is held. Every caller of it is
     * a coroutine of the scope of the controller.
     */
    private fun refreshLocked() {
        val status = call { receiver.status() }
        val config = readConfig()
        val sessions = call { receiver.sessions() }
        val identity = call { receiver.identity() }
        val build = call { receiver.buildInfo() }
        if (status == null || config == null || sessions == null || identity == null || build == null) {
            return
        }

        val cards = status.mapNotNull { entry -> cardOf(entry, config) }
        val running = cards.any { it.phase.isRunning }
        val nowPlaying = sessions.firstOrNull()?.let { session ->
            val base = nowPlayingOf(session)
            val extra = metadata[base.sessionId]
            base.copy(
                title = extra?.title,
                artist = extra?.artist,
                album = extra?.album,
                artwork = extra?.artwork,
            )
        }
        mutableState.value = mutableState.value.copy(
            running = running,
            deviceName = identity.name,
            version = build.version,
            protocols = cards,
            nowPlaying = nowPlaying,
        )
    }

    /** The card of one protocol. */
    private fun cardOf(status: FfiProtocolStatus, config: FfiAppConfig): ProtocolCardState? {
        val name = protocolIdOf(status.protocol)?.bridgeName ?: status.protocol.name
        val enabled = config.isProtocolEnabled(name)
        return protocolCard(
            kind = status.protocol,
            enabled = enabled,
            state = status.state,
            tcpPort = status.tcpPort,
            udpPort = status.udpPort,
            peerCount = status.peerCount,
            lastError = status.lastError,
        )
    }

    /** The configuration of the receiver. */
    private fun readConfig(): FfiAppConfig? = call { receiver.config() }

    /**
     * Applies one event of the bridge to the state.
     *
     * WHY: Every event leads to a new reading of the state. The bridge is
     * the only source of the truth, and reading it again is cheaper than
     * rebuilding the whole state out of the event — and it cannot drift
     * away from the bridge.
     */
    private fun applyEvent(event: FfiReceiverEvent) {
        when (event) {
            is FfiReceiverEvent.SessionEnded -> metadata.remove(event.sessionId)
            is FfiReceiverEvent.MetadataChanged -> {
                val known = metadata[event.sessionId] ?: SessionMetadata()
                metadata[event.sessionId] = known.copy(
                    title = event.title ?: known.title,
                    artist = event.artist ?: known.artist,
                    album = event.album ?: known.album,
                )
            }
            is FfiReceiverEvent.ArtworkChanged -> {
                val known = metadata[event.sessionId] ?: SessionMetadata()
                metadata[event.sessionId] = known.copy(artwork = event.artwork ?: known.artwork)
            }
            else -> Unit
        }
        refreshLocked()
    }

    /**
     * Calls the bridge and turns a failure into a report.
     *
     * WHY: The bridge reports every failure with an exception that carries
     * the code of the core. The shell must not let it escape: a screen that
     * throws is a screen that crashes the application.
     */
    private fun <T> call(block: () -> T): T? {
        return try {
            block()
        } catch (failure: FfiException) {
            report(failure.toBridgeFailure())
            null
        } catch (failure: IllegalStateException) {
            // The receiver was closed while the call was running.
            report(
                BridgeFailure(
                    code = INTERNAL_CODE,
                    details = failure.message ?: "the receiver is closed",
                ),
            )
            null
        }
    }

    /** Shows a failure of the bridge in the interface. */
    private fun report(failure: BridgeFailure) {
        mutableState.value = mutableState.value.copy(failure = failure)
    }

    /** The metadata of one session. */
    private data class SessionMetadata(
        val title: String? = null,
        val artist: String? = null,
        val album: String? = null,
        val artwork: ByteArray? = null,
    ) {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is SessionMetadata) return false
            return title == other.title &&
                artist == other.artist &&
                album == other.album &&
                artwork.contentEquals(other.artwork)
        }

        override fun hashCode(): Int {
            var result = title?.hashCode() ?: 0
            result = 31 * result + (artist?.hashCode() ?: 0)
            result = 31 * result + (album?.hashCode() ?: 0)
            result = 31 * result + (artwork?.contentHashCode() ?: 0)
            return result
        }
    }

    companion object {
        /**
         * The code that the shell uses when the bridge fails without a code
         * of its own.
         *
         * WHY: The number is the code of the core for an unspecified
         * internal failure. The shell never invents a code of its own.
         */
        const val INTERNAL_CODE: Int = 9
    }
}
