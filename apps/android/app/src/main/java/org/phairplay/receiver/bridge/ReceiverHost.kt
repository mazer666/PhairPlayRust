package org.phairplay.receiver.bridge

import android.content.Context
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.phairplay.receiver.model.BridgeFailure
import org.phairplay.receiver.model.ReceiverUiState
import org.phairplay.receiver.platform.PlatformCapabilities
import uniffi.phairplay_ffi.FfiException
import uniffi.phairplay_ffi.Receiver

/**
 * The owner of the receiver of the process.
 *
 * WHAT: the object that creates the receiver of the bridge, hands it the
 * capabilities of the device, and holds the controller that the screens
 * read.
 *
 * WHY: The service and the activity live in the same process, and both of
 * them need the same receiver. The owner makes sure that there is exactly
 * one: two receivers would fight over the ports of the protocols.
 *
 * EXAMPLE: the service calls `start()` when it comes up, and the activity
 * collects `state` to draw the dashboard.
 */
class ReceiverHost(context: Context) {

    /** The scope of the controller. */
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    /** What the device can do. */
    private val platform = PlatformCapabilities(context)

    /** The state that the screens read. */
    private val mutableState = MutableStateFlow(ReceiverUiState.EMPTY)

    /** The state that the screens read. */
    val state: StateFlow<ReceiverUiState> = mutableState.asStateFlow()

    /** The controller of the receiver, if the receiver is running. */
    var controller: ReceiverController? = null
        private set

    /**
     * Starts the receiver.
     *
     * WHY: The method may fail in three ways: the defaults file may be
     * broken, the configuration may be refused by the core, and the
     * runtime of the bridge may not come up. All three are reported as a
     * failure of the bridge, so that the interface can show the reason
     * instead of an empty receiver.
     */
    fun start() {
        if (controller != null) {
            return
        }
        val configuration = try {
            val text = context.assets.open(ReceiverDefaults.FILE_NAME)
                .bufferedReader()
                .use { it.readText() }
            ReceiverDefaults.toConfiguration(
                values = ReceiverDefaults.parse(text),
                deviceName = platform.deviceName(),
            )
        } catch (failure: Exception) {
            report(BridgeFailure(code = INTERNAL_CODE, details = failure.message ?: "the defaults are broken"))
            return
        }

        val receiver = try {
            Receiver(configuration)
        } catch (failure: FfiException) {
            report(failure.toBridgeFailure())
            return
        }

        try {
            receiver.setCapabilities(platform.capabilities())
        } catch (failure: FfiException) {
            report(failure.toBridgeFailure())
        }

        val created = ReceiverController(scope, receiver)
        controller = created
        scope.launch {
            created.state.collect { value -> mutableState.value = value }
        }
    }

    /**
     * Stops the receiver and gives the resources back.
     */
    fun stop() {
        val running = controller ?: return
        controller = null
        running.close()
        mutableState.value = ReceiverUiState.EMPTY
    }

    /**
     * Shows a failure in the interface.
     */
    private fun report(failure: BridgeFailure) {
        Log.e(TAG, "the receiver could not be started: ${failure.details}")
        mutableState.value = mutableState.value.copy(failure = failure)
    }

    companion object {
        /**
         * The code of the core for an unspecified internal failure.
         */
        const val INTERNAL_CODE: Int = 9

        /** The tag of the messages of this class. */
        private const val TAG = "PhairPlayHost"
    }
}
