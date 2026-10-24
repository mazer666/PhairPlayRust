package org.phairplay.receiver.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.model.ProtocolCardState
import org.phairplay.receiver.model.ProtocolId

/**
 * The grid of the four protocols.
 *
 * WHAT: the four cards of the protocols in two rows of two, and nothing
 * else.
 *
 * WHY: A user of a television must be able to see every protocol at once.
 * A list that is scrolled hides half of the protocols below the edge of
 * the screen, and a user who scrolls with a remote control loses the
 * place where the focus was. The grid therefore fills the space that is
 * left over, and it never scrolls.
 *
 * The order of the cards is fixed — AirPlay, Miracast, Cast, DLNA — and
 * the columns are the axis of the horizontal keys, the rows are the axis
 * of the vertical keys. The grid does not move the focus by itself: the
 * screen does that, with the rule of [TvFocusGraph].
 *
 * EXAMPLE: `ProtocolGrid(cards, registry) { protocol, enabled -> ... }`
 * draws the four cards and gives every card its own focus requester.
 */
@Composable
fun ProtocolGrid(
    cards: List<ProtocolCardState>,
    registry: TvFocusRegistry,
    onToggle: (ProtocolId, Boolean) -> Unit,
    onCardFocused: (Int) -> Unit,
    modifier: Modifier = Modifier,
    dense: Boolean = false,
) {
    val gap = if (dense) DENSE_GRID_GAP else GRID_GAP
    Column(
        modifier = modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(gap),
    ) {
        cards.chunked(COLUMNS).forEachIndexed { rowIndex, row ->
            Row(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(gap),
            ) {
                row.forEachIndexed { columnIndex, card ->
                    val index = rowIndex * COLUMNS + columnIndex
                    ProtocolCard(
                        card = card,
                        focusRequester = registry.cardRequester(index) ?: return@forEachIndexed,
                        onToggle = onToggle,
                        onFocusChanged = { focused ->
                            if (focused) {
                                onCardFocused(index)
                            }
                        },
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight(),
                    )
                }
                // A row that has only one card keeps the width of a card,
                // so that the grid does not jump when a protocol is
                // missing.
                repeat(COLUMNS - row.size) {
                    Spacer(modifier = Modifier.weight(1f))
                }
            }
        }
    }
}

/**
 * The number of columns of the grid.
 *
 * WHY: The grid has two columns on every television and on every
 * telephone. Four protocols in two rows of two fit on a screen of 853 dp
 * of width, and they are read in one glance.
 */
private const val COLUMNS: Int = 2

/**
 * The gap between the cards of the grid.
 */
private val GRID_GAP = 14.dp

/**
 * The gap between the cards of a grid on a small television.
 *
 * WHY: On a screen of 360 dp of height every dp of the gap is a dp that
 * the cards do not have. The gap becomes eight dp, so that the cards
 * keep the height of their type.
 */
private val DENSE_GRID_GAP = 8.dp
