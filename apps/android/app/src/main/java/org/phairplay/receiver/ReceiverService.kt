package org.phairplay.receiver

import android.annotation.SuppressLint
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import org.phairplay.receiver.bridge.ReceiverHost
import org.phairplay.receiver.platform.MulticastLockHolder

/**
 * The foreground service of the receiver.
 *
 * WHAT: the service that owns the receiver while the interface of the
 * application is in the background, and the notification that tells the
 * user that the receiver is still running.
 *
 * WHY: A mirroring session has to survive the backgrounding of the
 * interface — the user switches to another application to look something
 * up, and the picture must keep coming (FR-074). A foreground service is
 * the only way to keep a receiver alive on Android, and the service takes
 * the lock of the Wi-Fi radio, because Android drops the multicast packets
 * of an application that does not hold one.
 *
 * EXAMPLE: the activity starts the service; the user stops the receiver
 * with the action of the notification.
 */
class ReceiverService : Service() {

    /** The scope of the service. */
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    /** The lock of the Wi-Fi radio. */
    private lateinit var multicast: MulticastLockHolder

    /** The owner of the receiver. */
    private val host: ReceiverHost
        get() = (application as ReceiverApplication).host

    /**
     * Starts the receiver and keeps the notification up to date.
     *
     * WHY: the notification of a foreground service does not need the
     * permission `POST_NOTIFICATIONS`: the system shows the notification
     * of a running foreground service even when the user has withdrawn
     * the permission for notifications. The shell therefore does not ask
     * for it (FR-075) — and the check of the lint is silenced here on
     * purpose.
     */
    @SuppressLint("MissingPermission", "NotificationPermission")
    override fun onCreate() {
        super.onCreate()
        ReceiverNotification.createChannel(this)

        // The notification has to be there before the receiver starts: the
        // system gives the service five seconds to call `startForeground`,
        // and the receiver needs the shared library of the bridge, which
        // takes a moment on the first start.
        startForeground(
            ReceiverNotification.NOTIFICATION_ID,
            ReceiverNotification.build(this, host.state.value),
            ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK,
        )

        multicast = MulticastLockHolder(this)
        if (!multicast.acquire()) {
            Log.w(TAG, "the multicast lock could not be taken: the discovery may not work")
        }

        host.start()

        scope.launch {
            host.state.collect { state ->
                val notification = ReceiverNotification.build(this@ReceiverService, state)
                NotificationManagerCompat.from(this@ReceiverService)
                    .notify(ReceiverNotification.NOTIFICATION_ID, notification)
            }
        }
    }
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
            return START_NOT_STICKY
        }
        // The service is started again after the system has killed it: a
        // receiver that a user has started stays started.
        return START_STICKY
    }

    override fun onDestroy() {
        scope.cancel()
        if (::multicast.isInitialized) {
            multicast.release()
        }
        host.stop()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    companion object {
        /**
         * The action of the notification that stops the receiver.
         */
        const val ACTION_STOP: String = "org.phairplay.receiver.action.STOP"

        /** The tag of the messages of this class. */
        private const val TAG = "PhairPlayService"
    }
}
