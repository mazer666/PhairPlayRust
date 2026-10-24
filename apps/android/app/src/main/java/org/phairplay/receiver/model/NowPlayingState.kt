package org.phairplay.receiver.model

/**
 * The session that is running right now, as the shell shows it.
 *
 * WHAT: the sender, the formats of the stream, the metadata of the track
 * and the artwork that the sender sent.
 *
 * WHY: The metadata of a track does not arrive with the snapshot of a
 * session — it arrives later, as an event. The shell therefore keeps the
 * snapshot and the metadata of a session together, and the view of the
 * running session always shows the newest of both.
 *
 * EXAMPLE: a session of an AirPlay sender that plays a track carries the
 * title of the track, the artist and the album.
 */
data class NowPlayingState(
    /** The identifier of the session. */
    val sessionId: ULong,
    /** The stable name of the protocol that carries the session. */
    val protocol: String,
    /** The name of the sender, if it announced one. */
    val senderName: String?,
    /** The address of the sender, if it is known. */
    val senderAddress: String?,
    /**
     * The description of the video stream, if the session carries one.
     *
     * WHY: The description is built from the codec and the size of the
     * picture — "H.264 1920x1080". These are the names of the standards,
     * and they are the same in every language; the sentence around them
     * comes from the resources.
     */
    val video: String?,
    /** The description of the audio stream, if the session carries one. */
    val audio: String?,
    /** The title of the track, if the sender reported one. */
    val title: String?,
    /** The artist of the track, if the sender reported one. */
    val artist: String?,
    /** The album of the track, if the sender reported one. */
    val album: String?,
    /** The artwork of the track, if the sender sent one. */
    val artwork: ByteArray?,
    /** How long the session has been running, in milliseconds. */
    val uptimeMs: ULong,
) {
    /**
     * Compares two sessions.
     *
     * WHY: The artwork is an array, and the generated `equals` of a data
     * class compares arrays by their identity. Two states of the same
     * session would therefore never be equal, and the Compose runtime
     * would redraw the artwork on every frame.
     */
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is NowPlayingState) return false
        return sessionId == other.sessionId &&
            protocol == other.protocol &&
            senderName == other.senderName &&
            senderAddress == other.senderAddress &&
            video == other.video &&
            audio == other.audio &&
            title == other.title &&
            artist == other.artist &&
            album == other.album &&
            artwork.contentEquals(other.artwork) &&
            uptimeMs == other.uptimeMs
    }

    /** The hash code that belongs to [equals]. */
    override fun hashCode(): Int {
        var result = sessionId.hashCode()
        result = 31 * result + protocol.hashCode()
        result = 31 * result + (senderName?.hashCode() ?: 0)
        result = 31 * result + (senderAddress?.hashCode() ?: 0)
        result = 31 * result + (video?.hashCode() ?: 0)
        result = 31 * result + (audio?.hashCode() ?: 0)
        result = 31 * result + (title?.hashCode() ?: 0)
        result = 31 * result + (artist?.hashCode() ?: 0)
        result = 31 * result + (album?.hashCode() ?: 0)
        result = 31 * result + (artwork?.contentHashCode() ?: 0)
        result = 31 * result + uptimeMs.hashCode()
        return result
    }
}
