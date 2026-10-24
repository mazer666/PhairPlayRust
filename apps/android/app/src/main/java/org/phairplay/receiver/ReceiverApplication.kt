package org.phairplay.receiver

import android.app.Application
import org.phairplay.receiver.bridge.ReceiverHost

/**
 * The application of the shell.
 *
 * WHAT: the object that holds the receiver of the process.
 *
 * WHY: The service and the activity live in the same process and need the
 * same receiver. The application is the one place where an object of the
 * process lives longer than a screen, so the owner of the receiver lives
 * there. Two receivers would fight over the ports of the protocols.
 *
 * EXAMPLE: the service reads `host` and starts the receiver with it.
 */
class ReceiverApplication : Application() {

    /**
     * The owner of the receiver of the process.
     */
    lateinit var host: ReceiverHost
        private set

    override fun onCreate() {
        super.onCreate()
        host = ReceiverHost(this)
    }
}
