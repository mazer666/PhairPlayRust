package org.phairplay.receiver.ui

import androidx.compose.ui.focus.FocusRequester
import org.phairplay.receiver.model.ProtocolId

/**
 * The focus graph of the dashboard.
 *
 * WHAT: the places of the dashboard that can hold the focus, the four
 * directions of the D-pad, and the rule that says where a key of the
 * D-pad leads from a place to another.
 *
 * WHY: A user of a television does not tap, the user presses a key. The
 * order of the focus therefore has to be written down and has to be the
 * same every time, otherwise the user presses "right" and does not know
 * what happens. The rule lives in a function without any drawing in it,
 * so that it can be read — and reasoned about — on its own.
 *
 * The graph of the dashboard is:
 *
 * ```text
 *              header
 *                |  up / down
 *            [ banner ]        (only while a failure is shown)
 *                |  up / down
 *         +------+------+
 *         |             |
 *      card 0        card 1        left / right inside a row
 *         |             |
 *      card 2        card 3        up / down inside the column
 * ```
 *
 * EXAMPLE: the focus is on card 1 (Miracast, upper right) and the user
 * presses "down": the focus moves to card 3 (DLNA, lower right), because
 * the column is the vertical axis of the grid.
 */
enum class TvFocusPlace {
    /** The head of the dashboard. */
    HEADER,

    /** The banner of a failure. */
    BANNER,

    /** One of the four cards of the protocols. */
    CARD,
}

/**
 * A place of the dashboard, with the number of the card if it is a card.
 */
data class TvFocusSpot(
    /** The place of the dashboard. */
    val place: TvFocusPlace,
    /** The number of the card, counted from the upper left. */
    val index: Int = 0,
)

/**
 * The four directions of the D-pad.
 */
enum class TvDirection {
    /** The key "up". */
    UP,

    /** The key "down". */
    DOWN,

    /** The key "left". */
    LEFT,

    /** The key "right". */
    RIGHT,
}

/**
 * The rule of the movement of the focus.
 *
 * WHY: The movement is a pure function of the place, the direction and
 * the shape of the screen. It never asks a view where the focus is and
 * never guesses: a key that has no target leaves the focus where it is,
 * and the key is eaten anyway, so that the framework does not move the
 * focus somewhere else behind the back of the user.
 */
object TvFocusGraph {

    /** The number of columns of the grid of the protocols. */
    const val COLUMNS: Int = 2

    /**
     * The place that the focus takes when the key of the D-pad is pressed.
     *
     * @param current the place that has the focus right now
     * @param direction the key that was pressed
     * @param cardCount the number of cards of the grid
     * @param hasBanner whether a failure is shown above the grid
     */
    fun move(
        current: TvFocusSpot,
        direction: TvDirection,
        cardCount: Int,
        hasBanner: Boolean,
    ): TvFocusSpot {
        if (cardCount <= 0) {
            return current
        }
        return when (current.place) {
            TvFocusPlace.HEADER -> when (direction) {
                TvDirection.DOWN -> if (hasBanner) {
                    TvFocusSpot(TvFocusPlace.BANNER)
                } else {
                    TvFocusSpot(TvFocusPlace.CARD, 0)
                }
                else -> current
            }

            TvFocusPlace.BANNER -> when (direction) {
                TvDirection.UP -> TvFocusSpot(TvFocusPlace.HEADER)
                TvDirection.DOWN -> TvFocusSpot(TvFocusPlace.CARD, 0)
                else -> current
            }

            TvFocusPlace.CARD -> {
                val index = current.index.coerceIn(0, cardCount - 1)
                when (direction) {
                    TvDirection.LEFT ->
                        if (index % COLUMNS == 1) TvFocusSpot(TvFocusPlace.CARD, index - 1) else current
                    TvDirection.RIGHT ->
                        if (index % COLUMNS == 0 && index + 1 < cardCount) {
                            TvFocusSpot(TvFocusPlace.CARD, index + 1)
                        } else {
                            current
                        }
                    TvDirection.UP -> when {
                        index >= COLUMNS -> TvFocusSpot(TvFocusPlace.CARD, index - COLUMNS)
                        hasBanner -> TvFocusSpot(TvFocusPlace.BANNER)
                        else -> TvFocusSpot(TvFocusPlace.HEADER)
                    }
                    TvDirection.DOWN ->
                        if (index + COLUMNS < cardCount) {
                            TvFocusSpot(TvFocusPlace.CARD, index + COLUMNS)
                        } else {
                            current
                        }
                }
            }
        }
    }
}

/**
 * The focus requesters of the dashboard.
 *
 * WHAT: one requester for the head, one for the banner of a failure and
 * one for every card of the grid.
 *
 * WHY: The focus of the dashboard is moved by hand, and not by the
 * framework. The framework would walk over the cards in the order of the
 * composition, and a television that is read from a distance needs an
 * order that the user can learn in one try.
 *
 * EXAMPLE: `registry.request(TvFocusSpot(TvFocusPlace.CARD, 2))` puts the
 * focus on the card of Google Cast.
 */
class TvFocusRegistry(cardSlots: Int = ProtocolId.entries.size) {

    /** The requester of the head of the dashboard. */
    val header: FocusRequester = FocusRequester()

    /** The requester of the banner of a failure. */
    val banner: FocusRequester = FocusRequester()

    /** The requesters of the cards of the protocols. */
    private val cards: List<FocusRequester> = List(cardSlots) { FocusRequester() }

    /**
     * The requester of a card.
     *
     * @param index the number of the card, counted from the upper left
     */
    fun cardRequester(index: Int): FocusRequester? = cards.getOrNull(index)

    /**
     * The requester of a place.
     *
     * @param spot the place that should get the focus
     */
    fun requesterFor(spot: TvFocusSpot): FocusRequester? = when (spot.place) {
        TvFocusPlace.HEADER -> header
        TvFocusPlace.BANNER -> banner
        TvFocusPlace.CARD -> cardRequester(spot.index)
    }

    /**
     * Puts the focus on a place.
     *
     * @param spot the place that should get the focus
     */
    fun request(spot: TvFocusSpot) {
        requesterFor(spot)?.requestFocus()
    }
}
