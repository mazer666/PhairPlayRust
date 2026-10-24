package org.phairplay.receiver.platform

import android.content.Context
import android.content.pm.PackageManager
import android.media.MediaCodecInfo
import android.media.MediaCodecList
import android.os.Build
import uniffi.phairplay_ffi.FfiAudioCodec
import uniffi.phairplay_ffi.FfiMediaCapabilities
import uniffi.phairplay_ffi.FfiVideoCodec

/**
 * The capabilities of the device the shell runs on.
 *
 * WHAT: the question "what can this device decode?" and the question "does
 * this device have the radio that Miracast needs?".
 *
 * WHY: The receiver advertises the codecs that it can really decode. A
 * receiver that advertises a codec that the device cannot decode produces
 * a picture that never appears on the television — and the sender believes
 * that the picture is coming. The shell therefore asks the platform and
 * hands the answer to the bridge.
 *
 * The same applies to Miracast: the engine needs the Wi-Fi Direct radio of
 * the system. A device without that radio cannot run Miracast at all, and
 * the user has to see the reason instead of a service that never appears
 * (FR-035).
 *
 * EXAMPLE: `capabilities()` answers the codecs of the `MediaCodecList` of
 * the device; `hasWifiDirect()` answers whether the radio is there.
 */
class PlatformCapabilities(private val context: Context) {

    /**
     * The codecs that the device can decode.
     *
     * WHY: The list of the decoders of the platform is the truth. A codec
     * that is not in the list is not advertised, even if the core would
     * accept it.
     */
    fun capabilities(): FfiMediaCapabilities {
        val video = mutableListOf<FfiVideoCodec>()
        val audio = mutableListOf<FfiAudioCodec>()
        for (info in decoders()) {
            if (info.isEncoder) {
                continue
            }
            for (type in info.supportedTypes) {
                val videoCodec = videoCodecOf(type)
                if (videoCodec != null && !video.contains(videoCodec)) {
                    video.add(videoCodec)
                }
                val audioCodec = audioCodecOf(type)
                if (audioCodec != null && !audio.contains(audioCodec)) {
                    audio.add(audioCodec)
                }
            }
        }
        return FfiMediaCapabilities(
            video = video,
            audio = audio,
            // WHY: the pass-through of Dolby Digital needs a path to a
            // capable output device. The shell of this milestone has no
            // such path, and a codec that is advertised without one would
            // be a promise that the shell cannot keep.
            passthrough = emptyList(),
        )
    }

    /**
     * Whether the device has the Wi-Fi Direct radio of Miracast.
     */
    fun hasWifiDirect(): Boolean =
        context.packageManager.hasSystemFeature(PackageManager.FEATURE_WIFI_DIRECT)

    /**
     * Whether the device has a Wi-Fi radio at all.
     */
    fun hasWifi(): Boolean =
        context.packageManager.hasSystemFeature(PackageManager.FEATURE_WIFI)

    /**
     * The name that the platform reports for the device.
     *
     * WHY: A sender must be able to tell two receivers of the same
     * network apart. The model of the device is the name that a user
     * recognises; the name of the defaults file is only the fallback for a
     * device that does not report one.
     */
    fun deviceName(): String? = Build.MODEL?.takeIf { it.isNotBlank() }

    /**
     * The decoders of the platform.
     */
    private fun decoders(): List<MediaCodecInfo> {
        val list = MediaCodecList(MediaCodecList.REGULAR_CODECS)
        return list.codecInfos.toList()
    }

    /**
     * The video codec of a media type, if the type is a video codec.
     */
    private fun videoCodecOf(type: String): FfiVideoCodec? = when (type) {
        "video/avc" -> FfiVideoCodec.H264
        "video/hevc" -> FfiVideoCodec.HEVC
        "video/x-vnd.on2.vp8" -> FfiVideoCodec.VP8
        "video/x-vnd.on2.vp9" -> FfiVideoCodec.VP9
        "video/av01" -> FfiVideoCodec.AV1
        else -> null
    }

    /**
     * The audio codec of a media type, if the type is an audio codec.
     */
    private fun audioCodecOf(type: String): FfiAudioCodec? = when (type) {
        "audio/mp4a-latm" -> FfiAudioCodec.AAC_LC
        "audio/alac" -> FfiAudioCodec.ALAC
        "audio/raw" -> FfiAudioCodec.LPCM
        "audio/opus" -> FfiAudioCodec.OPUS
        "audio/mpeg" -> FfiAudioCodec.MP3
        "audio/flac" -> FfiAudioCodec.FLAC
        else -> null
    }
}
