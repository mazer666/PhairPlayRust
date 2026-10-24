/**
 * The state vocabulary of the dashboard.
 *
 * WHAT: the mapping from the state of a receiver engine to the wording and
 * the colour of its badge.
 * WHY: "Stopped" and "Disabled" are different facts — one receiver is
 * switched off in the configuration, the other is enabled but not running.
 * A user must never have to guess which one they are looking at, so every
 * state carries a sentence that explains it.
 * HOW: the states are the ones of `phairplay_core::protocol::ProtocolState`
 * plus the `disabled` case that the `enabled` flag produces.
 */

/** @type {Record<string, {tone: string, label: string, reason: string}>} */
export const STATE_VIEW = {
  idle: { tone: "disabled", label: "Idle", reason: "The receiver is loaded but not started." },
  starting: {
    tone: "transition",
    label: "Starting",
    reason: "Binding sockets and registering the network service.",
  },
  advertising: {
    tone: "advertising",
    label: "Advertising",
    reason: "Visible to senders on the local network, waiting for a connection.",
  },
  streaming: {
    tone: "streaming",
    label: "Streaming",
    reason: "A sender is connected and the stream is running.",
  },
  stopping: {
    tone: "transition",
    label: "Stopping",
    reason: "The peer connection is being torn down.",
  },
  stopped: {
    tone: "disabled",
    label: "Stopped",
    reason: "Enabled in the configuration, but the service is not running.",
  },
  failed: { tone: "error", label: "Error", reason: "The receiver could not be started." },
  disabled: {
    tone: "disabled",
    label: "Disabled",
    reason: "Switched off in the configuration — the service is not advertised.",
  },
  unknown: { tone: "disabled", label: "Unknown", reason: "The receiver did not report a state." },
};

/**
 * Resolves the badge of a receiver.
 *
 * @param {{enabled: boolean, state: string}} data
 * @returns {{tone: string, label: string, reason: string}}
 */
export function resolveState(data) {
  if (!data.enabled) return STATE_VIEW.disabled;
  return STATE_VIEW[data.state] || STATE_VIEW.unknown;
}
