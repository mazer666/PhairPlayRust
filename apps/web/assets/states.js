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
 * plus the `disabled` case that the `enabled` flag produces. The wording
 * comes from the language file, so the panel speaks the language of the
 * user.
 */

import { t } from "./i18n.js";

/**
 * The tone of the badge for every state of the core.
 *
 * WHY the tone is a table and not a part of the language: the tone is a
 * design decision — a colour and a shape — and the wording is a text. Only
 * the wording is translated.
 *
 * @type {Record<string, string>}
 */
const STATE_TONE = {
  idle: "disabled",
  starting: "transition",
  advertising: "advertising",
  streaming: "streaming",
  stopping: "transition",
  stopped: "disabled",
  failed: "error",
  disabled: "disabled",
  unknown: "disabled",
};

/**
 * The badge of every state, with the wording of the current language.
 *
 * @type {Record<string, {tone: string, label: string, reason: string}>}
 */
export const STATE_VIEW = {};

/**
 * Fills the vocabulary with the wording of the language that is spoken.
 */
function fillVocabulary() {
  for (const [state, tone] of Object.entries(STATE_TONE)) {
    STATE_VIEW[state] = {
      tone,
      label: t(`state.${state}.label`),
      reason: t(`state.${state}.reason`),
    };
  }
}

fillVocabulary();

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
