package org.phairplay.receiver

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import androidx.core.graphics.drawable.IconCompat
import org.phairplay.receiver.model.ReceiverUiState

/**
 * The notification of the foreground service.
 *
 * WHAT: the channel of the notification, the notification itself and the
 * action that stops the receiver.
 *
 * WHY: The receiver runs in the background, and the user has to be able to
 * see that it is still there and which sender is connected. The system
 * shows the notification of a foreground service even when the user has
 * withdrawn the permission for notifications, so the shell needs no
 * further permission for it.
 *
 * EXAMPLE: `build(context, state)` answers the notification that says
 * "Waiting for a sender" or the name of the sender that is sending.
 */
object ReceiverNotification {

    /** The channel of the notification of the receiver. */
    const val CHANNEL_ID: String = "phairplay-receiver"

    /** The number of the notification of the receiver. */
    const val NOTIFICATION_ID: Int = 4711

    /**
     * Creates the channel of the notification.
     *
     * WHY: A channel exists from the first start of the application on. The
     * system keeps the settings of the user for it, so it must not be
     * created again with other properties.
     */
    fun createChannel(context: Context) {
        val channel = NotificationChannel(
            CHANNEL_ID,
            context.getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        )
        channel.description = context.getString(R.string.notification_channel_description)
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    /**
     * The notification that shows the state of the receiver.
     *
     * @param context the context of the service
     * @param state the state of the receiver
     */
    fun build(context: Context, state: ReceiverUiState): Notification {
        val sender = state.nowPlaying?.senderName
        val text = if (sender.isNullOrBlank()) {
            context.getString(R.string.notification_text_idle)
        } else {
            context.getString(R.string.notification_text_session, sender)
        }
        val stop = PendingIntent.getService(
            context,
            0,
            Intent(context, ReceiverService::class.java).setAction(ReceiverService.ACTION_STOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val stopAction = NotificationCompat.Action.Builder(
            IconCompat.createWithResource(context, android.R.drawable.ic_menu_close_clear_cancel),
            context.getString(R.string.notification_action_stop),
            stop,
        ).build()
        return NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(context.getString(R.string.notification_title))
            .setContentText(text)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .addAction(stopAction)
            .build()
    }
}
