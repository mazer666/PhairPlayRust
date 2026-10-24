package org.phairplay.receiver.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/**
 * The theme of the shell.
 *
 * WHAT: the typography and the colours that the screens of the shell draw
 * with.
 *
 * WHY: The application runs on a television as often as on a telephone, and
 * it is read from a distance. The type is therefore larger than the type
 * of a normal application, and the colours are the ones of
 * [PhairPlayColors].
 *
 * EXAMPLE: `PhairPlayTheme { ReceiverScreen(state) }` draws the dashboard.
 */
@Composable
fun PhairPlayTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = MaterialTheme.colorScheme.copy(
            primary = PhairPlayColors.Accent,
            error = PhairPlayColors.Failure,
            background = PhairPlayColors.Background,
            surface = PhairPlayColors.Surface,
            onSurface = PhairPlayColors.Body,
        ),
        typography = PhairPlayTypography,
        content = content,
    )
}

/**
 * The type of the shell.
 */
private val PhairPlayTypography = Typography(
    headlineSmall = TextStyle(fontSize = 22.sp, fontWeight = FontWeight.SemiBold),
    titleLarge = TextStyle(fontSize = 20.sp, fontWeight = FontWeight.SemiBold),
    titleMedium = TextStyle(fontSize = 17.sp, fontWeight = FontWeight.Medium),
    bodyLarge = TextStyle(fontSize = 16.sp),
    bodyMedium = TextStyle(fontSize = 14.sp),
    labelLarge = TextStyle(fontSize = 14.sp, fontWeight = FontWeight.Medium),
    labelMedium = TextStyle(fontSize = 12.sp, fontWeight = FontWeight.Medium),
)
