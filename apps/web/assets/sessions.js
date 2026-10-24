/**
 * The live session list of the dashboard.
 *
 * WHAT: the list of the senders that stream to the receiver right now.
 * WHY: "who is casting to me, with which codec, since when" is the question
 * a user asks when something unexpected appears on the screen. The list
 * answers it with the address, the negotiated media and the running time.
 * HOW: plain DOM nodes built from the normalised session model. The sender
 * name is never trusted as markup — it is written as text.
 *
 * WHY the counters carry the protocol name: the server reports the counters
 * per protocol, not per session (`SessionDto` has no counters of its own).
 * The row therefore names the protocol the numbers belong to.
 */

import { PROTOCOLS } from "./api.js";
import { formatBytes, formatCount, formatDuration } from "./format.js";

/**
 * Describes the media of one session.
 *
 * @param {object} session
 * @returns {string[]}
 */
export function sessionDetails(session) {
  const details = [];
  if (session.senderAddress) details.push(session.senderAddress);
  if (session.video) details.push(session.video);
  if (session.audio) details.push(session.audio);
  const running = sessionRunningTime(session);
  if (running !== null) details.push(`running ${formatDuration(running)}`);
  return details;
}

/**
 * The running time of a session, in seconds.
 *
 * WHY two sources: the server reports the uptime of the session in
 * milliseconds; a session that was started somewhere else carries a
 * timestamp instead.
 *
 * @param {object} session
 * @returns {number|null}
 */
export function sessionRunningTime(session) {
  if (typeof session.uptimeMs === "number" && Number.isFinite(session.uptimeMs)) {
    return session.uptimeMs / 1000;
  }
  if (typeof session.startedAt === "number" && Number.isFinite(session.startedAt)) {
    return Math.max(0, (Date.now() - session.startedAt) / 1000);
  }
  return null;
}

/**
 * Builds one row of the session list.
 *
 * @param {object} session
 * @param {Record<string, object>} counters the counters of every protocol
 * @returns {HTMLElement}
 */
function buildSession(session, counters) {
  const protocol = PROTOCOLS.find((entry) => entry.id === session.protocol);
  const item = document.createElement("div");
  item.className = "session";

  const name = document.createElement("div");
  name.className = "session__name";
  name.textContent = session.senderName || "Unknown sender";
  item.append(name);

  const tag = document.createElement("span");
  tag.className = "tag";
  tag.textContent = protocol ? protocol.name : session.protocol || "unknown";
  item.append(tag);

  const meta = document.createElement("div");
  meta.className = "session__meta";
  for (const text of sessionDetails(session)) {
    const span = document.createElement("span");
    span.textContent = text;
    meta.append(span);
  }
  item.append(meta);

  const stats = document.createElement("div");
  stats.className = "session__stats";
  const counter = counters?.[session.protocol];
  if (counter) {
    const scope = document.createElement("span");
    scope.textContent = `${session.protocol} ·`;
    stats.append(scope);
  }
  const lines = counter
    ? [
        `${formatCount(counter.videoFrames)} frames`,
        `${formatBytes(counter.bytesReceived)} received`,
        `${formatCount(counter.packetsLost)} packets lost`,
      ]
    : [];
  for (const text of lines) {
    const span = document.createElement("span");
    span.textContent = text;
    stats.append(span);
  }
  item.append(stats);
  return item;
}

/**
 * Paints the list of live sessions.
 *
 * @param {object[]} sessions
 * @param {Record<string, object>} [counters] the counters of every protocol
 */
export function renderSessions(sessions, counters = {}) {
  const list = document.getElementById("sessions-list");
  const count = document.getElementById("sessions-count");
  list.textContent = "";
  if (!sessions || sessions.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = "Nothing is streaming at the moment.";
    list.append(empty);
    count.textContent = "No sender is connected.";
    return;
  }
  count.textContent = `${sessions.length} session${sessions.length === 1 ? "" : "s"} streaming.`;
  for (const session of sessions) {
    list.append(buildSession(session, counters));
  }
}
