package org.phairplay.receiver.model

/**
 * The state of one protocol, as the card of the dashboard shows it.
 *
 * WHAT: the protocol, the phase, the ports the engine listens on and the
 * number of senders that are connected.
 *
 * WHY: The card of the dashboard must show the same information on every
 * platform and must never invent a fact. Every field of this structure is
 * a fact that the bridge reported; the shell only chooses the wording of
 * the phase.
 *
 * EXAMPLE: a card of a running AirPlay engine has the phase
 * [ProtocolPhase.ADVERTISING] and the port `7000`.
 */
data class ProtocolCardState(
    /** The protocol the card describes. */
    val protocol: ProtocolId,
    /** The phase the card shows. */
    val phase: ProtocolPhase,
    /** Whether the protocol is switched on in the settings. */
    val enabled: Boolean,
    /** The TCP port the engine listens on, if it listens on TCP. */
    val tcpPort: UShort?,
    /** The UDP port the engine listens on, if it listens on UDP. */
    val udpPort: UShort?,
    /** The number of senders that are connected. */
    val peerCount: UInt,
    /** The reason of the last failure, if there was one. */
    val lastError: String?,
)
