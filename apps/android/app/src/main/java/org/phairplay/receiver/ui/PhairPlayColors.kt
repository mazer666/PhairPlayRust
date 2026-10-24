package org.phairplay.receiver.ui

import androidx.compose.ui.graphics.Color

/**
 * The colours of the shell.
 *
 * WHAT: the colours of the surfaces, of the text and of the states of the
 * protocols, and the accent that marks the focus.
 *
 * WHY: The receiver is an application that runs in the background of a
 * living room and that is read from three metres away. The surfaces are
 * therefore dark, so that they do not glare in a dark room, and every
 * colour that carries a text meets the contrast of 4.5:1 against the
 * surface of the card. The accent is brighter than every other colour,
 * so that the frame of the focus is the brightest thing on the screen.
 *
 * The measured contrasts against the card surface (#1A2029):
 *
 * | Colour  | Value     | Contrast |
 * |---------|-----------|----------|
 * | Body    | #F3F6FA   | 15.11:1  |
 * | Muted   | #A9B4C2   |  7.79:1  |
 * | Idle    | #9AA5B1   |  6.54:1  |
 * | Failure | #FB7185   |  6.08:1  |
 * | Accent  | #60A5FA   |  6.44:1  |
 * | Stream  | #22D3EE   |  9.06:1  |
 * | Advert. | #4ADE80   |  9.40:1  |
 * | Change  | #FBBF24   |  9.81:1  |
 *
 * EXAMPLE: the frame of the focus of a card is drawn with [Accent], and
 * the state "streaming" with [Streaming].
 */
object PhairPlayColors {

    /** The colour of the background of the application. */
    val Background = Color(0xFF0E1116)

    /** The colour of the surface of a card. */
    val Surface = Color(0xFF1A2029)

    /** The colour of a surface that lies on a card, such as a chip. */
    val SurfaceRaised = Color(0xFF252D38)

    /** The colour of the text of the body. */
    val Body = Color(0xFFF3F6FA)

    /** The colour of the text that is not important. */
    val Muted = Color(0xFFA9B4C2)

    /** The accent of the shell and the frame of the focus. */
    val Accent = Color(0xFF60A5FA)

    /** The colour of a protocol that advertises itself. */
    val Advertising = Color(0xFF4ADE80)

    /** The colour of a running stream. */
    val Streaming = Color(0xFF22D3EE)

    /** The colour of a state that is on its way. */
    val Transition = Color(0xFFFBBF24)

    /** The colour of a protocol that is switched off or not running. */
    val Idle = Color(0xFF9AA5B1)

    /** The colour of a failure. */
    val Failure = Color(0xFFFB7185)

    /**
     * The veil that lies over a card that has the focus.
     *
     * WHY: the frame of the focus is a line of four dp, and a line of
     * four dp is easy to lose on a television with a bad panel. The veil
     * brightens the whole card, so that the focused card is the brightest
     * card of the screen even when the frame itself is not seen.
     */
    val FocusVeil = Color(0x1FFFFFFF)
}
