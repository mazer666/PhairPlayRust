/**
 * The four receiver cards of the dashboard.
 *
 * WHAT: builds the cards, paints their state and switches a receiver on and
 * off.
 * WHY: the card is the unit of the panel and, on a television, the card is
 * also the control: the user points at a card with the arrow keys and
 * presses OK. A small switch in the corner of the card would be a target
 * of a few pixels that nobody hits with a remote control. The switch stays
 * in the card as the visible state, but the card itself is the thing that
 * is focused and that acts.
 * HOW: the cards are cloned from the template of the document, filled from
 * the normalised status of the API and wired to a [`FocusGroup`] that owns
 * the tab order and the arrow keys. The action calls the real endpoint of
 * the receiver — there is no local fiction about the state of a protocol.
 *
 * EXAMPLE:
 *
 * ```js
 * const cards = new ProtocolCards({
 *   grid: document.getElementById("protocol-grid"),
 *   template: document.getElementById("protocol-card-template"),
 *   api,
 *   onError: (message) => showBanner(message),
 * });
 * cards.build();
 * ```
 */

import { PROTOCOLS } from "./api.js";
import { FocusGroup } from "./focus-group.js";
import { formatBytes, formatCount } from "./format.js";
import { applyTranslations, t } from "./i18n.js";
import { resolveState } from "./states.js";

export class ProtocolCards {
  /**
   * @param {{grid: HTMLElement, template: HTMLElement, api: object,
   *   onError: (message: string) => void, onEscape?: () => void}} options
   */
  constructor(options) {
    this.grid = options.grid;
    this.template = options.template;
    this.api = options.api;
    this.onError = options.onError ?? (() => {});
    /** @type {Map<string, HTMLElement>} */
    this.cards = new Map();
    /** @type {Map<string, object>} */
    this.protocols = new Map();
    /** @type {Record<string, object>} */
    this.counters = {};
    /** @type {Set<string>} */
    this.pending = new Set();
    /**
     * The counter of the local changes.
     *
     * WHY it exists: the panel reads the state of the receiver every two
     * seconds. A reading that was requested before the user pressed OK can
     * arrive after it and would switch the card back — the user would see
     * the receiver as it was before the key press. Every local change
     * carries a number, and a reading that is older than the last change
     * does not overwrite it.
     */
    this.revision = 0;
    /** @type {Map<string, number>} */
    this.changedRevision = new Map();
    /** @type {Map<string, object>} */
    this.localState = new Map();
    this.group = new FocusGroup(this.grid, {
      onActivate: (item) => this.toggle(item.dataset.protocol),
      onEscape: options.onEscape ?? (() => {}),
    });
  }

  /**
   * Reads a named field of a card.
   *
   * @param {Element} root
   * @param {string} name
   * @returns {Element|null}
   */
  static field(root, name) {
    return root.querySelector(`[data-field="${name}"]`);
  }

  /**
   * Sets the text of a named field of a card.
   *
   * @param {Element} root
   * @param {string} name
   * @param {string} value
   */
  static setField(root, name, value) {
    const element = ProtocolCards.field(root, name);
    if (element) element.textContent = value;
  }

  /**
   * Builds the four cards and hands them to the focus group.
   */
  build() {
    const fragment = document.createDocumentFragment();
    for (const protocol of PROTOCOLS) {
      const card = this.template.content.firstElementChild.cloneNode(true);
      card.dataset.protocol = protocol.id;
      card.id = `card-${protocol.id}`;
      // WHY the translation of the clone: the template lives outside the
      // document, so the fill of the document does not reach it.
      applyTranslations(card);
      const name = ProtocolCards.field(card, "name");
      name.id = `card-name-${protocol.id}`;
      ProtocolCards.setField(card, "name", protocol.name);
      card.querySelector("use").setAttribute("href", protocol.glyph);
      const reason = ProtocolCards.field(card, "reason");
      reason.id = `reason-${protocol.id}`;
      const action = ProtocolCards.field(card, "action");
      action.id = `action-${protocol.id}`;
      ProtocolCards.setField(card, "action", t("card.action", { name: protocol.name }));
      // WHY the name and the reason are named: the card is a button, and a
      // button without a name is an anonymous target for a screen reader.
      // The name is the protocol, the state is in `aria-pressed`, and the
      // description says what the card does.
      card.setAttribute("aria-labelledby", name.id);
      card.setAttribute("aria-describedby", `${reason.id} ${action.id}`);
      card.addEventListener("click", () => this.toggle(protocol.id));
      this.cards.set(protocol.id, card);
      fragment.append(card);
    }
    this.grid.append(fragment);
    this.group.setItems([...this.cards.values()]);
  }

  /**
   * Writes the readings of the API into the cards.
   *
   * @param {Map<string, object>} protocols the status of every receiver
   * @param {Record<string, object>} counters the live counters of every receiver
   * @param {number} [revision] the number of the last local change at the
   *   moment the reading was requested
   */
  update(protocols, counters, revision = -1) {
    this.protocols = protocols;
    this.counters = counters ?? {};
    for (const protocol of PROTOCOLS) {
      const data = protocols.get(protocol.id);
      if (!data) continue;
      // WHY the local state: a reading that is older than the last change
      // of the user is not allowed to switch the card back.
      const local = this.localState.get(protocol.id);
      const changed = this.changedRevision.get(protocol.id) ?? -1;
      if (local && changed > revision) Object.assign(data, local);
      this.render(protocol.id, data, this.counters[protocol.id]);
    }
  }

  /**
   * Paints one receiver card.
   *
   * @param {string} id
   * @param {object} data the normalised status of the receiver
   * @param {object|null} counter the live counters of the receiver
   */
  render(id, data, counter) {
    const card = this.cards.get(id);
    if (!card || !data) return;
    const view = resolveState(data);

    const badge = ProtocolCards.field(card, "state");
    badge.dataset.state = view.tone;
    ProtocolCards.setField(card, "state-label", view.label);

    const reason = ProtocolCards.field(card, "reason");
    // WHY the class: the server reports a protocol that is enabled but has
    // no engine as the state `idle` with the reason in `last_error`. The
    // reason is the failure — it is shown in the colour of a failure.
    reason.className = data.lastError ? "state__reason state__reason--error" : "state__reason";
    reason.textContent = data.lastError
      ? t("state.lastError", { reason: view.reason, error: data.lastError })
      : view.reason;

    const port = ProtocolCards.field(card, "port");
    port.textContent = data.tcpPort ? String(data.tcpPort) : t("value.unknown");
    if (data.udpPort) {
      const suffix = document.createElement("span");
      suffix.className = "subtle";
      suffix.textContent = t("card.udpPort", { port: data.udpPort });
      port.append(suffix);
    }

    ProtocolCards.setField(card, "peers", formatCount(data.peers));

    // The switch is the state of the card, not a second control: it is
    // hidden from the screen reader, because the card itself carries the
    // state in `aria-pressed`.
    const toggle = ProtocolCards.field(card, "switch");
    toggle.dataset.on = String(data.enabled);
    ProtocolCards.setField(card, "switch-label", t(data.enabled ? "card.switch.on" : "card.switch.off"));
    card.setAttribute("aria-pressed", String(data.enabled));

    if (counter) {
      ProtocolCards.setField(card, "frames", formatCount(counter.videoFrames));
      ProtocolCards.setField(card, "bytes", formatBytes(counter.bytesReceived));
      const loss = ProtocolCards.field(card, "loss");
      loss.textContent = formatCount(counter.packetsLost);
      loss.dataset.tone = counter.packetsLost > 0 ? "danger" : "";
    }
  }

  /**
   * Switches a receiver on or off and reports the result.
   *
   * WHY the busy flag: the key of a remote control repeats, and two
   * requests would fight over the same configuration. The card shows that
   * it is working instead of freezing.
   *
   * @param {string} id
   */
  async toggle(id) {
    const data = this.protocols.get(id);
    const card = this.cards.get(id);
    if (!data || !card || this.pending.has(id)) return;
    const wanted = !data.enabled;
    this.pending.add(id);
    card.dataset.busy = "true";
    card.setAttribute("aria-busy", "true");
    try {
      const status = await this.api.setProtocolEnabled(id, wanted);
      // WHY optimistic: the server answers with the status after the action.
      // It is used right away; the next poll confirms it.
      data.enabled = wanted;
      if (status) Object.assign(data, status);
      // The answer of the server is the truth from now on. It is kept
      // aside, so that a reading that was already on its way cannot
      // switch the card back.
      this.revision += 1;
      this.changedRevision.set(id, this.revision);
      this.localState.set(id, {
        enabled: status ? status.enabled : wanted,
        state: status ? status.state : data.state,
      });
      this.render(id, data, this.counters[id]);
    } catch (error) {
      this.onError(
        t("error.toggleFailed", {
          state: t(wanted ? "error.switchedOn" : "error.switchedOff"),
          reason: error.message,
        }),
      );
    } finally {
      this.pending.delete(id);
      card.dataset.busy = "false";
      card.removeAttribute("aria-busy");
    }
  }
}
