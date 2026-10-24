package org.phairplay.receiver.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.model.ProtocolId
import org.phairplay.receiver.model.ReceiverUiState
import org.phairplay.receiver.model.orderedProtocols

/**
 * The dashboard of the shell.
 *
 * WHAT: the head of the dashboard, the banner of a failure, the panel of
 * the running session and the grid of the four protocols.
 *
 * WHY: The dashboard is the one screen of the shell, and it is read from
 * a distance with a remote control. It therefore shows everything at
 * once — the four protocols in a grid of two rows, never scrolled — and
 * it moves the focus with a written-down rule instead of with the order
 * of the composition.
 *
 * The screen owns the focus graph: it knows which of the places of the
 * dashboard has the focus, it moves the focus with [TvFocusGraph] and it
 * gives the key "back" the meaning "go back to the head".
 *
 * EXAMPLE: `ReceiverScreen(state, onToggle, onDismiss)` draws the whole
 * dashboard out of the state of the controller.
 */
@Composable
fun ReceiverScreen(
    state: ReceiverUiState,
    onToggle: (ProtocolId, Boolean) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val registry = remember { TvFocusRegistry() }
    var focused by remember { mutableStateOf(TvFocusSpot(TvFocusPlace.HEADER)) }
    val cards = remember(state.protocols) { orderedProtocols(state.protocols) }
    val failure = state.failure
    val hasBanner = failure != null

    // The screen opens with the focus on the head: the head carries the
    // status of the receiver and the help of the keys.
    LaunchedEffect(Unit) {
        registry.request(TvFocusSpot(TvFocusPlace.HEADER))
    }

    // The banner of a failure takes the focus with it when it is
    // dismissed. Without this the focus would be nowhere at all, and the
    // keys of the remote control would do nothing.
    LaunchedEffect(hasBanner) {
        if (!hasBanner && focused.place == TvFocusPlace.BANNER) {
            registry.request(TvFocusSpot(TvFocusPlace.HEADER))
        }
    }

    // The same happens when a protocol disappears from the grid while the
    // focus is on its card.
    LaunchedEffect(cards.size) {
        if (focused.place == TvFocusPlace.CARD && focused.index >= cards.size) {
            registry.request(TvFocusSpot(TvFocusPlace.HEADER))
        }
    }

    // The key "back" leads back to the head. On the head itself the key
    // belongs to the system: the user leaves the application.
    BackHandler(enabled = focused.place != TvFocusPlace.HEADER) {
        registry.request(TvFocusSpot(TvFocusPlace.HEADER))
    }

    Scaffold(
        modifier = modifier.fillMaxSize(),
        containerColor = PhairPlayColors.Background,
    ) { padding ->
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .onPreviewKeyEvent { event ->
                    onDpadKey(
                        event = event,
                        focused = focused,
                        cardCount = cards.size,
                        hasBanner = hasBanner,
                        registry = registry,
                    )
                },
            contentAlignment = Alignment.TopCenter,
        ) {
            // A television of the year 2010 has 360 dp of height. The
            // dashboard answers with a smaller type on such a screen,
            // instead of hiding half of the protocols below the edge.
            val dense = maxHeight < DENSE_HEIGHT
            Column(
                modifier = Modifier
                    .widthIn(max = 1600.dp)
                    .fillMaxSize()
                    .padding(
                        horizontal = 18.dp,
                        vertical = if (dense) 10.dp else 14.dp,
                    ),
            ) {
                ReceiverHeader(
                    deviceName = state.deviceName,
                    running = state.running,
                    waiting = cards.isEmpty(),
                    version = state.version,
                    focusRequester = registry.header,
                    dense = dense,
                    onFocusChanged = { hasFocus ->
                        if (hasFocus) {
                            focused = TvFocusSpot(TvFocusPlace.HEADER)
                        }
                    },
                )

                if (failure != null) {
                    Spacer(modifier = Modifier.height(12.dp))
                    FailureBanner(
                        code = failure.code,
                        details = failure.details,
                        focusRequester = registry.banner,
                        onDismiss = onDismiss,
                        onFocusChanged = { hasFocus ->
                            if (hasFocus) {
                                focused = TvFocusSpot(TvFocusPlace.BANNER)
                            }
                        },
                    )
                }

                val session = state.nowPlaying
                if (session != null) {
                    Spacer(modifier = Modifier.height(12.dp))
                    NowPlayingPanel(session = session, dense = dense)
                }

                Spacer(modifier = Modifier.height(12.dp))

                if (cards.isEmpty()) {
                    WaitingPanel(modifier = Modifier.weight(1f))
                } else {
                    ProtocolGrid(
                        cards = cards,
                        registry = registry,
                        onToggle = onToggle,
                        onCardFocused = { index ->
                            focused = TvFocusSpot(TvFocusPlace.CARD, index)
                        },
                        dense = dense,
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }
    }
}

/**
 * The keys of the D-pad move the focus of the dashboard.
 *
 * WHY: The focus of the dashboard is moved by hand and not by the
 * framework, so that the order is the same on every television and does
 * not depend on the order of the composition. A key that has no target
 * is eaten anyway: the user must not see the focus jump somewhere else
 * because a key was not handled.
 *
 * The key "back" is not handled here: it belongs to the [BackHandler] of
 * the screen, which lets the system close the application when the focus
 * is on the head.
 */
private fun onDpadKey(
    event: KeyEvent,
    focused: TvFocusSpot,
    cardCount: Int,
    hasBanner: Boolean,
    registry: TvFocusRegistry,
): Boolean {
    if (event.type != KeyEventType.KeyDown) {
        return false
    }
    if (event.key == Key.Escape) {
        registry.request(TvFocusSpot(TvFocusPlace.HEADER))
        return true
    }
    val direction = when (event.key) {
        Key.DirectionLeft -> TvDirection.LEFT
        Key.DirectionRight -> TvDirection.RIGHT
        Key.DirectionUp -> TvDirection.UP
        Key.DirectionDown -> TvDirection.DOWN
        else -> null
    } ?: return false
    registry.request(
        TvFocusGraph.move(
            current = focused,
            direction = direction,
            cardCount = cardCount,
            hasBanner = hasBanner,
        ),
    )
    return true
}

/**
 * The height below which the dashboard draws its texts smaller.
 *
 * WHY: The screen of a television of the year 2010 has 360 dp of height.
 * A dashboard that keeps the type of a big television on such a screen
 * pushes the grid of the protocols out of the picture — and a grid that
 * does not fit is worse than a grid with a smaller type.
 */
private val DENSE_HEIGHT = 420.dp
