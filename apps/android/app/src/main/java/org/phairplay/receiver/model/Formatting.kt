package org.phairplay.receiver.model

/**
 * The formatting of the running time of a session.
 *
 * WHAT: the number of hours, minutes and seconds that a session has been
 * running.
 *
 * WHY: The shell shows the age of a session, and the age is the only
 * information about a session that the core keeps in a form that survives
 * a change of the clock of the system. The format is a plain number and no
 * word, so that the text of the shell does not have to know the language
 * of the user.
 *
 * EXAMPLE: `uptimeText(65_000UL)` answers "1:05", and
 * `uptimeText(3_725_000UL)` answers "1:02:05".
 */
fun uptimeText(milliseconds: ULong): String {
    val totalSeconds = milliseconds / 1_000UL
    val seconds = totalSeconds % 60UL
    val minutes = (totalSeconds / 60UL) % 60UL
    val hours = totalSeconds / 3_600UL
    return if (hours > 0UL) {
        "$hours:${minutes.padded()}:${seconds.padded()}"
    } else {
        "$minutes:${seconds.padded()}"
    }
}

/**
 * A number with a leading zero, if it is smaller than ten.
 */
private fun ULong.padded(): String = if (this < 10UL) "0$this" else toString()
