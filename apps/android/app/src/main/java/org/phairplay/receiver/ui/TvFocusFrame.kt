package org.phairplay.receiver.ui

import androidx.compose.foundation.border
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * The frame that shows that an element has the focus.
 *
 * WHAT: three modifiers that together make the focus visible: a frame of
 * four dp in the accent colour, a growth of three percent and a veil that
 * brightens the surface.
 *
 * WHY: A user of a television holds the remote control at arm's length
 * and does not look at the screen the whole time. A frame of a few
 * millimetres is the first sign, but it is easy to lose on a panel with a
 * bad contrast — therefore the card also grows and becomes brighter. The
 * frame is four dp thick and not three, because three dp is the smallest
 * frame that survives a television of the year 2010.
 *
 * The colour of the frame meets the contrast of 3:1 against the surface
 * of the card (6.44:1) and against the background of the screen (7.44:1).
 *
 * EXAMPLE: `Modifier.tvFocusFrame(focused = true, shape = shape)` draws
 * the frame, the growth and the veil of a card that has the focus.
 */
fun Modifier.tvFocusFrame(
    focused: Boolean,
    shape: Shape,
    accent: Color = PhairPlayColors.Accent,
    width: Dp = 4.dp,
): Modifier {
    val scale = if (focused) FOCUS_SCALE else 1f
    return this
        .graphicsLayer {
            scaleX = scale
            scaleY = scale
        }
        .then(
            if (focused) {
                Modifier.border(width = width, color = accent, shape = shape)
            } else {
                Modifier
            },
        )
}

/**
 * The veil that lies over the surface of an element that has the focus.
 *
 * WHY: The veil is drawn *under* the text of the element, so that the
 * text keeps its contrast and only the surface becomes brighter.
 */
fun Modifier.tvFocusVeil(focused: Boolean): Modifier =
    if (focused) {
        drawBehind { drawRect(color = PhairPlayColors.FocusVeil) }
    } else {
        this
    }

/**
 * The growth of an element that has the focus.
 */
private const val FOCUS_SCALE: Float = 1.03f
