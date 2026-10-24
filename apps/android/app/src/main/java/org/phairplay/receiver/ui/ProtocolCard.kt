package org.phairplay.receiver.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R
import org.phairplay.receiver.model.ProtocolCardState
import org.phairplay.receiver.model.ProtocolId

/**
 * The card of one protocol.
 *
 * WHAT: the name of the protocol, the state of its engine, the ports it
 * listens on, the number of senders that are connected and the switch that
 * switches the protocol on and off.
 *
 * WHY: A user has to be able to see at a glance which protocols are
 * running and which ones are switched off. The card therefore shows the
 * state as a word and not only as a colour, and the state "switched off in
 * the settings" is a state of its own — it is not the same as "the service
 * is stopped".
 *
 * EXAMPLE: `ProtocolCard(card, onToggle)` draws the card of the AirPlay
 * engine and reports every change of the switch to `onToggle`.
 */
@Composable
fun ProtocolCard(
    card: ProtocolCardState,
    onToggle: (ProtocolId, Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = card.protocol.label(),
                    style = MaterialTheme.typography.titleLarge,
                    color = PhairPlayColors.Body,
                    modifier = Modifier.weight(1f),
                )
                val description = if (card.enabled) {
                    stringResource(R.string.protocol_switch_off, card.protocol.label())
                } else {
                    stringResource(R.string.protocol_switch_on, card.protocol.label())
                }
                Switch(
                    checked = card.enabled,
                    onCheckedChange = { wanted -> onToggle(card.protocol, wanted) },
                    modifier = Modifier.semantics { contentDescription = description },
                )
            }

            Spacer(modifier = Modifier.size(8.dp))

            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier
                        .size(10.dp)
                        .clip(CircleShape)
                        .background(card.phase.tone()),
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = card.phase.label(),
                    style = MaterialTheme.typography.titleMedium,
                    color = card.phase.tone(),
                )
            }

            Spacer(modifier = Modifier.size(4.dp))

            Text(
                text = card.phase.reason(),
                style = MaterialTheme.typography.bodyMedium,
                color = PhairPlayColors.Muted,
            )

            Spacer(modifier = Modifier.size(12.dp))

            Row(
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = portsOf(card),
                    style = MaterialTheme.typography.bodyMedium,
                    color = PhairPlayColors.Muted,
                )
                if (card.peerCount > 0u) {
                    Text(
                        text = pluralStringResource(
                            R.plurals.protocol_card_peers,
                            card.peerCount.toInt(),
                            card.peerCount.toInt(),
                        ),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                }
            }

            val failure = card.lastError
            if (!failure.isNullOrBlank()) {
                Spacer(modifier = Modifier.size(8.dp))
                Text(
                    text = failure,
                    style = MaterialTheme.typography.bodyMedium,
                    color = PhairPlayColors.Failure,
                )
            }
        }
    }
}

/**
 * The sentence about the ports of the protocol.
 */
@Composable
private fun portsOf(card: ProtocolCardState): String = when {
    card.tcpPort != null && card.udpPort != null ->
        stringResource(R.string.protocol_card_ports, card.tcpPort.toString(), card.udpPort.toString())
    card.tcpPort != null -> stringResource(R.string.protocol_card_ports_tcp_only, card.tcpPort.toString())
    card.udpPort != null -> stringResource(R.string.protocol_card_ports, "—", card.udpPort.toString())
    else -> stringResource(R.string.protocol_card_no_port)
}
