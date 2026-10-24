package org.phairplay.receiver.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import org.phairplay.receiver.R
import org.phairplay.receiver.model.ProtocolPhase

/**
 * The banner of a failure of the bridge.
 *
 * WHAT: the explanation of the failure, the code of the core and the
 * button that hides the banner.
 *
 * WHY: A failure of the bridge must not be silent, and it must not hide
 * the protocols either. The banner therefore lies between the head and
 * the grid, and it is the only thing on the screen that has to be read
 * first. The button that hides it is a real focus point: a user of a
 * remote control cannot tap a small cross in a corner.
 *
 * EXAMPLE: `FailureBanner(code = 9, details = "…", requester) { … }`
 * draws the banner of a failure of the bridge.
 */
@Composable
fun FailureBanner(
    code: Int,
    details: String,
    focusRequester: FocusRequester,
    onDismiss: () -> Unit,
    onFocusChanged: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val focused by interactionSource.collectIsFocusedAsState()
    val shape = RoundedCornerShape(18.dp)

    Card(
        modifier = modifier.fillMaxWidth(),
        shape = shape,
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Surface),
        border = BorderStroke(width = 2.dp, color = PhairPlayColors.Failure),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            StatusGlyph(
                phase = ProtocolPhase.FAILED,
                color = PhairPlayColors.Failure,
            )
            Spacer(modifier = Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(R.string.failure_title),
                    style = MaterialTheme.typography.titleMedium,
                    color = PhairPlayColors.Failure,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(modifier = Modifier.size(4.dp))
                Text(
                    text = stringResource(R.string.failure_code, code),
                    style = MaterialTheme.typography.bodySmall,
                    color = PhairPlayColors.Muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (details.isNotBlank()) {
                    Text(
                        text = details,
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Body,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            Spacer(modifier = Modifier.width(16.dp))
            Button(
                onClick = onDismiss,
                interactionSource = interactionSource,
                modifier = Modifier
                    .focusRequester(focusRequester)
                    .onFocusChanged { state ->
                        onFocusChanged(state.isFocused)
                    }
                    .tvFocusFrame(focused = focused, shape = RoundedCornerShape(12.dp)),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = PhairPlayColors.Failure,
                    contentColor = PhairPlayColors.Background,
                ),
                contentPadding = PaddingValues(horizontal = 20.dp, vertical = 10.dp),
            ) {
                Text(
                    text = stringResource(R.string.failure_dismiss),
                    style = MaterialTheme.typography.labelLarge,
                )
            }
        }
    }
}
