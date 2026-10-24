package org.phairplay.receiver.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R

/**
 * The panel that is drawn while the bridge has not answered yet.
 *
 * WHAT: one sentence in the middle of the screen.
 *
 * WHY: The grid of the protocols can only be drawn when the bridge has
 * told the shell which protocols exist. In the first second the screen
 * must therefore say that it is waiting, instead of showing an empty
 * grid that looks like a broken application.
 *
 * EXAMPLE: `WaitingPanel()` draws the sentence of a receiver that is
 * still starting.
 */
@Composable
fun WaitingPanel(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = stringResource(R.string.dashboard_waiting_protocols),
            style = MaterialTheme.typography.bodyLarge,
            color = PhairPlayColors.Muted,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(horizontal = 24.dp),
        )
    }
}
