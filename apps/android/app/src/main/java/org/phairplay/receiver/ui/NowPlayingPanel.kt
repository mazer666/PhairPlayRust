package org.phairplay.receiver.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.phairplay.receiver.R
import org.phairplay.receiver.model.NowPlayingState
import org.phairplay.receiver.model.uptimeText

/**
 * The panel of the session that is running.
 *
 * WHAT: the sender, the formats of the stream and the duration of the
 * session — on a surface of its own, above the cards of the protocols.
 *
 * WHY: The sender of a stream is the reason a user opens the application
 * while something is playing. The panel therefore leads with the name of
 * the sender, and it stands above the grid of the protocols instead of
 * being one line between the head and the cards: on a television the
 * running session is the most important thing on the screen, and it must
 * not be a sentence that scrolls out of the view.
 *
 * The panel is a picture and not a place of the focus. There is nothing
 * to do in it, and a focus point that does nothing is a trap for the
 * user of a remote control.
 *
 * EXAMPLE: `NowPlayingPanel(session)` draws the session of an AirPlay
 * sender that plays a track.
 */
@Composable
fun NowPlayingPanel(
    session: NowPlayingState,
    modifier: Modifier = Modifier,
    dense: Boolean = false,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(18.dp),
        colors = CardDefaults.cardColors(containerColor = PhairPlayColors.Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = if (dense) 10.dp else 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(modifier = Modifier.weight(1f)) {
                // The label above the name of the sender and the title of
                // the track are the first things that go on a small
                // television: the name of the sender, the codec and the
                // duration are what the user is looking for.
                val sender = session.senderName ?: stringResource(R.string.now_playing_sender_unknown)
                if (dense) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        PlayMark(color = PhairPlayColors.Advertising)
                        Spacer(modifier = Modifier.size(8.dp))
                        Text(
                            text = sender,
                            style = MaterialTheme.typography.titleMedium,
                            color = PhairPlayColors.Body,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                } else {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        PlayMark(color = PhairPlayColors.Advertising)
                        Spacer(modifier = Modifier.size(8.dp))
                        Text(
                            text = stringResource(R.string.now_playing_title).uppercase(),
                            style = MaterialTheme.typography.labelSmall,
                            color = PhairPlayColors.Muted,
                        )
                    }
                    Spacer(modifier = Modifier.size(2.dp))
                    Text(
                        text = sender,
                        style = MaterialTheme.typography.titleLarge,
                        color = PhairPlayColors.Body,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    val title = session.title ?: session.artist
                    if (!title.isNullOrBlank()) {
                        Spacer(modifier = Modifier.size(2.dp))
                        Text(
                            text = title,
                            style = MaterialTheme.typography.bodyMedium,
                            color = PhairPlayColors.Muted,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                Spacer(modifier = Modifier.size(2.dp))
                Text(
                    text = formatsOf(session),
                    style = MaterialTheme.typography.bodySmall,
                    color = PhairPlayColors.Muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            val artwork = remember(session.artwork) {
                session.artwork?.let { bytes ->
                    BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                }
            }
            if (artwork != null) {
                Spacer(modifier = Modifier.size(14.dp))
                Image(
                    bitmap = artwork.asImageBitmap(),
                    contentDescription = stringResource(R.string.now_playing_artwork),
                    modifier = Modifier
                        .size(72.dp)
                        .clip(RoundedCornerShape(10.dp)),
                )
            }
        }
    }
}

/**
 * The formats of the stream and the duration of the session.
 *
 * WHY: The codec is the technical detail that a user wants to see when
 * something does not look right, and the duration is the fact that tells
 * two sessions apart. The names of the codecs are the names of the
 * standards and are the same in every language; the sentence around them
 * comes from the resources.
 */
@Composable
private fun formatsOf(session: NowPlayingState): String {
    val video = session.video ?: stringResource(R.string.now_playing_video_unknown)
    val audio = session.audio ?: stringResource(R.string.now_playing_audio_unknown)
    val formats = stringResource(R.string.now_playing_formats, video, audio)
    val duration = stringResource(R.string.now_playing_chip_duration, uptimeText(session.uptimeMs))
    return "$formats · $duration"
}
