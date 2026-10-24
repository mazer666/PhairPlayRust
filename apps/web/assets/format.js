/**
 * Number and time formatting for the dashboard.
 *
 * WHAT: the formatters that turn raw counter values into the short strings
 * a human reads at a glance.
 * WHY: a receiver counter reaches hundreds of millions within minutes, and
 * "1048576" is not a thing a person can judge. Every number on the panel
 * goes through one of these helpers, so the units stay consistent.
 * HOW: `Intl` only — the platform ships it, nothing is downloaded.
 */

/** @type {Intl.NumberFormat} */
const plain = new Intl.NumberFormat("en-US");

/**
 * Formats a counter with thousands separators.
 *
 * @param {number|null|undefined} value
 * @returns {string} the formatted number, or an em dash when unknown
 */
export function formatCount(value) {
  if (typeof value !== "number" || !Number.isFinite(value)) return "—";
  return plain.format(Math.round(value));
}

/**
 * Formats a byte count with decimal units.
 *
 * WHY decimal: the panel reports network throughput, and network tooling
 * counts in kilobytes and megabytes per second, not in kibibytes.
 *
 * @param {number|null|undefined} value
 * @returns {string}
 */
export function formatBytes(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) return "—";
  if (value < 1000) return `${Math.round(value)} B`;
  const units = ["kB", "MB", "GB", "TB", "PB"];
  let scaled = value / 1000;
  let unit = 0;
  while (scaled >= 1000 && unit < units.length - 1) {
    scaled /= 1000;
    unit += 1;
  }
  const digits = scaled < 10 ? 2 : scaled < 100 ? 1 : 0;
  return `${scaled.toFixed(digits)} ${units[unit]}`;
}

/**
 * Formats a throughput value in bytes per second.
 *
 * @param {number|null|undefined} value
 * @returns {string}
 */
export function formatRate(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) return "—";
  return `${formatBytes(value)}/s`;
}

/**
 * Formats a frame rate.
 *
 * @param {number|null|undefined} value
 * @returns {string}
 */
export function formatFps(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) return "—";
  return `${value.toFixed(value < 10 ? 2 : 1)} fps`;
}

/**
 * Formats a duration in seconds as a compact human string.
 *
 * @param {number|null|undefined} seconds
 * @returns {string}
 */
export function formatDuration(seconds) {
  if (typeof seconds !== "number" || !Number.isFinite(seconds) || seconds < 0) return "—";
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  if (hours > 0) return `${hours}h ${String(minutes).padStart(2, "0")}m`;
  if (minutes > 0) return `${minutes}m ${String(rest).padStart(2, "0")}s`;
  return `${rest}s`;
}

/**
 * Formats a wall-clock timestamp as a local time of day.
 *
 * @param {number|null|undefined} milliseconds
 * @returns {string}
 */
export function formatClock(milliseconds) {
  if (typeof milliseconds !== "number" || !Number.isFinite(milliseconds)) return "—";
  const date = new Date(milliseconds);
  if (Number.isNaN(date.getTime())) return "—";
  return date.toLocaleTimeString("en-GB", { hour12: false });
}

/**
 * Returns the number of seconds since a timestamp, or null.
 *
 * @param {number|null|undefined} milliseconds
 * @returns {number|null}
 */
export function secondsSince(milliseconds) {
  if (typeof milliseconds !== "number" || !Number.isFinite(milliseconds)) return null;
  return Math.max(0, (Date.now() - milliseconds) / 1000);
}
