/**
 * The roving tabindex of the dashboard.
 *
 * WHAT: a group of elements that behave like one widget for the keyboard —
 * exactly one of them is in the tab order, the arrow keys move the focus
 * inside the group, and the Tab key leaves it.
 * WHY: the dashboard is watched on a television, where the only pointer is
 * the remote control. Four cards that are four tab stops would need four
 * presses of Tab before the user reaches the second card; a group that
 * behaves like one widget needs one press and then the arrow keys. This is
 * the standard pattern for composite widgets (the roving tabindex of the
 * WAI-ARIA authoring practices).
 * HOW: the group keeps the index of the card that is in the tab order, sets
 * `tabindex="0"` on it and `tabindex="-1"` on the others, and moves the
 * focus with the arrow keys. The columns of the grid are measured from the
 * rendered layout, so the navigation follows the grid the user sees and not
 * a constant that the stylesheet may contradict.
 *
 * EXAMPLE:
 *
 * ```js
 * const group = new FocusGroup(document.getElementById("protocol-grid"), {
 *   onActivate: (card) => toggle(card.dataset.protocol),
 * });
 * group.setItems([...document.querySelectorAll(".card")]);
 * ```
 */

/** The elements that can take the focus, in the order of the tab key. */
const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export class FocusGroup {
  /**
   * @param {HTMLElement} container the element that holds the items
   * @param {{onActivate?: (item: HTMLElement, index: number) => void,
   *   onEscape?: () => boolean|void, activate?: boolean}} [options]
   */
  constructor(container, options = {}) {
    /** @type {HTMLElement} */
    this.container = container;
    /** @type {HTMLElement[]} */
    this.items = [];
    /** @type {number} */
    this.activeIndex = 0;
    /** @type {(item: HTMLElement, index: number) => void} */
    this.onActivate = options.onActivate ?? (() => {});
    /** @type {() => boolean|void} */
    this.onEscape = options.onEscape ?? (() => {});
    /**
     * WHY the switch: a group of real buttons keeps the activation of the
     * browser — the space key presses a button on its own. A group of
     * elements that only look like buttons has to do it itself.
     */
    this.activate = options.activate !== false;
    this.container.addEventListener("keydown", (event) => this.handleKeydown(event));
    this.container.addEventListener("focusin", (event) => this.trackFocus(event));
  }

  /**
   * Declares the items of the group.
   *
   * @param {HTMLElement[]} items
   */
  setItems(items) {
    this.items = items.filter((item) => item instanceof HTMLElement);
    this.activeIndex = Math.min(this.activeIndex, Math.max(0, this.items.length - 1));
    this.applyTabIndex();
  }

  /**
   * Writes the tab order of the group: one item is in it, the others are not.
   */
  applyTabIndex() {
    this.items.forEach((item, index) => {
      item.tabIndex = index === this.activeIndex ? 0 : -1;
      item.dataset.rovingActive = index === this.activeIndex ? "true" : "false";
    });
  }

  /**
   * The index of an item, or -1 when it does not belong to the group.
   *
   * @param {EventTarget|null} target
   * @returns {number}
   */
  indexOf(target) {
    if (!(target instanceof HTMLElement)) return -1;
    const item = target.closest("[data-focus-item]");
    if (!(item instanceof HTMLElement)) return -1;
    return this.items.indexOf(item);
  }

  /**
   * Remembers the item that received the focus.
   *
   * WHY: the mouse is allowed to move the focus as well. A click on the
   * third card has to leave the third card in the tab order, otherwise the
   * next Tab would jump back to the first one.
   *
   * @param {Event} event
   */
  trackFocus(event) {
    const index = this.indexOf(event.target);
    if (index < 0 || index === this.activeIndex) return;
    this.activeIndex = index;
    this.applyTabIndex();
  }

  /**
   * The number of columns the grid shows right now.
   *
   * WHY measured and not configured: the grid has one column on a phone and
   * two or four on a television. The navigation has to follow the layout
   * that the user sees.
   *
   * WHY the offset and not the rectangle: the focused card is scaled up by
   * two percent, and a scaled rectangle sits a few pixels higher than the
   * others. Measuring the rectangle would then report one column for a
   * grid of four, and the arrow keys would walk down the column instead of
   * across the row. The offset is the place of the card in the layout and
   * does not move with a transform.
   *
   * @returns {number}
   */
  columnCount() {
    if (this.items.length === 0) return 1;
    const first = this.rowKey(this.items[0]);
    let columns = 0;
    for (const item of this.items) {
      if (this.rowKey(item) !== first) break;
      columns += 1;
    }
    return Math.max(1, columns);
  }

  /**
   * The row a card stands in.
   *
   * @param {HTMLElement} item
   * @returns {number}
   */
  rowKey(item) {
    return item.offsetTop;
  }

  /**
   * The card the arrow keys move the focus to.
   *
   * The rule, written down so that it can be tested:
   *   Right  the next card of the same row
   *   Left   the previous card of the same row
   *   Down   the card directly below
   *   Up     the card directly above
   *   Home   the first card of the group
   *   End    the last card of the group
   * In a grid with a single column Right and Down lead to the next card,
   * Left and Up to the previous one. A move that has no target does not
   * move the focus inside the group — it leaves the group, see
   * {@link FocusGroup.focusOutside}.
   *
   * @param {number} index the index of the card that has the focus
   * @param {string} key the key of the keyboard
   * @returns {number} the index of the target, or -1 when there is none
   */
  targetIndex(index, key) {
    const last = this.items.length - 1;
    const columns = this.columnCount();
    if (key === "Home") return 0;
    if (key === "End") return last;
    if (key === "ArrowRight") {
      if (columns === 1) return index < last ? index + 1 : -1;
      const next = index + 1;
      return next <= last && next % columns !== 0 ? next : -1;
    }
    if (key === "ArrowLeft") {
      if (columns === 1) return index > 0 ? index - 1 : -1;
      const previous = index - 1;
      return previous >= 0 && index % columns !== 0 ? previous : -1;
    }
    if (key === "ArrowDown") {
      const below = index + columns;
      return below <= last ? below : -1;
    }
    if (key === "ArrowUp") {
      const above = index - columns;
      return above >= 0 ? above : -1;
    }
    return -1;
  }

  /**
   * Moves the focus to a card of the group.
   *
   * @param {number} index
   * @param {boolean} [focus] whether the card receives the focus
   */
  focusIndex(index, focus = true) {
    if (this.items.length === 0) return;
    const clamped = Math.max(0, Math.min(index, this.items.length - 1));
    this.activeIndex = clamped;
    this.applyTabIndex();
    if (focus) this.items[clamped].focus();
  }

  /**
   * The focusable elements of the document that are not part of the group.
   *
   * WHY the visibility test: an element that is hidden cannot take the
   * focus, and the remote control would move the focus into nothing.
   *
   * @returns {HTMLElement[]}
   */
  outsideItems() {
    const all = Array.from(document.querySelectorAll(FOCUSABLE)).filter(
      (element) =>
        element instanceof HTMLElement &&
        !this.container.contains(element) &&
        element !== this.container &&
        element.getClientRects().length > 0,
    );
    return all;
  }

  /**
   * Moves the focus out of the group, to the element before or after it.
   *
   * WHY: a remote control has no Tab key. Without this, the arrow keys
   * would end at the last card and the rest of the panel would be
   * unreachable with the arrows alone — a dead end for the user.
   *
   * @param {"next"|"previous"} direction
   * @returns {boolean} whether the focus was moved
   */
  focusOutside(direction) {
    const candidates = this.outsideItems();
    if (candidates.length === 0) return false;
    const following = 4; // Node.DOCUMENT_POSITION_FOLLOWING
    const preceding = 2; // Node.DOCUMENT_POSITION_PRECEDING
    const position = (element) => this.container.compareDocumentPosition(element);
    if (direction === "next") {
      const next = candidates.find((element) => position(element) & following);
      if (!next) return false;
      next.focus();
      return true;
    }
    const previous = candidates.filter((element) => position(element) & preceding).pop();
    if (!previous) return false;
    previous.focus();
    return true;
  }

  /**
   * Handles the keys of the group.
   *
   * @param {KeyboardEvent} event
   */
  handleKeydown(event) {
    const index = this.indexOf(event.target);
    if (index < 0) return;
    const key = event.key;

    if (this.activate && (key === "Enter" || key === " " || key === "Spacebar")) {
      // WHY the preventDefault: the space key scrolls the page, and the
      // panel of a television must not move under the finger of the user.
      event.preventDefault();
      this.onActivate(this.items[index], index);
      return;
    }

    if (key === "Escape") {
      // WHY the callback decides: the escape key belongs to the message
      // that is open, not to the card. If there is no message, the key
      // keeps its meaning for the browser.
      if (this.onEscape() !== false) event.preventDefault();
      return;
    }

    const target = this.targetIndex(index, key);
    if (target >= 0) {
      event.preventDefault();
      this.focusIndex(target, true);
      return;
    }

    // The edge of the group: the arrows leave the group instead of doing
    // nothing at all.
    if (key === "ArrowDown" || key === "ArrowRight") {
      if (this.focusOutside("next")) event.preventDefault();
      return;
    }
    if (key === "ArrowUp" || key === "ArrowLeft") {
      if (this.focusOutside("previous")) event.preventDefault();
    }
  }
}
