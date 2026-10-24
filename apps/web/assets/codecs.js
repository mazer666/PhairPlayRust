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
 * stays free of errors. The wording comes from the language file, the MIME
 * types do not — they are a protocol fact.
 */

import { t } from "./i18n.js";

/**
 * The codecs the project knows, with the protocol that needs them.
 *
 * @type {Array<{key: string, kind: string, mime: string, where: string}>}
 */
export const CODECS = [
  { key: "h264", kind: "video", mime: "video/avc", where: "h264" },
  { key: "hevc", kind: "video", mime: "video/hevc", where: "hevc" },
  { key: "vp8", kind: "video", mime: "video/x-vnd.on2.vp8", where: "vp8" },
  { key: "vp9", kind: "video", mime: "video/x-vnd.on2.vp9", where: "vp9" },
  { key: "av1", kind: "video", mime: "video/av01", where: "av1" },
  { key: "aacLc", kind: "audio", mime: "audio/mp4a-latm", where: "aacLc" },
  { key: "aacEld", kind: "audio", mime: "audio/mp4a-latm", where: "aacEld" },
  { key: "alac", kind: "audio", mime: "audio/alac", where: "alac" },
  { key: "lpcm", kind: "audio", mime: "audio/raw", where: "lpcm" },
  { key: "opus", kind: "audio", mime: "audio/opus", where: "opus" },
  { key: "mp3", kind: "audio", mime: "audio/mpeg", where: "mp3" },
  { key: "flac", kind: "audio", mime: "audio/flac", where: "flac" },
  { key: "ac3", kind: "passthrough", mime: "audio/ac3", where: "ac3" },
  { key: "eac3", kind: "passthrough", mime: "audio/eac3", where: "eac3" },
];

/**
 * Paints the codec reference table.
 */
export function renderCodecs() {
  const body = document.getElementById("codec-table-body");
  body.textContent = "";
  for (const codec of CODECS) {
    const row = document.createElement("tr");
    const cells = [
      t(`codec.name.${codec.key}`),
      t(`codec.kind.${codec.kind}`),
      codec.mime,
      t(`codec.where.${codec.where}`),
    ];
    for (const text of cells) {
      const cell = document.createElement("td");
      cell.textContent = text;
      row.append(cell);
    }
    body.append(row);
  }
}
