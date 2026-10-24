/**
 * Normalisation of the answers of the local receiver API.
 *
 * WHAT: the translation of the JSON of the server into the model the panel
 * renders.
 * WHY: the panel must show what the receiver actually reports. The shapes
 * below are the ones of `apps/web/src/dto.rs` — the server is the truth, not
 * this file. Everything that is read tolerantly is documented as such.
 * HOW: every reader accepts the field name of the server and one older
 * variant, and returns `null` when nothing matches. The canonical shape of
 * every answer is documented in the deliverable of the dashboard work.
 */

/**
 * Reads the first defined value from a list of candidate field names.
 *
 * @param {Record<string, unknown>} source
 * @param {string[]} names
 * @returns {unknown}
 */
export function pick(source, names) {
  for (const name of names) {
    if (source && source[name] !== undefined && source[name] !== null) return source[name];
  }
  return undefined;
}

/**
 * Reads a numeric field, tolerating strings.
 *
 * @param {Record<string, unknown>} source
 * @param {string[]} names
 * @returns {number|null}
 */
export function pickNumber(source, names) {
  const raw = pick(source, names);
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string" && raw.trim() !== "") {
    const parsed = Number(raw);
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

/**
 * Reads a string field.
 *
 * @param {Record<string, unknown>} source
 * @param {string[]} names
 * @returns {string|null}
 */
export function pickString(source, names) {
  const raw = pick(source, names);
  if (typeof raw === "string" && raw.trim() !== "") return raw;
  if (typeof raw === "number") return String(raw);
  return null;
}

/**
 * Reads a boolean field, tolerating "true"/"false"/0/1.
 *
 * @param {Record<string, unknown>} source
 * @param {string[]} names
 * @returns {boolean|null}
 */
export function pickBoolean(source, names) {
  const raw = pick(source, names);
  if (typeof raw === "boolean") return raw;
  if (raw === "true" || raw === 1) return true;
  if (raw === "false" || raw === 0) return false;
  return null;
}

/**
 * Normalises a protocol status entry.
 *
 * The server sends `ProtocolStatusDto` (see `apps/web/src/dto.rs`): the
 * fields `protocol`, `display_name`, `state`, `enabled`, `tcp_port`,
 * `udp_port`, `peer_count` and `last_error`.
 *
 * @param {Record<string, unknown>} raw
 * @param {string} fallbackId
 * @returns {{id: string, name: string|null, state: string, enabled: boolean,
 *   tcpPort: number|null, udpPort: number|null, peers: number,
 *   lastError: string|null}}
 */
export function normalizeProtocol(raw, fallbackId) {
  const source = raw && typeof raw === "object" ? raw : {};
  const id = (pickString(source, ["protocol", "kind", "id", "name"]) || fallbackId).toLowerCase();
  const state = (pickString(source, ["state", "status", "phase"]) || "unknown").toLowerCase();
  const enabled = pickBoolean(source, ["enabled", "is_enabled", "active"]);
  return {
    id,
    name: pickString(source, ["display_name", "displayName", "title"]),
    state,
    enabled: enabled === null ? state !== "disabled" : enabled,
    tcpPort: pickNumber(source, ["tcp_port", "tcpPort", "port"]),
    udpPort: pickNumber(source, ["udp_port", "udpPort"]),
    peers: pickNumber(source, ["peer_count", "peerCount", "peers", "clients"]) ?? 0,
    lastError: pickString(source, ["last_error", "lastError", "error", "message"]),
  };
}

/**
 * Normalises the answer of `GET /api/status`.
 *
 * The server sends `StatusResponse`: the list of the protocols, the number of
 * live sessions and how long the receiver has been running.
 *
 * @param {unknown} payload
 * @returns {{device: {name: string|null, model: string|null}, protocols: object[],
 *   activeSessions: number|null, uptimeMs: number|null}}
 */
export function normalizeStatus(payload) {
  const root = payload && typeof payload === "object" ? payload : {};
  const body = root.status && typeof root.status === "object" ? root.status : root;
  const deviceSource = body.device && typeof body.device === "object" ? body.device : {};
  const protocols = [];
  const raw = body.protocols ?? body.receivers ?? body.protocol_status;
  if (Array.isArray(raw)) {
    for (const entry of raw) protocols.push(normalizeProtocol(entry, ""));
  } else if (raw && typeof raw === "object") {
    for (const [key, entry] of Object.entries(raw)) {
      protocols.push(normalizeProtocol(entry, key));
    }
  }
  return {
    device: {
      name: pickString(deviceSource, ["name", "device_name", "friendly_name"]),
      model: pickString(deviceSource, ["model", "device_model"]),
    },
    protocols,
    activeSessions: pickNumber(body, ["active_sessions", "activeSessions", "sessions"]),
    uptimeMs: pickNumber(body, ["uptime_ms", "uptimeMs"]),
  };
}

/**
 * Describes the media of a session.
 *
 * WHY two shapes: the server sends the negotiated media as one ready-made
 * string ("video/avc 1920x1080"), because the dashboard only shows it. The
 * object form is accepted as well, so a server that reports the parts
 * separately is rendered with the same words.
 *
 * @param {unknown} raw
 * @param {"video"|"audio"} kind
 * @returns {string|null}
 */
function describeMedia(raw, kind) {
  if (typeof raw === "string" && raw.trim() !== "") return raw.trim();
  if (!raw || typeof raw !== "object") return null;
  const source = raw;
  const parts = [];
  const codec = pickString(source, ["codec", "name", "type"]);
  if (codec) parts.push(codec.toUpperCase());
  if (kind === "video") {
    const width = pickNumber(source, ["width"]);
    const height = pickNumber(source, ["height"]);
    if (width && height) parts.push(`${width}×${height}`);
    const frameRate = pickNumber(source, ["frame_rate", "frameRate", "fps"]);
    if (frameRate) parts.push(`${frameRate} fps`);
  } else {
    const sampleRate = pickNumber(source, ["sample_rate", "sampleRate"]);
    if (sampleRate) parts.push(`${sampleRate} Hz`);
    const channels = pickNumber(source, ["channels"]);
    if (channels) parts.push(`${channels} ch`);
  }
  return parts.length > 0 ? parts.join(" ") : null;
}

/**
 * Normalises one session entry.
 *
 * The server sends `SessionDto`: the numeric id, the protocol, the name and
 * the address of the sender, the negotiated media as strings and how long
 * the session has been running. It carries **no** counters — those live in
 * `GET /api/metrics` and belong to the protocol, not to the session.
 *
 * @param {Record<string, unknown>} raw
 * @returns {object}
 */
function normalizeSession(raw) {
  const source = raw && typeof raw === "object" ? raw : {};
  const id = pick(source, ["id", "session_id", "sessionId"]);
  const uptimeMs = pickNumber(source, ["uptime_ms", "uptimeMs", "duration_ms"]);
  const startedRaw = pick(source, ["started_at", "startedAt", "started", "start_time"]);
  let startedAt = null;
  if (typeof startedRaw === "number") {
    startedAt = startedRaw > 1e12 ? startedRaw : startedRaw * 1000;
  } else if (typeof startedRaw === "string") {
    const parsed = Date.parse(startedRaw);
    if (!Number.isNaN(parsed)) startedAt = parsed;
  }
  return {
    id: id === undefined || id === null ? "—" : String(id),
    protocol: (pickString(source, ["protocol", "kind"]) || "").toLowerCase(),
    senderName: pickString(source, ["sender_name", "senderName", "name", "device", "device_name"]),
    senderAddress: pickString(source, ["sender_address", "senderAddress", "address", "ip", "host"]),
    video: describeMedia(pick(source, ["video", "video_format", "videoFormat"]), "video"),
    audio: describeMedia(pick(source, ["audio", "audio_format", "audioFormat"]), "audio"),
    startedAt,
    uptimeMs,
  };
}

/**
 * Normalises the answer of `GET /api/sessions`.
 *
 * @param {unknown} payload
 * @returns {object[]}
 */
export function normalizeSessions(payload) {
  const root = payload && typeof payload === "object" ? payload : {};
  const list = Array.isArray(payload)
    ? payload
    : Array.isArray(root.sessions)
      ? root.sessions
      : Array.isArray(root.items)
        ? root.items
        : [];
  return list.map(normalizeSession);
}

/**
 * Reads the live counters of one protocol.
 *
 * The server sends `ProtocolMetricsDto`: the name of the protocol and its
 * counters in the field `stats` (`StreamStatsDto`). The flat form is
 * accepted as well, so a server that reports the counters next to the name
 * is read the same way.
 *
 * @param {unknown} raw
 * @param {string} id
 * @returns {{id: string, videoFrames: number, bytesReceived: number,
 *   bytesSent: number, packetsLost: number, framesDropped: number,
 *   audioPackets: number, dropRatio: number}}
 */
function readCounters(raw, id) {
  const entry = raw && typeof raw === "object" ? raw : {};
  const source = entry.stats && typeof entry.stats === "object" ? entry.stats : entry;
  return {
    id,
    videoFrames: pickNumber(source, ["video_frames", "videoFrames", "frames"]) ?? 0,
    bytesReceived: pickNumber(source, ["bytes_received", "bytesReceived", "bytes"]) ?? 0,
    bytesSent: pickNumber(source, ["bytes_sent", "bytesSent"]) ?? 0,
    packetsLost: pickNumber(source, ["packets_lost", "packetsLost", "lost"]) ?? 0,
    framesDropped:
      pickNumber(source, ["video_frames_dropped", "framesDropped", "dropped"]) ?? 0,
    audioPackets: pickNumber(source, ["audio_packets", "audioPackets"]) ?? 0,
    dropRatio: pickNumber(source, ["drop_ratio", "dropRatio"]) ?? 0,
  };
}

/**
 * Normalises one sample of a time series.
 *
 * WHY: the server of today reports the cumulative counters, not a series —
 * the panel derives the rate from the counters itself (see rates.js). The
 * reader exists for a server that delivers a series of its own.
 *
 * @param {unknown} raw
 * @returns {{t: number, fps: number, bytesPerSecond: number, protocol: string}}
 */
export function normalizeSample(raw) {
  const source = raw && typeof raw === "object" ? raw : {};
  const timeRaw = pick(source, ["t", "time", "timestamp", "ts"]);
  let t = Date.now();
  if (typeof timeRaw === "number" && Number.isFinite(timeRaw)) {
    t = timeRaw > 1e12 ? timeRaw : timeRaw * 1000;
  } else if (typeof timeRaw === "string") {
    const parsed = Date.parse(timeRaw);
    if (!Number.isNaN(parsed)) t = parsed;
  }
  return {
    t,
    fps: pickNumber(source, ["fps", "frame_rate", "frameRate", "frames_per_second"]) ?? 0,
    bytesPerSecond:
      pickNumber(source, ["bytes_per_second", "bytesPerSecond", "throughput", "bitrate"]) ?? 0,
    protocol: (pickString(source, ["protocol", "kind"]) || "").toLowerCase(),
  };
}

/**
 * Normalises the answer of `GET /api/metrics`.
 *
 * The server sends `MetricsResponse`: a list of `ProtocolMetricsDto`.
 *
 * @param {unknown} payload
 * @returns {{protocols: Record<string, object>, series: object[]}}
 */
export function normalizeMetrics(payload) {
  const root = payload && typeof payload === "object" ? payload : {};
  const body = root.metrics && typeof root.metrics === "object" ? root.metrics : root;
  const protocols = {};
  const rawProtocols = body.protocols ?? body.receivers;
  if (Array.isArray(rawProtocols)) {
    for (const entry of rawProtocols) {
      const id = (pickString(entry, ["protocol", "kind", "id"]) || "").toLowerCase();
      if (id) protocols[id] = readCounters(entry, id);
    }
  } else if (rawProtocols && typeof rawProtocols === "object") {
    for (const [key, entry] of Object.entries(rawProtocols)) {
      protocols[key.toLowerCase()] = readCounters(entry, key.toLowerCase());
    }
  }
  const rawSeries = body.series ?? body.samples ?? body.history ?? body.points ?? [];
  const series = Array.isArray(rawSeries) ? rawSeries : [];
  return { protocols, series: series.map(normalizeSample) };
}
