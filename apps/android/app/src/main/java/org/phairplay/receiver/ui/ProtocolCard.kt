package org.phairplay.receiver.ui

import android.view.KeyEvent as AndroidKeyEvent
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R
import org.phairplay.receiver.model.ProtocolCardState
import org.phairplay.receiver.model.ProtocolId

/**
 * The card of one protocol.
 *
 * WHAT: the name of the protocol, the state of its engine as a big label
 * with a sign of its own, the ports it listens on, the number of senders
 * that are connected and the switch that shows whether the protocol is
 * switched on.
 *
 * WHY: The card is the button of the dashboard. A user with a remote
 * control cannot hit a switch of forty-eight dp in the corner of a card,
 * so the whole card is the thing that is focused, and the key "OK" on the
 * card switches the protocol. The switch itself stays in the card as the
 * visible state of the protocol — but it is a picture, not a target of the
 * focus.
 *
 * The card therefore knows three things at once: what the protocol is
 * called, in which state it is, and whether it is switched on. The state
 * is told by a word, by a sign and by a colour, so that it is readable
 * with a colour weakness as well.
 *
 * The card measures itself. A card on a big television shows the name, the
 * state, the sentence that explains the state and the ports; a card on a
 * small television shows the name and the state — with a type that is
 * still 22 sp and 20 sp — and leaves the rest out. The type never falls
 * below the size that can be read from three metres, and the grid of the
 * protocols never has to be scrolled.
 *
 * EXAMPLE: `ProtocolCard(card, focusRequester) { protocol, enabled -> ... }`
 * draws the card of AirPlay and reports every change to the bridge.
 */
@Composable
fun ProtocolCard(
    card: ProtocolCardState,
    focusRequester: FocusRequester,
    onToggle: (ProtocolId, Boolean) -> Unit,
    modifier: Modifier = Modifier,
    onFocusChanged: (Boolean) -> Unit = {},
) {
    var focused by remember { mutableStateOf(false) }
    val shape = RoundedCornerShape(20.dp)
    val protocolName = card.protocol.label()
    val statusText = card.phase.label()
    val actionLabel = if (card.enabled) {
        stringResource(R.string.protocol_switch_off, protocolName)
    } else {
        stringResource(R.string.protocol_switch_on, protocolName)
    }
    val switchLabel = if (card.enabled) {
        stringResource(R.string.protocol_switched_on)
    } else {
        stringResource(R.string.protocol_switched_off)
    }
    val description = stringResource(
        R.string.protocol_card_description,
        protocolName,
        statusText,
    )

    Card(
        modifier = modifier
            .focusRequester(focusRequester)
            .onFocusChanged {
                focused = it.isFocused
                onFocusChanged(it.isFocused)
            }
            .focusable()
            .onKeyEvent { event -> onCardKey(event) { onToggle(card.protocol, !card.enabled) } }
            .pointerInput(card.protocol, card.enabled) {
                detectTapGestures {
                    focusRequester.requestFocus()
                    onToggle(card.protocol, !card.enabled)
                }
            }
            .semantics {
                role = Role.Switch
                contentDescription = description
                stateDescription = switchLabel
                onClick(label = actionLabel) {
                    onToggle(card.protocol, !card.enabled)
                    true
                }
            }
            .tvFocusFrame(focused = focused, shape = shape),
        shape = shape,
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = if (focused) 6.dp else 2.dp),
    ) {
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxSize()
                .tvFocusVeil(focused),
        ) {
            // A card of a small television has no room for the sentence
            // that explains the state. The sentence is the first thing
            // that goes, the ports are the last.
            //
            // The three tiers of the card: the tight card needs 68 dp, the
            // middle card 124 dp and the roomy card 165 dp. The card
            // measures itself and takes the tier that fits — the type of
            // the name and of the state never falls below 22 sp and
            // 20 sp, so that the state stays readable from three metres.
            val tight = maxHeight < 124.dp
            val roomy = maxHeight >= 165.dp
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(if (tight) 8.dp else 16.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = protocolName,
                        style = if (tight) {
                            MaterialTheme.typography.titleMedium
                        } else {
                            MaterialTheme.typography.titleLarge
                        },
                        color = PhairPlayColors.Body,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                    Spacer(modifier = Modifier.width(12.dp))
                    SwitchIndicator(enabled = card.enabled, tight = tight)
                }

                Spacer(modifier = Modifier.height(if (tight) 4.dp else 10.dp))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    StatusGlyph(
                        phase = card.phase,
                        color = card.phase.tone(),
                        diameter = if (tight) 22.dp else 28.dp,
                    )
                    Spacer(modifier = Modifier.width(10.dp))
                    Text(
                        text = statusText,
                        style = if (tight) {
                            MaterialTheme.typography.titleSmall
                        } else {
                            MaterialTheme.typography.titleMedium
                        },
                        color = card.phase.tone(),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }

                Spacer(modifier = Modifier.height(6.dp))

                if (roomy) {
                    Text(
                        text = card.phase.reason(),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                } else {
                    Spacer(modifier = Modifier.weight(1f))
                }

                if (!tight) {
                    Text(
                        text = detailsOf(card),
                        style = MaterialTheme.typography.bodySmall,
                        color = if (card.lastError.isNullOrBlank()) {
                            PhairPlayColors.Muted
                        } else {
                            PhairPlayColors.Failure
                        },
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

/**
 * The switch of the protocol, drawn as a picture.
 *
 * WHY: The card is the thing that is focused, and the switch is the state
 * that the card shows. The switch therefore has no semantics of its own —
 * the card announces "on" or "off" — and it cannot be reached with the
 * focus.
 */
@Composable
private fun SwitchIndicator(
    enabled: Boolean,
    tight: Boolean,
    modifier: Modifier = Modifier,
) {
    val label = if (enabled) {
        stringResource(R.string.protocol_switched_on)
    } else {
        stringResource(R.string.protocol_switched_off)
    }
    Row(
        modifier = modifier.clearAndSetSemantics { },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(width = if (tight) 34.dp else 46.dp, height = if (tight) 20.dp else 26.dp)
                .clip(RoundedCornerShape(13.dp))
                .background(
                    if (enabled) PhairPlayColors.Accent else PhairPlayColors.SurfaceRaised,
                ),
        ) {
            Box(
                modifier = Modifier
                    .padding(3.dp)
                    .size(if (tight) 14.dp else 20.dp)
                    .align(if (enabled) Alignment.CenterEnd else Alignment.CenterStart)
                    .clip(CircleShape)
                    .background(PhairPlayColors.Body),
            )
        }
        if (!tight) {
            Spacer(modifier = Modifier.width(8.dp))
        }
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = PhairPlayColors.Muted,
        )
    }
}

/**
 * The keys that switch a protocol on and off.
 *
 * WHY: The key "OK" of a remote control arrives as `KEYCODE_DPAD_CENTER`,
 * a keyboard arrives as `Enter` or as the space bar, and a switch of a
 * pair of glasses arrives as `KEYCODE_ENTER`. All of them mean the same
 * thing here: the card is the button.
 */
private fun onCardKey(event: KeyEvent, activate: () -> Unit): Boolean {
    if (event.type != KeyEventType.KeyDown) {
        return false
    }
    // A key that is held down must not switch the protocol again and
    // again.
    if ((event.nativeKeyEvent?.repeatCount ?: 0) > 0) {
        return false
    }
    val isActivation = when (event.key) {
        Key.Enter, Key.NumPadEnter, Key.Spacebar, Key.DirectionCenter -> true
        else -> when (event.nativeKeyEvent?.keyCode) {
            AndroidKeyEvent.KEYCODE_ENTER,
            AndroidKeyEvent.KEYCODE_DPAD_CENTER,
            AndroidKeyEvent.KEYCODE_NUMPAD_ENTER,
            -> true
            else -> false
        }
    }
    if (!isActivation) {
        return false
    }
    activate()
    return true
}

/**
 * The sentence about the ports and the senders of the protocol.
 */
@Composable
private fun detailsOf(card: ProtocolCardState): String {
    val failure = card.lastError
    if (!failure.isNullOrBlank()) {
        return failure
    }
    val ports = when {
        card.tcpPort != null && card.udpPort != null ->
            stringResource(R.string.protocol_card_ports, card.tcpPort.toString(), card.udpPort.toString())
        card.tcpPort != null ->
            stringResource(R.string.protocol_card_ports_tcp_only, card.tcpPort.toString())
        card.udpPort != null ->
            stringResource(R.string.protocol_card_ports, "—", card.udpPort.toString())
        else -> stringResource(R.string.protocol_card_no_port)
    }
    if (card.peerCount > 0u) {
        val peers = pluralStringResource(
            R.plurals.protocol_card_peers,
            card.peerCount.toInt(),
            card.peerCount.toInt(),
        )
        return "$ports · $peers"
    }
    return ports
}
