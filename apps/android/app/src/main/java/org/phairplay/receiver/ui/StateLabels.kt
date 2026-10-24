package org.phairplay.receiver.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import org.phairplay.receiver.R
import org.phairplay.receiver.model.ProtocolId
import org.phairplay.receiver.model.ProtocolPhase

/**
 * The wording of the states of a protocol.
 *
 * WHAT: the text and the colour of every phase of a protocol card.
 *
 * WHY: The state of the engine is a fact of the core, and the wording of
 * it is a fact of the shell. The two are kept apart here, so that the
 * card does not have to know where a state comes from — and so that every
 * platform of the product can say the same thing in the same way.
 *
 * EXAMPLE: the phase `DISABLED` reads "Disabled in the settings" and
 * explains that the service is not advertised at all; the phase `STOPPED`
 * reads "Service stopped" and explains that the protocol is switched on
 * but the service is not running.
 */
@Composable
fun ProtocolPhase.label(): String = stringResource(
    when (this) {
        ProtocolPhase.DISABLED -> R.string.protocol_state_disabled
        ProtocolPhase.NOT_STARTED -> R.string.protocol_state_not_started
        ProtocolPhase.STOPPED -> R.string.protocol_state_stopped
        ProtocolPhase.STARTING -> R.string.protocol_state_starting
        ProtocolPhase.ADVERTISING -> R.string.protocol_state_advertising
        ProtocolPhase.STREAMING -> R.string.protocol_state_streaming
        ProtocolPhase.STOPPING -> R.string.protocol_state_stopping
        ProtocolPhase.FAILED -> R.string.protocol_state_failed
    },
)

/**
 * The sentence that explains the phase of a protocol.
 */
@Composable
fun ProtocolPhase.reason(): String = stringResource(
    when (this) {
        ProtocolPhase.DISABLED -> R.string.protocol_state_disabled_reason
        ProtocolPhase.NOT_STARTED -> R.string.protocol_state_not_started_reason
        ProtocolPhase.STOPPED -> R.string.protocol_state_stopped_reason
        ProtocolPhase.STARTING -> R.string.protocol_state_starting_reason
        ProtocolPhase.ADVERTISING -> R.string.protocol_state_advertising_reason
        ProtocolPhase.STREAMING -> R.string.protocol_state_streaming_reason
        ProtocolPhase.STOPPING -> R.string.protocol_state_stopping_reason
        ProtocolPhase.FAILED -> R.string.protocol_state_failed_reason
    },
)

/**
 * The name of a protocol that the user reads.
 *
 * WHY: The names of the products are the names of the senders, and they
 * are the same in every language. The shell nevertheless keeps them in the
 * resources, so that a translation can adapt them.
 */
@Composable
fun ProtocolId.label(): String = stringResource(
    when (this) {
        ProtocolId.AIRPLAY -> R.string.protocol_airplay
        ProtocolId.MIRACAST -> R.string.protocol_miracast
        ProtocolId.CAST -> R.string.protocol_cast
        ProtocolId.DLNA -> R.string.protocol_dlna
    },
)

/**
 * The colour of the phase of a protocol.
 *
 * WHY: The colour is the third sign of the state — the word and the shape
 * come first, so that the state is also readable without a colour. Every
 * colour meets the contrast of 4.5:1 against the surface of the card.
 */
@Composable
fun ProtocolPhase.tone(): Color = when (this) {
    ProtocolPhase.DISABLED -> PhairPlayColors.Idle
    ProtocolPhase.NOT_STARTED -> PhairPlayColors.Idle
    ProtocolPhase.STOPPED -> PhairPlayColors.Idle
    ProtocolPhase.STARTING -> PhairPlayColors.Transition
    ProtocolPhase.ADVERTISING -> PhairPlayColors.Advertising
    ProtocolPhase.STREAMING -> PhairPlayColors.Streaming
    ProtocolPhase.STOPPING -> PhairPlayColors.Transition
    ProtocolPhase.FAILED -> PhairPlayColors.Failure
}
