package org.phairplay.receiver.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R
import org.phairplay.receiver.model.ProtocolId
import org.phairplay.receiver.model.ReceiverUiState

/**
 * The dashboard of the shell.
 *
 * WHAT: the head with the identity of the receiver, the card of every
 * protocol, the view of the running session and the banner of a failure.
 *
 * WHY: The dashboard is the one screen of the shell, and it answers the
 * two questions a user has: "is my receiver running?" and "which sender is
 * sending right now?". The state of the four protocols is the middle of
 * the screen, because that is what the user changes.
 *
 * EXAMPLE: `ReceiverScreen(state, onToggle, onDismiss)` draws the whole
 * dashboard out of the state of the controller.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReceiverScreen(
    state: ReceiverUiState,
    onToggle: (ProtocolId, Boolean) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        containerColor = PhairPlayColors.Background,
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text(
                            text = stringResource(R.string.dashboard_title),
                            style = MaterialTheme.typography.titleLarge,
                        )
                        Text(
                            text = stringResource(R.string.dashboard_device, state.deviceName),
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = PhairPlayColors.Background,
                    titleContentColor = PhairPlayColors.Body,
                ),
            )
        },
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            val failure = state.failure
            if (failure != null) {
                item(key = "failure") {
                    FailureBanner(
                        code = failure.code,
                        details = failure.details,
                        onDismiss = onDismiss,
                    )
                }
            }

            item(key = "now-playing") {
                NowPlayingSection(session = state.nowPlaying)
            }

            items(items = state.protocols, key = { it.protocol.bridgeName }) { card ->
                ProtocolCard(card = card, onToggle = onToggle)
            }

            if (state.version.isNotBlank()) {
                item(key = "version") {
                    Text(
                        text = stringResource(R.string.dashboard_version, state.version),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                        modifier = Modifier.padding(top = 8.dp),
                    )
                }
            }
        }
    }
}

/**
 * The banner of a failure of the bridge.
 *
 * WHY: The bridge reports a failure with a code and an explanation. The
 * banner shows both, because the code is the part that a bug report needs
 * and the explanation is the part that the user needs.
 */
@Composable
private fun FailureBanner(
    code: Int,
    details: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Failure.copy(alpha = 0.08f)),
        elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.failure_title),
                style = MaterialTheme.typography.titleMedium,
                color = PhairPlayColors.Failure,
            )
            Spacer(modifier = Modifier.size(4.dp))
            Text(
                text = stringResource(R.string.failure_code, code),
                style = MaterialTheme.typography.bodyMedium,
                color = PhairPlayColors.Failure,
            )
            Text(
                text = details,
                style = MaterialTheme.typography.bodyMedium,
                color = PhairPlayColors.Body,
            )
            TextButton(onClick = onDismiss) {
                Text(text = stringResource(R.string.failure_dismiss))
            }
        }
    }
}
