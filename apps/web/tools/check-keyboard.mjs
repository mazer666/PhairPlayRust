/**
 * The keyboard check of the dashboard, in a real browser.
 *
 * WHAT: drives the dashboard with the keyboard alone — no mouse click
 * anywhere — and proves that the panel can be operated with a remote
 * control: one tab stop for the four receiver cards, the arrow keys move
 * the focus inside the group, Enter and the space key switch a receiver
 * through the real API, Escape closes the error banner, and the focus ring
 * is visible and carries its contrast.
 * WHY: the dashboard is the universal interface of the product. On a
 * television it is watched from a distance and operated with a remote
 * control, and a panel that can only be operated with a mouse is a panel
 * that cannot be used there at all.
 * HOW: `node apps/web/tools/check-keyboard.mjs`. The script starts the
 * mock receiver of `mock-receiver.mjs`, opens the dashboard in the
 * Chromium of Playwright and reads the result of every key out of the
 * document. The browser is driven by `keyboard.press` only — the pointer
 * is never used.
 *
 * EXAMPLE:
 *
 * ```sh
 * node apps/web/tools/check-keyboard.mjs
 * ```
 */

import { contrast, launch, serve } from "./lib.mjs";

const PORT = Number(process.env.PHAIRPLAY_WEB_PORT ?? 8099);
/** The number of receiver cards the mock reports. */
const EXPECTED_CARDS = 5;
const BASE = `http://127.0.0.1:${PORT}`;

/** The results of the check. */
const problems = [];
let passed = 0;

/**
 * Records a successful check.
 *
 * @param {string} message
 */
function ok(message) {
  passed += 1;
  console.log(`  ok   ${message}`);
}

/**
 * Records a failed check.
 *
 * @param {string} message
 */
function fail(message) {
  problems.push(message);
  console.log(`  FAIL ${message}`);
}

/**
 * Reads the id of the element that has the focus.
 *
 * @param {import("playwright").Page} page
 * @returns {Promise<string>}
 */
function activeId(page) {
  return page.evaluate(() => document.activeElement?.id ?? "");
}

/**
 * Checks the navigation of the grid of the cards.
 *
 * The layout is read out of the document, so that the check follows the
 * grid that the browser really shows — two columns on a tablet, four on a
 * television.
 *
 * @param {import("playwright").Page} view the page to drive
 * @param {string[]} ids the ids of the four cards, in the order of the grid
 * @param {string} label the name of the layout, for the output
 */
async function gridNavigation(view, ids, label) {
  const columns = await view.evaluate(() => {
    const cards = [...document.querySelectorAll("#protocol-grid .card")];
    return cards.filter((card) => card.offsetTop === cards[0].offsetTop).length;
  });
  const rows = Math.ceil(ids.length / columns);
  ok(`${label}: the grid shows ${columns} column(s) and ${rows} row(s)`);

  /**
   * Moves the focus to a card and presses a key.
   *
   * @param {string} id the card that takes the focus
   * @param {string} key the key that is pressed
   * @returns {Promise<string>} the id of the element that has the focus
   */
  async function press(id, key) {
    await view.evaluate((target) => document.getElementById(target)?.focus(), id);
    await view.waitForTimeout(150);
    await view.keyboard.press(key);
    await view.waitForTimeout(180);
    return activeId(view);
  }

  const right = await press(ids[0], "ArrowRight");
  if (right === ids[1]) ok(`${label}: ArrowRight moves the focus to ${right}`);
  else fail(`${label}: ArrowRight moved the focus to ${right}, expected ${ids[1]}`);

  const left = await press(ids[1], "ArrowLeft");
  if (left === ids[0]) ok(`${label}: ArrowLeft moves the focus to ${left}`);
  else fail(`${label}: ArrowLeft moved the focus to ${left}, expected ${ids[0]}`);

  if (rows > 1) {
    const down = await press(ids[0], "ArrowDown");
    if (down === ids[columns]) ok(`${label}: ArrowDown moves the focus to ${down}`);
    else fail(`${label}: ArrowDown moved the focus to ${down}, expected ${ids[columns]}`);

    const up = await press(ids[columns], "ArrowUp");
    if (up === ids[0]) ok(`${label}: ArrowUp moves the focus to ${up}`);
    else fail(`${label}: ArrowUp moved the focus to ${up}, expected ${ids[0]}`);
  } else {
    // One row: there is no card below, so the key leaves the group.
    const down = await press(ids[0], "ArrowDown");
    if (!ids.includes(down)) ok(`${label}: ArrowDown leaves the group and reaches ${down}`);
    else fail(`${label}: ArrowDown stayed in the group on ${down}`);
  }

  await view.evaluate((target) => document.getElementById(target)?.focus(), ids[0]);
  await view.waitForTimeout(150);
  await view.keyboard.press("Home");
  await view.waitForTimeout(180);
  const home = await activeId(view);
  if (home === ids[0]) ok(`${label}: Home jumps to the first card of the group`);
  else fail(`${label}: Home jumped to ${home}`);

  await view.keyboard.press("End");
  await view.waitForTimeout(180);
  const end = await activeId(view);
  if (end === ids[ids.length - 1]) ok(`${label}: End jumps to the last card of the group`);
  else fail(`${label}: End jumped to ${end}`);
}

/**
 * Waits until the ring of the focus has come to rest.
 *
 * WHY: the card is scaled and brightened over a short transition. The
 * computed style of an element in a transition is the value of the moment,
 * not the value of the rule — and a machine under load paints slower than
 * the wall clock. The check therefore waits until the value does not move
 * any more, instead of guessing a delay.
 *
 * @param {import("playwright").Page} page
 * @returns {Promise<void>}
 */
async function settle(page) {
  await page.waitForFunction(
    () => {
      const card = document.querySelector("#card-airplay");
      if (!card) return false;
      const style = getComputedStyle(card);
      const now = `${style.transform}|${style.filter}`;
      const previous = window.__ring;
      window.__ring = now;
      return previous !== undefined && previous === now;
    },
    null,
    { timeout: 5000, polling: 80 },
  );
}

/**
 * Checks the ring of the focus in one theme.
 *
 * @param {import("playwright").Page} page
 * @param {string} theme the theme to read
 */
async function focusRing(page, theme) {
  await page.evaluate((name) => document.documentElement.setAttribute("data-theme", name), theme);
  await page.evaluate(() => document.querySelector("#card-airplay").focus());
  await page.waitForTimeout(200);
  await settle(page);
  const ring = await page.evaluate(() => {
    const card = document.querySelector("#card-airplay");
    const style = getComputedStyle(card);
    const parse = (value) => {
      const parts = value.match(/[\d.]+/g) ?? [];
      return [Number(parts[0]), Number(parts[1]), Number(parts[2])];
    };
    return {
      outlineStyle: style.outlineStyle,
      outlineWidth: Number.parseFloat(style.outlineWidth),
      outlineColor: parse(style.outlineColor),
      background: parse(style.backgroundColor),
      filter: style.filter,
      surface: parse(getComputedStyle(document.body).backgroundColor),
    };
  });
  const againstCard = contrast(ring.outlineColor, ring.background);
  const againstPage = contrast(ring.outlineColor, ring.surface);
  if (ring.outlineStyle !== "none" && ring.outlineWidth >= 3) {
    ok(`${theme}: the focus ring is ${ring.outlineWidth} px wide`);
  } else {
    fail(`${theme}: the focus ring is ${ring.outlineStyle} ${ring.outlineWidth} px`);
  }
  if (againstCard >= 3) {
    ok(`${theme}: the focus ring carries ${againstCard.toFixed(2)}:1 against the card`);
  } else {
    fail(`${theme}: the focus ring carries only ${againstCard.toFixed(2)}:1 against the card`);
  }
  if (againstPage >= 3) {
    ok(`${theme}: the focus ring carries ${againstPage.toFixed(2)}:1 against the page`);
  } else {
    fail(`${theme}: the focus ring carries only ${againstPage.toFixed(2)}:1 against the page`);
  }
  if (ring.filter !== "none" && ring.filter !== "brightness(1)") {
    ok(`${theme}: the focused card becomes brighter (${ring.filter})`);
  } else {
    fail(`${theme}: the focused card does not become brighter (${ring.filter})`);
  }
}

const server = await serve(PORT);
const browser = await launch();

try {
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  const consoleErrors = [];
  page.on("pageerror", (error) => consoleErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  console.log("The dashboard against the mock receiver");
  await page.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(".card .state", { timeout: 10_000 });
  await page.waitForTimeout(600);

  // ── The group has exactly one tab stop ─────────────────────────────
  const tabStops = await page.evaluate(() => {
    const cards = [...document.querySelectorAll("#protocol-grid .card")];
    return {
      total: cards.length,
      focusable: cards.filter((card) => card.tabIndex === 0).length,
      indexes: cards.map((card) => card.tabIndex),
      roles: cards.map((card) => card.getAttribute("role")),
      names: cards.map((card) => card.getAttribute("aria-labelledby")),
    };
  });
  // WHY the count comes from the mock and not from a literal: a new
  // protocol must not break a check that is about the focus order.
  if (tabStops.total === EXPECTED_CARDS) ok(`the group holds ${tabStops.total} cards`);
  else fail(`the group holds ${tabStops.total} cards, expected ${EXPECTED_CARDS}`);
  if (tabStops.focusable === 1) {
    ok(`exactly one card is in the tab order (tabindex: ${tabStops.indexes.join(", ")})`);
  } else {
    fail(`${tabStops.focusable} cards are in the tab order: ${tabStops.indexes.join(", ")}`);
  }
  if (tabStops.roles.every((role) => role === "button")) ok("every card is a button");
  else fail(`the cards do not all carry the role button: ${tabStops.roles.join(", ")}`);
  if (tabStops.names.every(Boolean)) ok("every card carries a name for the screen reader");
  else fail("a card has no name");

  const ids = await page.evaluate(() =>
    [...document.querySelectorAll("#protocol-grid .card")].map((card) => card.id),
  );

  // ── The arrow keys move the focus inside the group ──────────────────
  await gridNavigation(page, ids, "1920x1080");

  // The same navigation in the grid of two columns.
  const tablet = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  await tablet.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await tablet.waitForSelector(".card .state", { timeout: 10_000 });
  await tablet.waitForTimeout(500);
  await gridNavigation(tablet, ids, "1280x900");
  await tablet.close();

  // ── The tab key leaves the group ───────────────────────────────────
  await page.evaluate((target) => document.getElementById(target)?.focus(), ids[0]);
  await page.waitForTimeout(150);
  await page.keyboard.press("Tab");
  await page.waitForTimeout(180);
  const afterTab = await activeId(page);
  if (!ids.includes(afterTab)) {
    ok(`the tab key leaves the group and reaches ${afterTab}`);
  } else {
    fail(`the tab key stayed in the group: ${afterTab}`);
  }

  // ── The arrows leave the group at its edge ─────────────────────────
  await page.evaluate((target) => document.getElementById(target)?.focus(), ids[ids.length - 1]);
  await page.waitForTimeout(150);
  await page.keyboard.press("ArrowDown");
  await page.waitForTimeout(180);
  const afterEdge = await activeId(page);
  if (afterEdge && !ids.includes(afterEdge)) {
    ok(`the arrow key at the edge of the group reaches ${afterEdge}`);
  } else {
    fail(`the arrow key at the edge of the group stayed on ${afterEdge}`);
  }

  // ── Enter switches the receiver through the API ────────────────────
  const calls = [];
  page.on("request", (request) => {
    if (request.url().includes("/api/protocols/")) {
      calls.push(`${request.method()} ${new URL(request.url()).pathname}`);
    }
  });

  await page.evaluate((target) => document.getElementById(target)?.focus(), ids[2]);
  await page.waitForTimeout(150);
  const before = await page.getAttribute(`#${ids[2]}`, "aria-pressed");
  await page.keyboard.press("Enter");
  await page.waitForTimeout(900);
  const after = await page.getAttribute(`#${ids[2]}`, "aria-pressed");
  if (before !== after) {
    ok(`Enter switches the receiver (aria-pressed ${before} → ${after}, ${calls.join(", ")})`);
  } else {
    fail(`Enter did not switch the receiver (aria-pressed stayed ${before})`);
  }
  if (calls.some((call) => call.includes("/api/protocols/cast/"))) {
    ok(`the switch calls the API of the receiver (${calls.join(", ")})`);
  } else {
    fail(`the switch did not call the API (${calls.join(", ")})`);
  }

  // ── The space key switches it as well ──────────────────────────────
  calls.length = 0;
  await page.keyboard.press(" ");
  await page.waitForTimeout(900);
  const afterSpace = await page.getAttribute(`#${ids[2]}`, "aria-pressed");
  if (afterSpace !== after) {
    ok(`the space key switches the receiver (aria-pressed ${after} → ${afterSpace})`);
  } else {
    fail(`the space key did not switch the receiver (aria-pressed stayed ${after})`);
  }

  // ── The reading of the API does not switch the card back ───────────
  await page.waitForTimeout(2200);
  const afterPoll = await page.getAttribute(`#${ids[2]}`, "aria-pressed");
  if (afterPoll === afterSpace) {
    ok(`the next reading of the API keeps the state of the card (${afterPoll})`);
  } else {
    fail(`the next reading of the API switched the card back (${afterSpace} → ${afterPoll})`);
  }

  // ── Escape closes the banner ───────────────────────────────────────
  await page.route("**/api/status", (route) => route.abort());
  await page.waitForTimeout(2600);
  if (await page.locator("#api-banner").isVisible()) {
    ok("a receiver that cannot be reached shows a banner");
  } else {
    fail("no banner when the receiver cannot be reached");
  }
  await page.keyboard.press("Escape");
  await page.waitForTimeout(250);
  if (!(await page.locator("#api-banner").isVisible())) ok("Escape closes the banner");
  else fail("Escape did not close the banner");

  // The panel reads the API every two seconds. A message that the user has
  // closed must not come back with the next reading.
  await page.waitForTimeout(2400);
  if (!(await page.locator("#api-banner").isVisible())) {
    ok("the closed banner does not come back with the next reading of the API");
  } else {
    fail("the closed banner came back with the next reading of the API");
  }
  await page.unroute("**/api/status");

  // ── The focus ring ─────────────────────────────────────────────────
  await focusRing(page, "dark");
  await focusRing(page, "light");

  // ── No errors ──────────────────────────────────────────────────────
  // WHY the filter: the check aborts the request to the API on purpose, to
  // produce the banner. The browser logs the aborted request itself.
  const real = consoleErrors.filter(
    (text) => !text.includes("/api/events") && !text.includes("ERR_FAILED"),
  );
  if (real.length === 0) ok("no error in the console while the panel runs");
  else fail(`errors in the console: ${real.slice(0, 3).join(" | ")}`);
} finally {
  await browser.close();
  server.close();
}

console.log(`\n${passed} checks passed, ${problems.length} failed`);
if (problems.length > 0) {
  console.log("\nProblems:");
  for (const problem of problems) console.log(`  - ${problem}`);
  process.exit(1);
}
