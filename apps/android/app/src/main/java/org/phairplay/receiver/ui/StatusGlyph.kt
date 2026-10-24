package org.phairplay.receiver.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.model.ProtocolPhase

/**
 * The sign of the state of a protocol.
 *
 * WHAT: a small drawing that says in which state a protocol is, drawn
 * with the shape of the state and not only with its colour.
 *
 * WHY: A user with a colour weakness has to be able to tell the states
 * apart as well. Every state therefore has a shape of its own:
 *
 * | State        | Shape                          |
 * |--------------|--------------------------------|
 * | disabled     | a circle with a bar through it |
 * | not started  | an empty circle                |
 * | stopped      | an empty circle                |
 * | starting     | a half circle                  |
 * | stopping     | a half circle                  |
 * | advertising  | a ring with a dot in the middle|
 * | streaming    | a filled triangle              |
 * | error        | a triangle with a warning mark |
 *
 * EXAMPLE: `StatusGlyph(ProtocolPhase.STREAMING, PhairPlayColors.Streaming)`
 * draws the filled triangle of a running stream.
 */
@Composable
fun StatusGlyph(
    phase: ProtocolPhase,
    color: Color,
    modifier: Modifier = Modifier,
    diameter: Dp = 28.dp,
) {
    Canvas(modifier = modifier.size(diameter)) {
        val stroke = 3.dp.toPx()
        val radius = (size.minDimension / 2f) - stroke
        val centre = Offset(size.width / 2f, size.height / 2f)
        when (phase) {
            // A circle with a bar through it: the protocol is off.
            ProtocolPhase.DISABLED -> {
                drawCircle(
                    color = color,
                    radius = radius,
                    center = centre,
                    style = Stroke(width = stroke),
                )
                drawLine(
                    color = color,
                    start = Offset(centre.x - radius * 0.72f, centre.y + radius * 0.72f),
                    end = Offset(centre.x + radius * 0.72f, centre.y - radius * 0.72f),
                    strokeWidth = stroke,
                    cap = StrokeCap.Round,
                )
            }

            // An empty circle: the service does not run.
            ProtocolPhase.NOT_STARTED, ProtocolPhase.STOPPED -> {
                drawCircle(
                    color = color,
                    radius = radius,
                    center = centre,
                    style = Stroke(width = stroke),
                )
            }

            // A half circle: the service is on its way.
            ProtocolPhase.STARTING, ProtocolPhase.STOPPING -> {
                drawArc(
                    color = color,
                    startAngle = 90f,
                    sweepAngle = 180f,
                    useCenter = true,
                    topLeft = Offset(centre.x - radius, centre.y - radius),
                    size = Size(radius * 2f, radius * 2f),
                )
            }

            // A ring with a dot: the service waits for a sender.
            ProtocolPhase.ADVERTISING -> {
                drawCircle(
                    color = color,
                    radius = radius,
                    center = centre,
                    style = Stroke(width = stroke),
                )
                drawCircle(color = color, radius = radius * 0.34f, center = centre)
            }

            // A filled triangle: a stream is running.
            ProtocolPhase.STREAMING -> {
                val triangle = Path().apply {
                    moveTo(centre.x - radius * 0.7f, centre.y - radius)
                    lineTo(centre.x + radius, centre.y)
                    lineTo(centre.x - radius * 0.7f, centre.y + radius)
                    close()
                }
                drawPath(path = triangle, color = color)
            }

            // A triangle with a warning mark: the service failed.
            ProtocolPhase.FAILED -> {
                val triangle = Path().apply {
                    moveTo(centre.x, centre.y - radius)
                    lineTo(centre.x + radius * 0.92f, centre.y + radius * 0.72f)
                    lineTo(centre.x - radius * 0.92f, centre.y + radius * 0.72f)
                    close()
                }
                drawPath(path = triangle, color = color, style = Stroke(width = stroke))
                drawRect(
                    color = color,
                    topLeft = Offset(centre.x - stroke, centre.y - radius * 0.30f),
                    size = Size(stroke * 2f, radius * 0.62f),
                )
                drawCircle(
                    color = color,
                    radius = stroke * 1.15f,
                    center = Offset(centre.x, centre.y + radius * 0.52f),
                )
            }
        }
    }
}
