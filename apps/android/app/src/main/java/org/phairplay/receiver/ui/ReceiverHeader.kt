package org.phairplay.receiver.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R

/**
 * The head of the dashboard.
 *
 * WHAT: the name of the device, one big status that says whether the
 * receiver runs, and the help that explains the keys of the remote
 * control.
 *
 * WHY: A user of a television looks at the screen from a distance and
 * holds the remote control in the hand. The head therefore answers the
 * two questions that come first — "is my receiver running?" and "what do
 * I press?" — in the largest type of the screen, and it is a focus point
 * of its own, so that the key "back" can bring the user back to the top.
 *
 * The status is a word and not a colour: "The receiver is running" and
 * "The receiver is not running" are two different sentences.
 *
 * EXAMPLE: `ReceiverHeader("Attic", running = true, waiting = false)`
 * draws the head of a receiver that is waiting for a sender.
 */
@Composable
fun ReceiverHeader(
    deviceName: String,
    running: Boolean,
    waiting: Boolean,
    version: String,
    focusRequester: FocusRequester,
    onFocusChanged: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    dense: Boolean = false,
) {
    var focused by remember { mutableStateOf(false) }
    val shape = RoundedCornerShape(18.dp)
    val statusText = when {
        waiting -> stringResource(R.string.dashboard_receiver_waiting)
        running -> stringResource(R.string.dashboard_receiver_running)
        else -> stringResource(R.string.dashboard_receiver_stopped)
    }
    val statusColor = when {
        waiting -> PhairPlayColors.Transition
        running -> PhairPlayColors.Advertising
        else -> PhairPlayColors.Idle
    }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .focusRequester(focusRequester)
            .onFocusChanged {
                focused = it.isFocused
                onFocusChanged(it.isFocused)
            }
            .focusable()
            .tvFocusFrame(focused = focused, shape = shape)
            .padding(horizontal = 18.dp, vertical = if (dense) 8.dp else 12.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(modifier = Modifier.weight(1f)) {
                // The word "receiver" above the name of the device is a
                // decoration. On a small television the decoration is the
                // first thing that goes.
                if (!dense) {
                    Text(
                        text = stringResource(R.string.dashboard_title).uppercase(),
                        style = MaterialTheme.typography.labelSmall,
                        color = PhairPlayColors.Muted,
                    )
                    Spacer(modifier = Modifier.size(2.dp))
                }
                Text(
                    text = deviceName.ifBlank { stringResource(R.string.dashboard_device_unknown) },
                    style = if (dense) {
                        MaterialTheme.typography.titleLarge
                    } else {
                        MaterialTheme.typography.headlineSmall
                    },
                    color = PhairPlayColors.Body,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            StatusBadge(
                text = statusText,
                color = statusColor,
                dense = dense,
            )
        }

        Spacer(modifier = Modifier.size(6.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                text = stringResource(R.string.dashboard_dpad_help),
                style = if (dense) {
                    MaterialTheme.typography.bodySmall
                } else {
                    MaterialTheme.typography.bodyMedium
                },
                color = PhairPlayColors.Muted,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            if (version.isNotBlank()) {
                Spacer(modifier = Modifier.width(12.dp))
                Text(
                    text = stringResource(R.string.dashboard_version, version),
                    style = MaterialTheme.typography.labelSmall,
                    color = PhairPlayColors.Muted,
                    maxLines = 1,
                )
            }
        }
    }
}

/**
 * The big status of the receiver.
 *
 * WHY: The status is the answer to the first question of the user, and
 * it has to be read from three metres. It is therefore the largest text
 * of the head, it stands on a surface of its own, and it carries a dot
 * that has the colour of the state.
 */
@Composable
private fun StatusBadge(
    text: String,
    color: Color,
    dense: Boolean,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(14.dp))
            .background(PhairPlayColors.Surface)
            .padding(horizontal = 16.dp, vertical = if (dense) 6.dp else 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(14.dp)
                .clip(CircleShape)
                .background(color),
        )
        Spacer(modifier = Modifier.width(10.dp))
        Text(
            text = text,
            style = if (dense) {
                MaterialTheme.typography.titleMedium
            } else {
                MaterialTheme.typography.titleLarge
            },
            color = color,
            maxLines = 1,
        )
    }
}
