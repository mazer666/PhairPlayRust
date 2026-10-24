package org.phairplay.receiver.ui

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The order of the focus of the dashboard.
 *
 * WHAT: the tests of the rule that says where a key of the D-pad leads
 * from one place of the dashboard to another.
 *
 * WHY: The order of the focus is the promise of the shell towards the user
 * of a remote control: "right" always leads to the same card, and the
 * focus never jumps to a place that the user did not ask for. The rule is
 * a pure function, and a pure function can be tested without a device —
 * which is the only way to keep the promise true when the dashboard is
 * changed.
 *
 * EXAMPLE: the focus is on the card of Miracast (the second card of the
 * grid) and the user presses "down": the focus moves to the card of DLNA
 * (the fourth card), because the column is the vertical axis of the grid.
 */
class TvFocusGraphTest {

    /** The number of the cards of the grid in the tests. */
    private val cards = 4

    /**
     * Moves the focus with the rule of the dashboard.
     */
    private fun move(
        from: TvFocusSpot,
        direction: TvDirection,
        banner: Boolean = false,
        cardCount: Int = cards,
    ): TvFocusSpot = TvFocusGraph.move(
        current = from,
        direction = direction,
        cardCount = cardCount,
        hasBanner = banner,
    )

    @Test
    fun downFromTheHeadGoesToTheFirstCard() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 0),
            move(TvFocusSpot(TvFocusPlace.HEADER), TvDirection.DOWN),
        )
    }

    @Test
    fun downFromTheHeadGoesToTheBannerWhileAFailureIsShown() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.BANNER),
            move(TvFocusSpot(TvFocusPlace.HEADER), TvDirection.DOWN, banner = true),
        )
    }

    @Test
    fun theHeadOnlyKnowsTheKeyDown() {
        val head = TvFocusSpot(TvFocusPlace.HEADER)
        assertEquals(head, move(head, TvDirection.UP))
        assertEquals(head, move(head, TvDirection.LEFT))
        assertEquals(head, move(head, TvDirection.RIGHT))
    }

    @Test
    fun upFromTheBannerGoesToTheHeadAndDownToTheFirstCard() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.HEADER),
            move(TvFocusSpot(TvFocusPlace.BANNER), TvDirection.UP),
        )
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 0),
            move(TvFocusSpot(TvFocusPlace.BANNER), TvDirection.DOWN),
        )
    }

    @Test
    fun rightAndLeftStayInTheRow() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 1),
            move(TvFocusSpot(TvFocusPlace.CARD, 0), TvDirection.RIGHT),
        )
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 0),
            move(TvFocusSpot(TvFocusPlace.CARD, 1), TvDirection.LEFT),
        )
    }

    @Test
    fun theEdgeOfTheRowKeepsTheFocus() {
        val left = TvFocusSpot(TvFocusPlace.CARD, 0)
        val right = TvFocusSpot(TvFocusPlace.CARD, 1)
        assertEquals(left, move(left, TvDirection.LEFT))
        assertEquals(right, move(right, TvDirection.RIGHT))
    }

    @Test
    fun upAndDownStayInTheColumn() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 2),
            move(TvFocusSpot(TvFocusPlace.CARD, 0), TvDirection.DOWN),
        )
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 3),
            move(TvFocusSpot(TvFocusPlace.CARD, 1), TvDirection.DOWN),
        )
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 0),
            move(TvFocusSpot(TvFocusPlace.CARD, 2), TvDirection.UP),
        )
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 1),
            move(TvFocusSpot(TvFocusPlace.CARD, 3), TvDirection.UP),
        )
    }

    @Test
    fun theLowerRowHasNoRowBelowIt() {
        val lower = TvFocusSpot(TvFocusPlace.CARD, 2)
        assertEquals(lower, move(lower, TvDirection.DOWN))
    }

    @Test
    fun upFromTheUpperRowGoesToTheHead() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.HEADER),
            move(TvFocusSpot(TvFocusPlace.CARD, 0), TvDirection.UP),
        )
    }

    @Test
    fun upFromTheUpperRowGoesToTheBannerWhileAFailureIsShown() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.BANNER),
            move(TvFocusSpot(TvFocusPlace.CARD, 0), TvDirection.UP, banner = true),
        )
    }

    @Test
    fun aDashboardWithoutCardsKeepsTheFocus() {
        val head = TvFocusSpot(TvFocusPlace.HEADER)
        assertEquals(head, move(head, TvDirection.DOWN, cardCount = 0))
    }

    @Test
    fun aGridWithOneCardStaysInItsColumn() {
        assertEquals(
            TvFocusSpot(TvFocusPlace.CARD, 0),
            move(TvFocusSpot(TvFocusPlace.CARD, 0), TvDirection.RIGHT, cardCount = 1),
        )
    }
}
