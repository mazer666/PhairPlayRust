/**
 * The English language file of the dashboard.
 *
 * WHAT: every text the dashboard shows to a person — the labels of the
 * header, the sentences of the cards, the wording of the states, the names
 * of the protocols and the units of the counters.
 * WHY: a text that lives in the markup is a text that cannot be translated,
 * cannot be measured and drifts between the panels. The language file is
 * the one place a wording exists; the markup carries the key, the module
 * carries the string.
 * HOW: a flat map with dotted keys. `i18n.js` reads it, replaces the
 * placeholders `{name}` and writes the result into the document. A second
 * language is a second file with the same keys — `lang/de.js` is that
 * file. A key that is missing in a language falls back to English, so a
 * half-finished translation never blanks the panel.
 *
 * EXAMPLE:
 *
 * ```js
 * import { t } from "./i18n.js";
 *
 * t("card.switch.on");                  // "On"
 * t("session.count", { count: 2 });     // "2 sessions streaming."
 * ```
 */

export const MESSAGES = {
  /* ── The document ────────────────────────────────────────────────────── */
  "app.title": "PhairPlayRust — Receiver Dashboard",
  "app.description":
    "Local control panel for the PhairPlayRust receiver: AirPlay 2, Miracast, Google Cast and DLNA on your own network.",
  "a11y.skipToDashboard": "Skip to the dashboard",
  "a11y.protocolGroup": "The four receivers of this device",
  "a11y.noScript":
    "The live state of the receivers is read from the local API and needs JavaScript.",

  /* ── The header ──────────────────────────────────────────────────────── */
  "brand.tagline": "AirPlay 2 · Miracast · Google Cast · DLNA — on your network only",
  "brand.taglineWithDevice": "{name} · AirPlay 2 · Miracast · Google Cast · DLNA — on your network only",
  "header.version": "Version",
  "header.commit": "Commit",
  "header.target": "Target",
  "header.uptime": "Uptime",
  "header.theme.toLight": "Switch to the light theme",
  "header.theme.toDark": "Switch to the dark theme",
  "header.connection.connecting": "Connecting",
  "header.connection.live": "Live",
  "header.connection.polling": "Polling",
  "header.connection.offline": "Offline",

  /* ── The banner ──────────────────────────────────────────────────────── */
  "error.apiUnreachable":
    "The dashboard cannot reach the local API. Start the receiver and reload this page.",
  "error.apiUnreachableWithReason":
    "The dashboard cannot reach the local API ({reason}). Start the receiver and reload this page.",
  "error.toggleFailed": "The receiver could not be switched {state}: {reason}",
  "error.switchedOn": "on",
  "error.switchedOff": "off",
  "error.dismiss": "Dismiss the message",
  "error.bannerLabel": "Problem with the local receiver",

  /* ── The sections ────────────────────────────────────────────────────── */
  "section.receivers.title": "Receivers",
  "section.receivers.note":
    "Every receiver advertises itself on the local network. The badge states what the service is doing right now — a service that is switched off in the configuration reads Disabled, a service that failed reads Error.",
  "section.sessions.title": "Live sessions",
  "section.sessions.empty": "No session is streaming.",
  "section.chart.title": "Throughput",
  "section.chart.note":
    "Frames per second of the running session, over the last minutes.",
  "section.codecs.title": "Codec reference",
  "section.codecs.note":
    "The media vocabulary the receivers speak. What the device itself can decode depends on its platform decoders — the receiver advertises that in the network announcement.",

  /* ── The remote control ──────────────────────────────────────────────── */
  "remote.hint":
    "With the arrow keys you move, with OK or Enter you switch a receiver on or off.",
  "remote.hintLabel": "Operation with a remote control",

  /* ── The protocol cards ──────────────────────────────────────────────── */
  "card.port": "Port",
  "card.peers": "Peers",
  "card.frames": "Frames",
  "card.bytes": "Bytes",
  "card.loss": "Packet loss",
  "card.udpPort": "· UDP {port}",
  "card.switch.on": "On",
  "card.switch.off": "Off",
  "card.action": "{name} — switch this receiver on or off",

  /* ── The states of a receiver ────────────────────────────────────────── */
  "state.idle.label": "Idle",
  "state.idle.reason": "The receiver is loaded but not started.",
  "state.starting.label": "Starting",
  "state.starting.reason": "Binding sockets and registering the network service.",
  "state.advertising.label": "Advertising",
  "state.advertising.reason": "Visible to senders on the local network, waiting for a connection.",
  "state.streaming.label": "Streaming",
  "state.streaming.reason": "A sender is connected and the stream is running.",
  "state.stopping.label": "Stopping",
  "state.stopping.reason": "The peer connection is being torn down.",
  "state.stopped.label": "Stopped",
  "state.stopped.reason": "Enabled in the configuration, but the service is not running.",
  "state.failed.label": "Error",
  "state.failed.reason": "The receiver could not be started.",
  "state.disabled.label": "Disabled",
  "state.disabled.reason": "Switched off in the configuration — the service is not advertised.",
  "state.unknown.label": "Unknown",
  "state.unknown.reason": "The receiver did not report a state.",
  "state.lastError": "{reason} Last error: {error}",

  /* ── The live sessions ───────────────────────────────────────────────── */
  "session.count": "{count} session streaming.",
  "session.countPlural": "{count} sessions streaming.",
  "session.none": "No sender is connected.",
  "session.empty": "Nothing is streaming at the moment.",
  "session.unknownSender": "Unknown sender",
  "session.unknownProtocol": "unknown",
  "session.running": "running {duration}",
  "session.frames": "{count} frames",
  "session.received": "{size} received",
  "session.lost": "{count} packets lost",

  /* ── The graph ───────────────────────────────────────────────────────── */
  "chart.groupLabel": "Metric of the graph",
  "chart.metric.fps": "Frames / s",
  "chart.metric.bytes": "Throughput",
  "chart.scopeLabel": "Scope",
  "chart.scope.all": "All receivers",
  "chart.description":
    "A line graph of the selected metric over the last minutes of the running session. The current value is written out below the graph.",
  "chart.readout.current": "Current",
  "chart.readout.peak": "Peak",
  "chart.readout.average": "Average",
  "chart.empty": "No measurement yet",
  "chart.note.fps": "Frames per second of the running session, derived from the counters of the receiver.",
  "chart.note.bytes": "Bytes per second of the running session, derived from the counters of the receiver.",

  /* ── The codec reference ─────────────────────────────────────────────── */
  "table.codec": "Codec",
  "table.type": "Type",
  "table.mime": "MIME type",
  "table.usedBy": "Used by",
  "table.caption":
    "Codecs the PhairPlayRust receivers understand, with their MIME type and the protocols that use them.",
  "codec.kind.video": "Video",
  "codec.kind.audio": "Audio",
  "codec.kind.passthrough": "Pass-through",
  "codec.name.h264": "H.264 / AVC",
  "codec.name.hevc": "HEVC / H.265",
  "codec.name.vp8": "VP8",
  "codec.name.vp9": "VP9",
  "codec.name.av1": "AV1",
  "codec.name.aacLc": "AAC-LC",
  "codec.name.aacEld": "AAC-ELD",
  "codec.name.alac": "Apple Lossless",
  "codec.name.lpcm": "Linear PCM",
  "codec.name.opus": "Opus",
  "codec.name.mp3": "MP3",
  "codec.name.flac": "FLAC",
  "codec.name.ac3": "Dolby Digital",
  "codec.name.eac3": "Dolby Digital Plus",
  "codec.where.h264": "AirPlay 2, Miracast, Google Cast",
  "codec.where.hevc": "AirPlay 2, Google Cast — if the device decodes it",
  "codec.where.vp8": "Google Cast",
  "codec.where.vp9": "Google Cast",
  "codec.where.av1": "Google Cast",
  "codec.where.aacLc": "AirPlay 2, Miracast, Google Cast, DLNA",
  "codec.where.aacEld": "AirPlay 2",
  "codec.where.alac": "AirPlay 2",
  "codec.where.lpcm": "Miracast, Google Cast, DLNA",
  "codec.where.opus": "DLNA, Google Cast",
  "codec.where.mp3": "Google Cast, DLNA",
  "codec.where.flac": "DLNA",
  "codec.where.ac3": "Devices with a pass-through output",
  "codec.where.eac3": "Devices with a pass-through output",

  /* ── The names of the protocols ──────────────────────────────────────── */
  "protocol.airplay": "AirPlay 2",
  "protocol.miracast": "Miracast",
  "protocol.cast": "Google Cast",
  "protocol.dlna": "DLNA / UPnP",

  /* ── The units of the counters ───────────────────────────────────────── */
  "unit.byte": "B",
  "unit.kilobyte": "kB",
  "unit.megabyte": "MB",
  "unit.gigabyte": "GB",
  "unit.terabyte": "TB",
  "unit.petabyte": "PB",
  "unit.fps": "fps",
  "unit.hour": "h",
  "unit.minute": "m",
  "unit.second": "s",
  "value.unknown": "—",

  /* ── The footer ──────────────────────────────────────────────────────── */
  "footer.license": "PhairPlayRust — AGPL-3.0-or-later. No account, no advertising, no telemetry.",
  "footer.local": "The dashboard talks only to the local receiver on this machine.",
  "footer.language": "Language",
};

export default MESSAGES;
