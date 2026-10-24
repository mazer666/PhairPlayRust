package org.phairplay.receiver.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import uniffi.phairplay_ffi.FfiProtocolState

/**
 * The tests of the phase of a protocol.
 *
 * WHAT: the proof that the shell tells "switched off in the settings" and
 * "the service is not running" apart, and that every state of the engine
 * finds its phase.
 *
 * WHY: The two cases look the same on a screen and mean something
 * completely different. A protocol that is switched off is not advertised
 * at all; a protocol that is switched on but not running was stopped, and
 * the user can start it. A test that does not check the difference cannot
 * catch a mistake in it.
 */
class ProtocolPhaseTest {

    @Test
    fun aProtocolThatIsSwitchedOffIsDisabled() {
        val phase = protocolPhase(enabled = false, state = FfiProtocolState.ADVERTISING)
        assertEquals(ProtocolPhase.DISABLED, phase)
    }

    @Test
    fun aProtocolThatIsSwitchedOffIsDisabledEvenWhileItStops() {
        val phase = protocolPhase(enabled = false, state = FfiProtocolState.STOPPING)
        assertEquals(ProtocolPhase.DISABLED, phase)
    }

    @Test
    fun aStoppedProtocolIsStopped() {
        val phase = protocolPhase(enabled = true, state = FfiProtocolState.STOPPED)
        assertEquals(ProtocolPhase.STOPPED, phase)
    }

    @Test
    fun stoppedAndDisabledAreNotTheSame() {
        val stopped = protocolPhase(enabled = true, state = FfiProtocolState.STOPPED)
        val disabled = protocolPhase(enabled = false, state = FfiProtocolState.STOPPED)
        assertNotEquals(stopped, disabled)
    }

    @Test
    fun aProtocolThatNeverRanIsNotStarted() {
        val phase = protocolPhase(enabled = true, state = FfiProtocolState.IDLE)
        assertEquals(ProtocolPhase.NOT_STARTED, phase)
    }

    @Test
    fun everyStateOfTheEngineFindsItsPhase() {
        val expected = mapOf(
            FfiProtocolState.IDLE to ProtocolPhase.NOT_STARTED,
            FfiProtocolState.STARTING to ProtocolPhase.STARTING,
            FfiProtocolState.ADVERTISING to ProtocolPhase.ADVERTISING,
            FfiProtocolState.STREAMING to ProtocolPhase.STREAMING,
            FfiProtocolState.STOPPING to ProtocolPhase.STOPPING,
            FfiProtocolState.STOPPED to ProtocolPhase.STOPPED,
            FfiProtocolState.FAILED to ProtocolPhase.FAILED,
        )
        for ((state, phase) in expected) {
            assertEquals(phase, protocolPhase(enabled = true, state = state))
        }
    }

    @Test
    fun onlyTheStatesThatServeTheNetworkCountAsRunning() {
        assertEquals(false, ProtocolPhase.DISABLED.isRunning)
        assertEquals(false, ProtocolPhase.NOT_STARTED.isRunning)
        assertEquals(false, ProtocolPhase.STOPPED.isRunning)
        assertEquals(false, ProtocolPhase.FAILED.isRunning)
        assertEquals(true, ProtocolPhase.STARTING.isRunning)
        assertEquals(true, ProtocolPhase.ADVERTISING.isRunning)
        assertEquals(true, ProtocolPhase.STREAMING.isRunning)
        assertEquals(true, ProtocolPhase.STOPPING.isRunning)
    }
}
