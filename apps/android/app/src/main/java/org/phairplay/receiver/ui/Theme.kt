package org.phairplay.receiver.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
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
 * it is read from a distance with a remote control in the hand. The type
 * is therefore the type of a television and not the type of a telephone:
 * the title of a card is 24 sp, the state of a card is 22 sp and the
 * smallest text of a card is 16 sp. The surfaces are dark, because the
 * shell is read in a living room.
 *
 * EXAMPLE: `PhairPlayTheme { ReceiverScreen(state) }` draws the dashboard.
 */
@Composable
fun PhairPlayTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(
            primary = PhairPlayColors.Accent,
            onPrimary = PhairPlayColors.Background,
            error = PhairPlayColors.Failure,
            onError = PhairPlayColors.Background,
            background = PhairPlayColors.Background,
            onBackground = PhairPlayColors.Body,
            surface = PhairPlayColors.Surface,
            onSurface = PhairPlayColors.Body,
            surfaceVariant = PhairPlayColors.SurfaceRaised,
            onSurfaceVariant = PhairPlayColors.Muted,
            outline = PhairPlayColors.Muted,
        ),
        typography = PhairPlayTypography,
        content = content,
    )
}

/**
 * The type of the shell.
 *
 * WHY: the sizes are the sizes of a television. The smallest text that
 * the dashboard draws is 16 sp, and the state of a protocol — the text
 * that a user reads first — is 22 sp.
 */
private val PhairPlayTypography = Typography(
    headlineMedium = TextStyle(fontSize = 30.sp, fontWeight = FontWeight.SemiBold),
    headlineSmall = TextStyle(fontSize = 26.sp, fontWeight = FontWeight.SemiBold),
    titleLarge = TextStyle(fontSize = 24.sp, fontWeight = FontWeight.Bold),
    titleMedium = TextStyle(fontSize = 22.sp, fontWeight = FontWeight.Bold),
    titleSmall = TextStyle(fontSize = 20.sp, fontWeight = FontWeight.SemiBold),
    bodyLarge = TextStyle(fontSize = 18.sp),
    bodyMedium = TextStyle(fontSize = 17.sp),
    bodySmall = TextStyle(fontSize = 16.sp),
    labelLarge = TextStyle(fontSize = 18.sp, fontWeight = FontWeight.Medium),
    labelMedium = TextStyle(fontSize = 16.sp, fontWeight = FontWeight.Medium),
    labelSmall = TextStyle(fontSize = 14.sp, fontWeight = FontWeight.Medium),
)
