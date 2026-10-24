package org.phairplay.receiver.model

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The tests of the formatting of the running time.
 *
 * WHY: The age of a session is the one number about a session that the
 * core keeps in a form that survives a change of the clock of the system.
 * A test that does not check the format cannot catch a session that is
 * shown with an age of 0 seconds.
 */
class FormattingTest {

    @Test
    fun aSessionThatJustStartedHasNoMinutes() {
        assertEquals("0:00", uptimeText(0uL))
    }

    @Test
    fun theSecondsAreWrittenWithTwoDigits() {
        assertEquals("0:05", uptimeText(5_000uL))
        assertEquals("0:59", uptimeText(59_000uL))
    }

    @Test
    fun theMinutesAreCounted() {
        assertEquals("1:05", uptimeText(65_000uL))
        assertEquals("12:34", uptimeText(754_000uL))
    }

    @Test
    fun theHoursAreAddedWhenTheSessionIsLongerThanAnHour() {
        assertEquals("1:00:00", uptimeText(3_600_000uL))
        assertEquals("1:02:05", uptimeText(3_725_000uL))
    }
}
