package org.phairplay.receiver

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import org.phairplay.receiver.bridge.ReceiverHost
import org.phairplay.receiver.ui.PhairPlayTheme
import org.phairplay.receiver.ui.ReceiverScreen

/**
 * The one screen of the shell.
 *
 * WHAT: the activity that shows the dashboard and that starts the
 * foreground service of the receiver.
 *
 * WHY: The service is the part of the shell that outlives the screen. The
 * activity therefore does not own the receiver — the application does —
 * and it only starts the service and draws what the controller reports.
 *
 * EXAMPLE: the activity collects the state of the controller and hands it
 * to the dashboard.
 */
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val host: ReceiverHost = (application as ReceiverApplication).host

        setContent {
            PhairPlayTheme {
                val state by host.state.collectAsStateWithLifecycle()
                ReceiverScreen(
                    state = state,
                    onToggle = { protocol, enabled ->
                        host.controller?.setProtocolEnabled(protocol, enabled)
                    },
                    onDismiss = { host.controller?.dismissFailure() },
                )
            }
        }

        // The receiver keeps running while the user is in another
        // application, so the service is started with the screen.
        ContextCompat.startForegroundService(this, Intent(this, ReceiverService::class.java))
    }
}
