package org.phairplay.receiver.model

import uniffi.phairplay_ffi.FfiAppConfig
import uniffi.phairplay_ffi.FfiAudioCodec
import uniffi.phairplay_ffi.FfiAudioFormat
import uniffi.phairplay_ffi.FfiProtocolConfig
import uniffi.phairplay_ffi.FfiProtocolKind
import uniffi.phairplay_ffi.FfiProtocolState
import uniffi.phairplay_ffi.FfiSessionSnapshot
import uniffi.phairplay_ffi.FfiVideoCodec
import uniffi.phairplay_ffi.FfiVideoFormat

/**
 * The translation of the objects of the bridge into the model of the shell.
 *
 * WHAT: the pure functions that turn the data transfer objects of the
 * bridge into the structures that the screens draw, and the configuration
 * of the receiver into the settings of the protocols.
 *
 * WHY: The screens must not know the DTOs of the bridge: a change of the
 * bridge would then ripple into every screen. The functions of this file
 * are the only place where the two vocabularies meet, and they are pure,
 * so that they can be tested without a device and without a library.
 *
 * EXAMPLE: `protocolPhase(enabled = false, state = FfiProtocolState.ADVERTISING)`
 * answers [ProtocolPhase.DISABLED], because a protocol that is switched
 * off is not advertised at all.
 */

/**
 * The phase of a protocol.
 *
 * @param enabled whether the protocol is switched on in the settings
 * @param state the state that the engine reported
 */
fun protocolPhase(enabled: Boolean, state: FfiProtocolState): ProtocolPhase {
    if (!enabled) {
        return ProtocolPhase.DISABLED
    }
    return when (state) {
        FfiProtocolState.IDLE -> ProtocolPhase.NOT_STARTED
        FfiProtocolState.STARTING -> ProtocolPhase.STARTING
        FfiProtocolState.ADVERTISING -> ProtocolPhase.ADVERTISING
        FfiProtocolState.STREAMING -> ProtocolPhase.STREAMING
        FfiProtocolState.STOPPING -> ProtocolPhase.STOPPING
        FfiProtocolState.STOPPED -> ProtocolPhase.STOPPED
        FfiProtocolState.FAILED -> ProtocolPhase.FAILED
    }
}

/**
 * The card of one protocol.
 *
 * @param kind the protocol the status belongs to
 * @param enabled whether the protocol is switched on in the settings
 * @param state the state that the engine reported
 * @param tcpPort the TCP port the engine listens on
 * @param udpPort the UDP port the engine listens on
 * @param peerCount the number of senders that are connected
 * @param lastError the reason of the last failure
 */
fun protocolCard(
    kind: FfiProtocolKind,
    enabled: Boolean,
    state: FfiProtocolState,
    tcpPort: UShort?,
    udpPort: UShort?,
    peerCount: UInt,
    lastError: String?,
): ProtocolCardState? {
    val protocol = protocolIdOf(kind) ?: return null
    return ProtocolCardState(
        protocol = protocol,
        phase = protocolPhase(enabled, state),
        enabled = enabled,
        tcpPort = tcpPort,
        udpPort = udpPort,
        peerCount = peerCount,
        lastError = lastError,
    )
}

/**
 * The protocol of a kind of the bridge.
 *
 * WHY: The enumeration of the bridge and the one of the shell have the
 * same members. The function is written anyway, so that a new member of
 * the bridge cannot reach the screens unnoticed: the shell would leave the
 * card out instead of drawing a protocol that it cannot name.
 */
fun protocolIdOf(kind: FfiProtocolKind): ProtocolId? = when (kind) {
    FfiProtocolKind.AIR_PLAY -> ProtocolId.AIRPLAY
    FfiProtocolKind.MIRACAST -> ProtocolId.MIRACAST
    FfiProtocolKind.CAST -> ProtocolId.CAST
    FfiProtocolKind.DLNA -> ProtocolId.DLNA
}

/**
 * Whether a protocol is switched on in the configuration.
 *
 * WHY: The core treats a protocol that is not mentioned in the
 * configuration as switched on, because the default of the protocol is
 * "enabled". The shell has to read the configuration the same way, or the
 * switch of the card and the behaviour of the engine would disagree.
 */
fun FfiAppConfig.isProtocolEnabled(bridgeName: String): Boolean =
    protocols.firstOrNull { it.name == bridgeName }?.enabled ?: true

/**
 * The configuration in which one protocol is switched on or off.
 *
 * WHY: The configuration is the only place where the user can change a
 * protocol without touching the source. A protocol that is not mentioned
 * in it yet is added with the default of the core: no pinned port, no
 * PIN, and a limit of one megabyte for a single message.
 */
fun FfiAppConfig.withProtocolEnabled(bridgeName: String, enabled: Boolean): FfiAppConfig {
    val existing = protocols.firstOrNull { it.name == bridgeName }
    val entry = existing ?: FfiProtocolConfig(
        name = bridgeName,
        enabled = enabled,
        port = 0,
        requirePin = false,
        maxMessageBytes = DEFAULT_MAX_MESSAGE_BYTES,
    )
    val updated = entry.copy(enabled = enabled)
    val remaining = protocols.filterNot { it.name == bridgeName }
    return copy(protocols = remaining + updated)
}

/**
 * The default limit of a single protocol message, in bytes.
 *
 * WHY: The value is the default of the core (`ProtocolConfig`). The shell
 * repeats it here because the configuration of the bridge carries the
 * value as a number and not as a reference to the core.
 */
const val DEFAULT_MAX_MESSAGE_BYTES: ULong = 1_048_576UL

/**
 * The session that is running, as the shell shows it.
 *
 * @param session the snapshot of the session
 */
fun nowPlayingOf(session: FfiSessionSnapshot): NowPlayingState = NowPlayingState(
    sessionId = session.id,
    protocol = protocolIdOf(session.protocol)?.bridgeName ?: session.protocol.name,
    senderName = session.senderName,
    senderAddress = session.senderAddress,
    video = videoDescription(session.video),
    audio = audioDescription(session.audio),
    title = null,
    artist = null,
    album = null,
    artwork = null,
    uptimeMs = session.uptimeMs,
)

/**
 * The description of a video stream, if the session carries one.
 *
 * WHY: The name of a codec and the size of a picture are the names of the
 * standards, and they are the same in every language. Only the sentence
 * around them belongs to the resources of the shell.
 */
fun videoDescription(format: FfiVideoFormat?): String? {
    if (format == null) {
        return null
    }
    val codec = videoCodecName(format.codec)
    val size = "${format.width}x${format.height}"
    val frameRate = format.frameRate?.let { " @${it}" } ?: ""
    return "$codec $size$frameRate"
}

/**
 * The description of an audio stream, if the session carries one.
 */
fun audioDescription(format: FfiAudioFormat?): String? {
    if (format == null) {
        return null
    }
    val codec = audioCodecName(format.codec)
    val channels = if (format.channels == 1.toUShort()) "mono" else "stereo"
    return "$codec ${format.sampleRate} Hz $channels"
}

/**
 * The name of a video codec.
 */
fun videoCodecName(codec: FfiVideoCodec): String = when (codec) {
    FfiVideoCodec.H264 -> "H.264"
    FfiVideoCodec.HEVC -> "HEVC"
    FfiVideoCodec.VP8 -> "VP8"
    FfiVideoCodec.VP9 -> "VP9"
    FfiVideoCodec.AV1 -> "AV1"
}

/**
 * The name of an audio codec.
 */
fun audioCodecName(codec: FfiAudioCodec): String = when (codec) {
    FfiAudioCodec.AAC_LC -> "AAC-LC"
    FfiAudioCodec.AAC_ELD -> "AAC-ELD"
    FfiAudioCodec.ALAC -> "ALAC"
    FfiAudioCodec.LPCM -> "LPCM"
    FfiAudioCodec.OPUS -> "Opus"
    FfiAudioCodec.MP3 -> "MP3"
    FfiAudioCodec.FLAC -> "FLAC"
}
