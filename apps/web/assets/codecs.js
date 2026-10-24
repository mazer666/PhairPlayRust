/**
 * The codec reference of the dashboard.
 *
 * WHAT: the media vocabulary of PhairPlayRust and the rendering of the
 * reference table.
 * WHY: a user who wonders "why does my phone not see a picture" needs to
 * know which codec the receiver speaks. The table names the codec, its MIME
 * type and the protocols that need it.
 * HOW: the vocabulary is taken from `phairplay_core::capabilities` and from
 * the baselines of the core. What the *device* can decode is not in the
 * table: it depends on the decoders of the platform, and the receiver
 * advertises it in the network announcement. An endpoint that reports the
 * capabilities is documented as a request in the deliverable; the panel does
 * not call an endpoint that does not exist, so that the log of the browser
 * stays free of errors.
 */

/**
 * The codecs the project knows, with the protocol that needs them.
 *
 * @type {Array<{key: string, name: string, kind: string, mime: string, where: string}>}
 */
export const CODECS = [
  {
    key: "h264",
    name: "H.264 / AVC",
    kind: "Video",
    mime: "video/avc",
    where: "AirPlay 2, Miracast, Google Cast",
  },
  {
    key: "hevc",
    name: "HEVC / H.265",
    kind: "Video",
    mime: "video/hevc",
    where: "AirPlay 2, Google Cast — if the device decodes it",
  },
  {
    key: "vp8",
    name: "VP8",
    kind: "Video",
    mime: "video/x-vnd.on2.vp8",
    where: "Google Cast",
  },
  {
    key: "vp9",
    name: "VP9",
    kind: "Video",
    mime: "video/x-vnd.on2.vp9",
    where: "Google Cast",
  },
  {
    key: "av1",
    name: "AV1",
    kind: "Video",
    mime: "video/av01",
    where: "Google Cast",
  },
  {
    key: "aac_lc",
    name: "AAC-LC",
    kind: "Audio",
    mime: "audio/mp4a-latm",
    where: "AirPlay 2, Miracast, Google Cast, DLNA",
  },
  {
    key: "aac_eld",
    name: "AAC-ELD",
    kind: "Audio",
    mime: "audio/mp4a-latm",
    where: "AirPlay 2",
  },
  {
    key: "alac",
    name: "Apple Lossless",
    kind: "Audio",
    mime: "audio/alac",
    where: "AirPlay 2",
  },
  {
    key: "lpcm",
    name: "Linear PCM",
    kind: "Audio",
    mime: "audio/raw",
    where: "Miracast, Google Cast, DLNA",
  },
  {
    key: "opus",
    name: "Opus",
    kind: "Audio",
    mime: "audio/opus",
    where: "DLNA, Google Cast",
  },
  {
    key: "mp3",
    name: "MP3",
    kind: "Audio",
    mime: "audio/mpeg",
    where: "Google Cast, DLNA",
  },
  {
    key: "flac",
    name: "FLAC",
    kind: "Audio",
    mime: "audio/flac",
    where: "DLNA",
  },
  {
    key: "ac3",
    name: "Dolby Digital",
    kind: "Pass-through",
    mime: "audio/ac3",
    where: "Devices with a pass-through output",
  },
  {
    key: "eac3",
    name: "Dolby Digital Plus",
    kind: "Pass-through",
    mime: "audio/eac3",
    where: "Devices with a pass-through output",
  },
];

/**
 * Paints the codec reference table.
 */
export function renderCodecs() {
  const body = document.getElementById("codec-table-body");
  body.textContent = "";
  for (const codec of CODECS) {
    const row = document.createElement("tr");
    for (const text of [codec.name, codec.kind, codec.mime, codec.where]) {
      const cell = document.createElement("td");
      cell.textContent = text;
      row.append(cell);
    }
    body.append(row);
  }
}
