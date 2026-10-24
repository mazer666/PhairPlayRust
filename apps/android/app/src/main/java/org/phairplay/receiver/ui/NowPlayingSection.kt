package org.phairplay.receiver.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R
import org.phairplay.receiver.model.NowPlayingState
import org.phairplay.receiver.model.uptimeText

/**
 * The view of the session that is running.
 *
 * WHAT: the sender, the formats of the stream, the metadata of the track
 * and the artwork that the sender sent.
 *
 * WHY: The sender of a stream is the reason a user opens the application
 * while something is playing. The view therefore leads with the name of
 * the sender and with the track, and it shows the technical details
 * underneath — a user who does not care about the sample rate does not
 * have to read it.
 *
 * EXAMPLE: `NowPlayingSection(session)` draws the session of an AirPlay
 * sender that plays a track.
 */
@Composable
fun NowPlayingSection(
    session: NowPlayingState?,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.now_playing_title),
                style = MaterialTheme.typography.titleMedium,
                color = PhairPlayColors.Muted,
            )
            Spacer(modifier = Modifier.size(8.dp))
            if (session == null) {
                Text(
                    text = stringResource(R.string.now_playing_empty),
                    style = MaterialTheme.typography.bodyLarge,
                    color = PhairPlayColors.Body,
                )
                return@Column
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                val artwork = remember(session.artwork) {
                    session.artwork?.let { bytes ->
                        BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                    }
                }
                if (artwork != null) {
                    Image(
                        bitmap = artwork.asImageBitmap(),
                        contentDescription = stringResource(R.string.now_playing_artwork),
                        modifier = Modifier
                            .size(72.dp)
                            .clip(RoundedCornerShape(8.dp)),
                    )
                    Spacer(modifier = Modifier.size(12.dp))
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = session.title ?: stringResource(R.string.now_playing_title_unknown),
                        style = MaterialTheme.typography.titleLarge,
                        color = PhairPlayColors.Body,
                    )
                    Text(
                        text = session.artist ?: stringResource(R.string.now_playing_artist_unknown),
                        style = MaterialTheme.typography.bodyLarge,
                        color = PhairPlayColors.Muted,
                    )
                    Text(
                        text = session.album ?: stringResource(R.string.now_playing_album_unknown),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                }
            }

            Spacer(modifier = Modifier.size(12.dp))

            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(
                    text = session.senderName ?: stringResource(R.string.now_playing_sender_unknown),
                    style = MaterialTheme.typography.bodyLarge,
                    color = PhairPlayColors.Body,
                )
                val address = session.senderAddress
                if (!address.isNullOrBlank()) {
                    Text(
                        text = address,
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                }
                val video = session.video
                if (video != null) {
                    Text(
                        text = stringResource(R.string.now_playing_video_format, video),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                } else {
                    Text(
                        text = stringResource(R.string.now_playing_video_unknown),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                }
                val audio = session.audio
                if (audio != null) {
                    Text(
                        text = stringResource(R.string.now_playing_audio_format, audio),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                } else {
                    Text(
                        text = stringResource(R.string.now_playing_audio_unknown),
                        style = MaterialTheme.typography.bodyMedium,
                        color = PhairPlayColors.Muted,
                    )
                }
                Text(
                    text = stringResource(R.string.now_playing_uptime, uptimeText(session.uptimeMs)),
                    style = MaterialTheme.typography.bodyMedium,
                    color = PhairPlayColors.Muted,
                )
            }
        }
    }
}
