/**
 * Client for the local receiver API.
 *
 * WHAT: the single place that talks to the Rust server.
 * WHY: the panel must work against the documented contract without knowing
 * how the server is built, and every request must fail soft — a receiver
 * that is not running has to say so, not break the page.
 * HOW: plain `fetch` and the native `WebSocket`. No dependency, no polyfill,
 * no build step. The answers are normalised in normalize.js.
 */

import {
  normalizeMetrics,
  normalizeProtocol,
  normalizeSessions,
  normalizeStatus,
} from "./normalize.js";

/** The four receivers, in the order the panel shows them. */
export const PROTOCOLS = [
  { id: "airplay", name: "AirPlay 2", glyph: "#glyph-airplay" },
  { id: "miracast", name: "Miracast", glyph: "#glyph-miracast" },
  { id: "cast", name: "Google Cast", glyph: "#glyph-cast" },
  { id: "dlna", name: "DLNA / UPnP", glyph: "#glyph-dlna" },
];

/**
 * The HTTP and WebSocket client of the local receiver.
 */
export class ReceiverApi {
  /**
   * @param {string} [base] the base path of the API, "" for the same origin
   */
  constructor(base = "") {
    this.base = base.replace(/\/$/, "");
  }

  /**
   * Performs a GET request and parses the answer as JSON.
   *
   * @param {string} path
   * @returns {Promise<unknown>}
   */
  async get(path) {
    const response = await fetch(`${this.base}${path}`, {
      headers: { accept: "application/json" },
      cache: "no-store",
    });
    if (!response.ok) {
      throw new Error(`${path} responded with ${response.status}`);
    }
    return response.json();
  }

  /**
   * Reads the state of all receivers.
   *
   * @returns {Promise<object>}
   */
  async status() {
    return normalizeStatus(await this.get("/api/status"));
  }

  /**
   * Reads the live sessions.
   *
   * @returns {Promise<object[]>}
   */
  async sessions() {
    return normalizeSessions(await this.get("/api/sessions"));
  }

  /**
   * Reads the live counters and the time series.
   *
   * @returns {Promise<{protocols: Record<string, object>, series: object[]}>}
   */
  async metrics() {
    return normalizeMetrics(await this.get("/api/metrics"));
  }

  /**
   * Reads the configuration of the receiver.
   *
   * @returns {Promise<unknown>}
   */
  async config() {
    return this.get("/api/config");
  }

  /**
   * Reads the build metadata.
   *
   * @returns {Promise<Record<string, unknown>>}
   */
  async buildInfo() {
    const payload = await this.get("/api/build-info");
    return payload && typeof payload === "object" ? payload : {};
  }

  /**
   * Switches a receiver on or off.
   *
   * The server answers with `ProtocolActionResponse` — the status of the
   * protocol *after* the action — so the panel can paint it without
   * waiting for the next poll.
   *
   * @param {string} name the protocol id, e.g. "airplay"
   * @param {boolean} enabled
   * @returns {Promise<object|null>} the new status, or null without an answer
   */
  async setProtocolEnabled(name, enabled) {
    const action = enabled ? "enable" : "disable";
    const response = await fetch(`${this.base}/api/protocols/${name}/${action}`, {
      method: "POST",
      headers: { accept: "application/json" },
    });
    if (!response.ok) {
      throw new Error(`${action} ${name} responded with ${response.status}`);
    }
    if (response.status === 204) return null;
    const payload = await response.json();
    const status = payload && typeof payload === "object" ? payload.status : null;
    return status ? normalizeProtocol(status, name) : null;
  }

  /**
   * Opens the event channel.
   *
   * @param {(message: unknown) => void} onMessage
   * @param {(state: string) => void} onState
   * @returns {WebSocket}
   */
  events(onMessage, onState) {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const socket = new WebSocket(`${protocol}//${window.location.host}${this.base}/api/events`);
    socket.addEventListener("message", (event) => {
      try {
        onMessage(JSON.parse(event.data));
      } catch (error) {
        onMessage(null);
      }
    });
    socket.addEventListener("open", () => onState("open"));
    socket.addEventListener("close", () => onState("closed"));
    socket.addEventListener("error", () => onState("error"));
    return socket;
  }
}
