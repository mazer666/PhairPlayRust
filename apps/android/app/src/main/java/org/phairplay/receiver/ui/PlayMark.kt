package org.phairplay.receiver.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * The mark of a running stream.
 *
 * WHAT: a filled triangle, the sign that everybody knows.
 *
 * WHY: The panel of the running session and the card of a protocol both
 * have to say "something is playing here" with a sign that is not a
 * colour. The triangle is the sign of a stream in every interface of the
 * world, and it is different from the ring of a protocol that only waits
 * for a sender.
 *
 * EXAMPLE: `PlayMark(color = PhairPlayColors.Advertising)` draws the
 * triangle of a running session.
 */
@Composable
fun PlayMark(
    color: Color,
    modifier: Modifier = Modifier,
    diameter: Dp = 18.dp,
) {
    Canvas(modifier = modifier.size(diameter)) {
        val radius = size.minDimension / 2f
        val path = Path().apply {
            moveTo(size.width / 2f - radius * 0.7f, size.height / 2f - radius)
            lineTo(size.width / 2f + radius * 0.9f, size.height / 2f)
            lineTo(size.width / 2f - radius * 0.7f, size.height / 2f + radius)
            close()
        }
        drawPath(path = path, color = color)
    }
}
