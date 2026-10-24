/**
 * The shared tools of the checks of the dashboard.
 *
 * WHAT: the browser, the colour arithmetic and the mock receiver that the
 * checks of `apps/web/tools` share.
 * WHY: the checks must measure the same thing in the same way — a contrast
 * that is computed in one file and estimated in another is not a
 * measurement. Everything that two checks need lives here, once.
 * HOW: the browser is the Chromium of Playwright; the colours follow the
 * formula of WCAG 2.1; the mock receiver is `mock-receiver.mjs`.
 *
 * EXAMPLE:
 *
 * ```js
 * import { contrast, launch, luminance } from "./lib.mjs";
 * ```
 */

export { serve } from "./mock-receiver.mjs";

/**
 * Loads Playwright from the place where it is installed.
 *
 * WHY the fallback: the repository has no package manager and no
 * dependency of its own — the dashboard ships without a build step. The
 * browser driver is a tool of the check, not a dependency of the product.
 *
 * @returns {Promise<import("playwright").Browser>}
 */
export async function launch() {
  const candidates = [
    process.env.PLAYWRIGHT_MODULE,
    "playwright",
    "/usr/local/lib/node_modules/playwright/index.mjs",
  ].filter(Boolean);
  for (const candidate of candidates) {
    try {
      const module = await import(candidate);
      return await module.chromium.launch({ headless: true });
    } catch (error) {
      /* try the next one */
    }
  }
  throw new Error("no Chromium found — install Playwright or set PLAYWRIGHT_MODULE");
}

/**
 * The relative luminance of a colour, as WCAG 2.1 defines it.
 *
 * @param {[number, number, number]} rgb the red, green and blue of the colour
 * @returns {number}
 */
export function luminance([red, green, blue]) {
  const channel = (value) => {
    const part = value / 255;
    return part <= 0.04045 ? part / 12.92 : ((part + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue);
}

/**
 * The contrast ratio of two colours, as WCAG 2.1 defines it.
 *
 * @param {[number, number, number]} first
 * @param {[number, number, number]} second
 * @returns {number}
 */
export function contrast(first, second) {
  const a = luminance(first);
  const b = luminance(second);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

/**
 * Composes a translucent colour over the colour below it.
 *
 * WHY: the badge of a state is a wash of 12 to 16 percent over the card.
 * The ratio against the wash alone would be a number that the user never
 * sees.
 *
 * @param {[number, number, number, number]} top the colour with its alpha
 * @param {[number, number, number]} bottom the colour below
 * @returns {[number, number, number]}
 */
export function compose(top, bottom) {
  const alpha = top[3] ?? 1;
  return [
    top[0] * alpha + bottom[0] * (1 - alpha),
    top[1] * alpha + bottom[1] * (1 - alpha),
    top[2] * alpha + bottom[2] * (1 - alpha),
  ];
}

/**
 * Multiplies a colour with a factor, the way the `brightness()` filter of
 * the browser does.
 *
 * WHY: the focused card is painted with `filter: brightness(1.06)`. The
 * colour that the browser writes into the picture is not the colour of the
 * style sheet, and the ratio that the user sees is the ratio of the
 * painted colour.
 *
 * @param {[number, number, number]} color
 * @param {number} factor
 * @returns {[number, number, number]}
 */
export function brighten(color, factor) {
  return [
    Math.min(255, color[0] * factor),
    Math.min(255, color[1] * factor),
    Math.min(255, color[2] * factor),
  ];
}
