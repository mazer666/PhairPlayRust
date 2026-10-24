package org.phairplay.receiver.ui

import androidx.compose.ui.graphics.Color

/**
 * The colours of the shell.
 *
 * WHY: The receiver is an application that runs in the background of a
 * living room. The colours are calm on purpose: a bright red would shine
 * through a dark room and would suggest an error where there is none. The
 * red is reserved for a real failure.
 */
object PhairPlayColors {

    /** The colour of the receiver itself. */
    val Accent = Color(0xFF1F6FEB)

    /** The colour of a protocol that advertises itself. */
    val Advertising = Color(0xFF1F8A5B)

    /** The colour of a running stream. */
    val Streaming = Color(0xFF0F7C8A)

    /** The colour of a state that is on its way. */
    val Transition = Color(0xFF8A6D1F)

    /** The colour of a protocol that is switched off or not running. */
    val Idle = Color(0xFF5B6672)

    /** The colour of a failure. */
    val Failure = Color(0xFFB3261E)

    /** The colour of the surface of a card. */
    val Surface = Color(0xFFFFFFFF)

    /** The colour of the background of the application. */
    val Background = Color(0xFFF7F8FA)

    /** The colour of the text of the body. */
    val Body = Color(0xFF1F2933)

    /** The colour of the text that is not important. */
    val Muted = Color(0xFF5B6672)
}
