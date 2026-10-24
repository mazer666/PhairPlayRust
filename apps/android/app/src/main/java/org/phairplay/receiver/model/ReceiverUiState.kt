package org.phairplay.receiver.model

/**
 * The state of the receiver, as the dashboard of the shell shows it.
 *
 * WHAT: the four protocol cards, the session that is running, the identity
 * of the device and the last failure of the bridge.
 *
 * WHY: The shell has one state and one flow of changes. The service, the
 * notification and the screens all read the same state, so they cannot
 * disagree about what the receiver is doing.
 *
 * EXAMPLE: the dashboard draws one card per entry of [protocols] and the
 * section of the running session from [nowPlaying].
 */
data class ReceiverUiState(
    /** Whether the receiver of the shell is running. */
    val running: Boolean,
    /** The name of the device that the receiver advertises. */
    val deviceName: String,
    /** The version of the build that carries the bridge. */
    val version: String,
    /** The card of every protocol, in the order of the core. */
    val protocols: List<ProtocolCardState>,
    /** The session that is running, if there is one. */
    val nowPlaying: NowPlayingState?,
    /** The last failure of the bridge, if there was one. */
    val failure: BridgeFailure?,
) {
    companion object {
        /**
         * The state before the bridge has answered for the first time.
         *
         * WHY: The screen must have something to draw before the first
         * answer arrives, and it must say that it does not know yet
         * instead of showing an empty receiver as if it were the truth.
         */
        val EMPTY: ReceiverUiState = ReceiverUiState(
            running = false,
            deviceName = "",
            version = "",
            protocols = emptyList(),
            nowPlaying = null,
            failure = null,
        )
    }
}

/**
 * The cards of the protocols in the order of the grid of the dashboard.
 *
 * WHY: The order of the cards is the order in which the user meets them,
 * and it must not depend on the order in which the core happens to
 * answer. The grid of the dashboard therefore always shows AirPlay,
 * Miracast, Cast and DLNA in this order, and the focus of the remote
 * control walks through the grid in the same order.
 *
 * EXAMPLE: `orderedProtocols(cards)` puts the card of AirPlay in the
 * upper left corner of the grid.
 */
fun orderedProtocols(cards: List<ProtocolCardState>): List<ProtocolCardState> =
    cards.sortedBy { it.protocol.ordinal }
