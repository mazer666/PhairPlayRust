/**
 * The hint for the operation with a remote control.
 *
 * WHAT: a single line that tells the user that the arrow keys move and
 * that OK switches a receiver — shown only when the device has no fine
 * pointer.
 * WHY: a television has no mouse. The panel has to say how it is operated,
 * because the switch in the corner of a card is not a target that a remote
 * control can hit. The hint is a hint and not an overlay: it never covers
 * the content, it never takes the focus and it disappears the moment a
 * mouse is plugged in.
 * HOW: the media queries of the browser tell whether the device has a
 * coarse pointer (a finger, a remote control) and whether it also has a
 * fine one (a mouse). The hint appears when the coarse pointer is there and
 * the fine one is not.
 *
 * EXAMPLE:
 *
 * ```js
 * watchRemoteHint(document.getElementById("remote-hint"));
 * ```
 */

/**
 * Whether the device is operated without a fine pointer.
 *
 * @returns {boolean}
 */
export function isRemoteOnly() {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return false;
  try {
    const coarse = window.matchMedia("(pointer: coarse)").matches;
    const fine = window.matchMedia("(any-pointer: fine)").matches;
    return coarse && !fine;
  } catch (error) {
    return false;
  }
}

/**
 * Shows the hint when the device has no fine pointer, and hides it when a
 * mouse appears.
 *
 * @param {HTMLElement|null} element the element that carries the hint
 * @returns {boolean} whether the hint is shown
 */
export function watchRemoteHint(element) {
  if (!element) return false;
  const update = () => {
    element.hidden = !isRemoteOnly();
  };
  update();
  if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
    for (const query of ["(pointer: coarse)", "(any-pointer: fine)"]) {
      try {
        const list = window.matchMedia(query);
        if (typeof list.addEventListener === "function") {
          list.addEventListener("change", update);
        }
      } catch (error) {
        /* A browser without the query keeps the state of the first reading. */
      }
    }
  }
  return !element.hidden;
}
