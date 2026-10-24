/**
 * A mock of the local receiver, for the end-to-end check of the dashboard.
 *
 * WHAT: a small HTTP server that serves the dashboard out of `apps/web`
 * and answers the API endpoints of the receiver in the shape of
 * `apps/web/src/dto.rs`.
 * WHY: the dashboard has to be looked at and operated in a real browser.
 * A mock that invents its own JSON would only prove that the mock and the
 * panel agree with each other — the shape below is the shape of the server,
 * field for field, so that a check against the mock is a check against the
 * contract.
 * HOW: `node mock-receiver.mjs [port]`. The port defaults to 8099. The
 * counters of the running stream grow with the time, so that the graph has
 * a curve.
 *
 * EXAMPLE:
 *
 * ```sh
 * node apps/web/tools/mock-receiver.mjs 8099
 * ```
 */

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const STARTED = Date.now();

/** The content types of the files of the dashboard. */
const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".svg": "image/svg+xml",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
};

/** The status of the four receivers, in the shape of `ProtocolStatusDto`. */
const protocols = [
  {
    protocol: "airplay",
    display_name: "AirPlay 2",
    state: "streaming",
    enabled: true,
    tcp_port: 7000,
    udp_port: 6000,
    peer_count: 1,
    last_error: null,
  },
  {
    protocol: "miracast",
    display_name: "Miracast",
    state: "advertising",
    enabled: true,
    tcp_port: 7236,
    udp_port: null,
    peer_count: 0,
    last_error: null,
  },
  {
    protocol: "cast",
    display_name: "Google Cast",
    state: "stopped",
    enabled: false,
    tcp_port: null,
    udp_port: null,
    peer_count: 0,
    last_error: null,
  },
  {
    protocol: "dlna",
    display_name: "DLNA / UPnP",
    state: "failed",
    enabled: true,
    tcp_port: null,
    udp_port: null,
    peer_count: 0,
    last_error: "could not bind the SOAP listener: address already in use",
  },
  {
    protocol: "smartview",
    display_name: "Samsung Smart View",
    state: "advertising",
    enabled: true,
    tcp_port: 8001,
    udp_port: null,
    peer_count: 0,
    last_error: null,
  },
];

/** The live session, in the shape of `SessionDto`. */
const sessions = [
  {
    id: 1,
    protocol: "airplay",
    sender_name: "Studio iPhone",
    sender_address: "192.168.1.24",
    video: "video/avc 1920x1080",
    audio: "audio/aac 44100 Hz 2 ch",
    uptime_ms: 41230,
  },
];

/** The counters, in the shape of `StreamStatsDto`. */
function stats(frames, bytes) {
  return {
    video_frames: frames,
    video_frames_dropped: 0,
    audio_packets: 900,
    audio_packets_dropped: 0,
    bytes_received: bytes,
    bytes_sent: 0,
    packets_lost: 0,
    frames_skipped: 0,
    drop_ratio: 0,
  };
}

/**
 * Answers the API of the receiver.
 *
 * @param {string} path the path of the request
 * @param {string} method the method of the request
 * @returns {unknown|null} the payload, or null when the path is a file
 */
export function answer(path, method) {
  if (method === "POST" && path.startsWith("/api/protocols/")) {
    // "/api/protocols/cast/enable" → ["", "api", "protocols", "cast", "enable"]
    const [, , , name, action] = path.split("/");
    const entry = protocols.find((item) => item.protocol === name);
    if (!entry) return { error: { code: 1, message: "unknown protocol" } };
    const ports = { airplay: 7000, miracast: 7236, cast: 8009, dlna: 8200, smartview: 8001 };
    if (action === "enable") {
      entry.enabled = true;
      entry.state = "advertising";
      entry.tcp_port = ports[name];
    } else if (action === "disable") {
      entry.enabled = false;
      entry.state = "stopped";
      entry.tcp_port = null;
    }
    // `ProtocolActionResponse`: the status after the action.
    return { status: entry };
  }
  const elapsed = (Date.now() - STARTED) / 1000;
  if (path === "/api/status") {
    return { protocols, active_sessions: sessions.length, uptime_ms: 812345 };
  }
  if (path === "/api/sessions") return { sessions };
  if (path === "/api/metrics") {
    return {
      protocols: [
        { protocol: "airplay", stats: stats(Math.floor(1200 + elapsed * 60), Math.floor(9_800_000 + elapsed * 5_240_000)) },
        { protocol: "miracast", stats: stats(0, 0) },
        { protocol: "cast", stats: stats(0, 0) },
        { protocol: "dlna", stats: stats(0, 0) },
        { protocol: "smartview", stats: stats(0, 0) },
      ],
    };
  }
  if (path === "/api/config") {
    return { device: { name: "Wohnzimmer" }, protocols: { airplay: { enabled: true, port: 7000 } } };
  }
  if (path === "/api/build-info") {
    return {
      version: "0.1.0",
      name: "phairplayrust",
      license: "AGPL-3.0-or-later",
      repository: "https://github.com/mazer666/PhairPlayRust",
      rustc_version: "rustc 1.99.1",
      target: "x86_64-unknown-linux-gnu",
    };
  }
  return null;
}

/**
 * Serves the dashboard and the API.
 *
 * @param {number} port the port to listen on
 * @returns {Promise<import("node:http").Server>}
 */
export async function serve(port) {
  const server = createServer(async (request, response) => {
    const path = new URL(request.url ?? "/", "http://localhost").pathname;
    const payload = answer(path, request.method ?? "GET");
    if (payload !== null) {
      const body = JSON.stringify(payload);
      response.writeHead(200, {
        "content-type": "application/json; charset=utf-8",
        "cache-control": "no-store",
      });
      response.end(body);
      return;
    }
    const relative = path === "/" ? "index.html" : path.replace(/^\/+/, "");
    const file = join(ROOT, normalize(relative));
    if (!file.startsWith(ROOT)) {
      response.writeHead(403).end("forbidden");
      return;
    }
    try {
      const body = await readFile(file);
      response.writeHead(200, {
        "content-type": TYPES[extname(file)] ?? "application/octet-stream",
        "cache-control": "no-store",
      });
      response.end(body);
    } catch (error) {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end("not found");
    }
  });
  await new Promise((resolve) => server.listen(port, "127.0.0.1", resolve));
  return server;
}

if (process.argv[1] && import.meta.url.endsWith(process.argv[1].split("/").pop() ?? "")) {
  const port = Number(process.argv[2] ?? 8099);
  await serve(port);
  console.log(`mock receiver on http://127.0.0.1:${port}`);
}
