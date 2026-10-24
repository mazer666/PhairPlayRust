/**
 * The German language file of the dashboard.
 *
 * WHAT: the German wording of every text the dashboard shows.
 * WHY: the dashboard is the universal interface of the project — it runs on
 * the television, on a tablet and on a phone. A user who reads German must
 * not have to read an English panel to switch a receiver off.
 * HOW: the same keys as `lang/en.js`; a key that is missing here falls back
 * to English, so this file can grow without breaking the panel.
 *
 * EXAMPLE:
 *
 * ```js
 * import { t } from "./i18n.js";
 *
 * t("state.streaming.label");   // "Streaming"
 * t("card.switch.on");          // "An"
 * ```
 */

export const MESSAGES = {
  /* ── Das Dokument ────────────────────────────────────────────────────── */
  "app.title": "PhairPlayRust — Empfänger-Dashboard",
  "app.description":
    "Lokale Bedienoberfläche für den PhairPlayRust-Empfänger: AirPlay 2, Miracast, Google Cast und DLNA — ausschließlich im eigenen Netzwerk.",
  "a11y.skipToDashboard": "Zum Dashboard springen",
  "a11y.protocolGroup": "Die vier Empfänger dieses Geräts",
  "a11y.noScript":
    "Der aktuelle Zustand der Empfänger wird aus der lokalen API gelesen und benötigt JavaScript.",

  /* ── Die Kopfzeile ───────────────────────────────────────────────────── */
  "brand.tagline": "AirPlay 2 · Miracast · Google Cast · DLNA — nur in Ihrem Netzwerk",
  "brand.taglineWithDevice": "{name} · AirPlay 2 · Miracast · Google Cast · DLNA — nur in Ihrem Netzwerk",
  "header.version": "Version",
  "header.commit": "Commit",
  "header.target": "Ziel",
  "header.uptime": "Laufzeit",
  "header.theme.toLight": "Zum hellen Design wechseln",
  "header.theme.toDark": "Zum dunklen Design wechseln",
  "header.connection.connecting": "Verbinde",
  "header.connection.live": "Live",
  "header.connection.polling": "Abfrage",
  "header.connection.offline": "Offline",

  /* ── Das Fehlerbanner ────────────────────────────────────────────────── */
  "error.apiUnreachable":
    "Das Dashboard erreicht die lokale API nicht. Starten Sie den Empfänger und laden Sie diese Seite neu.",
  "error.apiUnreachableWithReason":
    "Das Dashboard erreicht die lokale API nicht ({reason}). Starten Sie den Empfänger und laden Sie diese Seite neu.",
  "error.toggleFailed": "Der Empfänger konnte nicht {state} geschaltet werden: {reason}",
  "error.switchedOn": "eingeschaltet",
  "error.switchedOff": "ausgeschaltet",
  "error.dismiss": "Meldung schließen",
  "error.bannerLabel": "Problem mit dem lokalen Empfänger",

  /* ── Die Abschnitte ──────────────────────────────────────────────────── */
  "section.receivers.title": "Empfänger",
  "section.receivers.note":
    "Jeder Empfänger meldet sich im lokalen Netzwerk an. Das Badge zeigt, was der Dienst gerade tut — ein im Konfiguration abgeschalteter Dienst liest Abgeschaltet, ein fehlgeschlagener Dienst liest Fehler.",
  "section.sessions.title": "Laufende Sitzungen",
  "section.sessions.empty": "Es streamt keine Sitzung.",
  "section.chart.title": "Durchsatz",
  "section.chart.note":
    "Bilder pro Sekunde der laufenden Sitzung, über die letzten Minuten.",
  "section.codecs.title": "Codec-Übersicht",
  "section.codecs.note":
    "Das Medienvokabular, das die Empfänger sprechen. Was das Gerät selbst dekodieren kann, hängt von seinen Plattform-Decodern ab — der Empfänger kündigt das in der Netzwerkmeldung an.",

  /* ── Die Fernbedienung ───────────────────────────────────────────────── */
  "remote.hint":
    "Mit den Pfeiltasten bewegen Sie sich, mit OK oder Enter schalten Sie einen Empfänger ein oder aus.",
  "remote.hintLabel": "Bedienung mit der Fernbedienung",

  /* ── Die Protokollkarten ─────────────────────────────────────────────── */
  "card.port": "Port",
  "card.peers": "Geräte",
  "card.frames": "Bilder",
  "card.bytes": "Bytes",
  "card.loss": "Verlust",
  "card.udpPort": "· UDP {port}",
  "card.switch.on": "An",
  "card.switch.off": "Aus",
  "card.action": "{name} — diesen Empfänger ein- oder ausschalten",

  /* ── Die Zustände eines Empfängers ──────────────────────────────────── */
  "state.idle.label": "Bereit",
  "state.idle.reason": "Der Empfänger ist geladen, aber nicht gestartet.",
  "state.starting.label": "Startet",
  "state.starting.reason": "Bindet die Sockets und meldet den Netzwerkdienst an.",
  "state.advertising.label": "Angekündigt",
  "state.advertising.reason": "Für Sender im lokalen Netzwerk sichtbar, wartet auf eine Verbindung.",
  "state.streaming.label": "Streaming",
  "state.streaming.reason": "Ein Sender ist verbunden und der Stream läuft.",
  "state.stopping.label": "Stoppt",
  "state.stopping.reason": "Die Verbindung zum Sender wird abgebaut.",
  "state.stopped.label": "Gestoppt",
  "state.stopped.reason": "In der Konfiguration aktiv, der Dienst läuft aber nicht.",
  "state.failed.label": "Fehler",
  "state.failed.reason": "Der Empfänger konnte nicht gestartet werden.",
  "state.disabled.label": "Abgeschaltet",
  "state.disabled.reason": "In der Konfiguration ausgeschaltet — der Dienst wird nicht angekündigt.",
  "state.unknown.label": "Unbekannt",
  "state.unknown.reason": "Der Empfänger hat keinen Zustand gemeldet.",
  "state.lastError": "{reason} Letzter Fehler: {error}",

  /* ── Die laufenden Sitzungen ─────────────────────────────────────────── */
  "session.count": "{count} Sitzung streamt.",
  "session.countPlural": "{count} Sitzungen streamen.",
  "session.none": "Es ist kein Sender verbunden.",
  "session.empty": "Momentan streamt nichts.",
  "session.unknownSender": "Unbekannter Sender",
  "session.unknownProtocol": "unbekannt",
  "session.running": "läuft seit {duration}",
  "session.frames": "{count} Bilder",
  "session.received": "{size} empfangen",
  "session.lost": "{count} Pakete verloren",

  /* ── Die Grafik ──────────────────────────────────────────────────────── */
  "chart.groupLabel": "Messwert der Grafik",
  "chart.metric.fps": "Bilder / s",
  "chart.metric.bytes": "Durchsatz",
  "chart.scopeLabel": "Umfang",
  "chart.scope.all": "Alle Empfänger",
  "chart.description":
    "Eine Linie des gewählten Messwerts über die letzten Minuten der laufenden Sitzung. Der aktuelle Wert steht unter der Grafik.",
  "chart.readout.current": "Aktuell",
  "chart.readout.peak": "Spitze",
  "chart.readout.average": "Mittel",
  "chart.empty": "Noch keine Messung",
  "chart.note.fps": "Bilder pro Sekunde der laufenden Sitzung, abgeleitet aus den Zählern des Empfängers.",
  "chart.note.bytes": "Bytes pro Sekunde der laufenden Sitzung, abgeleitet aus den Zählern des Empfängers.",

  /* ── Die Codec-Übersicht ─────────────────────────────────────────────── */
  "table.codec": "Codec",
  "table.type": "Art",
  "table.mime": "MIME-Typ",
  "table.usedBy": "Verwendet von",
  "table.caption":
    "Codecs, die die PhairPlayRust-Empfänger verstehen, mit MIME-Typ und den Protokollen, die sie nutzen.",
  "codec.kind.video": "Video",
  "codec.kind.audio": "Audio",
  "codec.kind.passthrough": "Durchleitung",
  "codec.name.h264": "H.264 / AVC",
  "codec.name.hevc": "HEVC / H.265",
  "codec.name.vp8": "VP8",
  "codec.name.vp9": "VP9",
  "codec.name.av1": "AV1",
  "codec.name.aacLc": "AAC-LC",
  "codec.name.aacEld": "AAC-ELD",
  "codec.name.alac": "Apple Lossless",
  "codec.name.lpcm": "Lineares PCM",
  "codec.name.opus": "Opus",
  "codec.name.mp3": "MP3",
  "codec.name.flac": "FLAC",
  "codec.name.ac3": "Dolby Digital",
  "codec.name.eac3": "Dolby Digital Plus",
  "codec.where.h264": "AirPlay 2, Miracast, Google Cast",
  "codec.where.hevc": "AirPlay 2, Google Cast — wenn das Gerät dekodieren kann",
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
  "codec.where.ac3": "Geräte mit Durchleitungsausgang",
  "codec.where.eac3": "Geräte mit Durchleitungsausgang",

  /* ── Die Namen der Protokolle ───────────────────────────────────────── */
  "protocol.airplay": "AirPlay 2",
  "protocol.miracast": "Miracast",
  "protocol.cast": "Google Cast",
  "protocol.dlna": "DLNA / UPnP",

  /* ── Die Einheiten der Zähler ────────────────────────────────────────── */
  "unit.byte": "B",
  "unit.kilobyte": "kB",
  "unit.megabyte": "MB",
  "unit.gigabyte": "GB",
  "unit.terabyte": "TB",
  "unit.petabyte": "PB",
  "unit.fps": "fps",
  "unit.hour": "Std.",
  "unit.minute": "Min.",
  "unit.second": "s",
  "value.unknown": "—",

  /* ── Die Fußzeile ────────────────────────────────────────────────────── */
  "footer.license": "PhairPlayRust — AGPL-3.0-or-later. Kein Konto, keine Werbung, kein Telemetrie.",
  "footer.local": "Das Dashboard spricht nur mit dem lokalen Empfänger auf diesem Rechner.",
  "footer.language": "Sprache",
};

export default MESSAGES;
