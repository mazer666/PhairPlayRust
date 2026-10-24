/**
 * The layout check of the dashboard, in a real browser.
 *
 * WHAT: opens the dashboard in the resolutions of the devices the panel is
 * used on — a phone, a tablet, a television with 720 lines, a television
 * with 1080 lines, an ultra-wide desktop — and measures whether the four
 * receiver cards stand in a grid, whether the page scrolls sideways and
 * whether the four cards are visible without scrolling.
 * WHY: the dashboard is the universal interface of the product. It has to
 * work on a television that is watched from a distance and on a phone that
 * is held in the hand, and a panel that has to be dragged sideways on a
 * television is a panel that cannot be read.
 * HOW: `node apps/web/tools/check-layout.mjs`. The script starts the mock
 * receiver, opens the dashboard in the Chromium of Playwright and reads the
 * geometry out of the document. It also checks the hint for the remote
 * control: it appears on a device without a mouse and disappears on a
 * device with one.
 *
 * EXAMPLE:
 *
 * ```sh
 * node apps/web/tools/check-layout.mjs
 * ```
 */

import { launch, serve } from "./lib.mjs";

const PORT = Number(process.env.PHAIRPLAY_WEB_PORT ?? 8096);
const BASE = `http://127.0.0.1:${PORT}`;

/** The resolutions the check measures. */
const SCREENS = [
  [360, 800],
  [768, 1024],
  [1280, 720],
  [1920, 1080],
  [2560, 1440],
  [3840, 2160],
];

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

const server = await serve(PORT);
const browser = await launch();

try {
  console.log("The grid of the receivers");
  for (const [width, height] of SCREENS) {
    const view = await browser.newPage({ viewport: { width, height } });
    await view.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
    await view.waitForSelector(".card .state", { timeout: 10_000 });
    await view.waitForTimeout(500);
    const layout = await view.evaluate(() => {
      const grid = document.getElementById("protocol-grid");
      const section = grid.closest(".section");
      const cards = [...document.querySelectorAll(".card")];
      const rows = new Set(cards.map((card) => card.offsetTop));
      const boxes = cards.map((card) => card.getBoundingClientRect());
      // WHY the boxes of the cards and not the box of the grid: the grid
      // carries a padding that holds the card that has the focus, and the
      // user reads the cards, not the padding of the grid.
      const top = Math.min(...boxes.map((box) => box.top));
      const bottom = Math.max(...boxes.map((box) => box.bottom));
      return {
        overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        columns: cards.length / rows.size,
        top: Math.round(top),
        bottom: Math.round(bottom),
        viewport: window.innerHeight,
        gridScroll: grid.scrollHeight - grid.clientHeight,
        sectionScroll: section.scrollHeight - section.clientHeight,
        title: getComputedStyle(cards[0].querySelector(".card__name")).fontSize,
        state: getComputedStyle(cards[0].querySelector(".state")).fontSize,
        detail: getComputedStyle(cards[0].querySelector(".card__fact dd")).fontSize,
        label: getComputedStyle(cards[0].querySelector(".card__fact dt")).fontSize,
      };
    });
    if (layout.overflow <= 1) ok(`no horizontal overflow at ${width}×${height}`);
    else fail(`horizontal overflow of ${layout.overflow} px at ${width}×${height}`);

    // The focused card is scaled up by two percent. The scale must not
    // push the panel sideways.
    await view.evaluate(() => document.querySelector("#card-airplay").focus());
    await view.waitForTimeout(400);
    const focusedOverflow = await view.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    if (focusedOverflow <= 1) {
      ok(`no horizontal overflow at ${width}×${height} with a focused card`);
    } else {
      fail(`horizontal overflow of ${focusedOverflow} px at ${width}×${height} with a focused card`);
    }

    // WHY this check: the card that has the focus is two percent bigger
    // than the others. A card that reaches out of the box of the grid
    // makes the grid scrollable — the page would move by a few pixels
    // every time the focus moves to a card, and a panel that moves under
    // the eyes of the user on a television is a panel that is hard to
    // read.
    const focusScroll = await view.evaluate(() => {
      const grid = document.getElementById("protocol-grid");
      const section = grid.closest(".section");
      return {
        grid: grid.scrollHeight - grid.clientHeight,
        section: section.scrollHeight - section.clientHeight,
      };
    });
    if (focusScroll.grid <= 0 && focusScroll.section <= 0) {
      ok(`the grid does not scroll at ${width}×${height} with a focused card`);
    } else {
      fail(
        `the grid scrolls ${focusScroll.grid} px and the section ${focusScroll.section} px ` +
          `at ${width}×${height} with a focused card`,
      );
    }

    // WHY this check: the three counters of a card are one row. A label
    // that needs two lines — "Packet loss" does not fit into a third of a
    // card on a television with 720 lines — must not push its value down,
    // and no label may be cut with an ellipsis: a name that the user
    // cannot read is worse than a name that takes two lines.
    const counters = await view.evaluate(() => {
      const card = document.querySelector("#card-airplay");
      const labels = [...card.querySelectorAll(".counter__label")];
      const values = [...card.querySelectorAll(".counter__value")];
      return {
        cut: labels.filter((label) => label.scrollWidth > label.clientWidth + 1).length,
        tops: [...new Set(values.map((value) => Math.round(value.getBoundingClientRect().top)))],
      };
    });
    if (counters.cut === 0 && counters.tops.length === 1) {
      ok(`the counters of a card stand on one line at ${width}×${height}`);
    } else {
      fail(
        `the counters of a card do not stand on one line at ${width}×${height} ` +
          `(${counters.cut} label(s) cut, ${counters.tops.length} value rows)`,
      );
    }

    const sizes = `title ${layout.title}, state ${layout.state}, detail ${layout.detail}, label ${layout.label}`;    if (layout.bottom <= layout.viewport) {
      ok(
        `the four receivers are visible without scrolling at ${width}×${height} ` +
          `(${layout.columns} column(s), ${layout.top}–${layout.bottom} of ${layout.viewport}; ${sizes})`,
      );
    } else {
      console.log(
        `  note the four receivers need ${layout.bottom - layout.viewport} px of scrolling at ${width}×${height} ` +
          `(${layout.columns} column(s), ${layout.top}–${layout.bottom} of ${layout.viewport}; ${sizes})`,
      );
    }
    await view.close();
  }

  console.log("\nThe hint for the remote control");
  const remote = await browser.newContext({ viewport: { width: 1280, height: 720 }, hasTouch: true });
  const remotePage = await remote.newPage();
  await remotePage.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await remotePage.waitForSelector(".card .state", { timeout: 10_000 });
  await remotePage.waitForTimeout(400);
  if (await remotePage.locator("#remote-hint").isVisible()) {
    ok("a device without a mouse sees the hint for the arrow keys");
  } else {
    fail("no hint for the arrow keys on a device without a mouse");
  }
  await remote.close();

  const mouse = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const mousePage = await mouse.newPage();
  await mousePage.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await mousePage.waitForSelector(".card .state", { timeout: 10_000 });
  await mousePage.waitForTimeout(400);
  if (!(await mousePage.locator("#remote-hint").isVisible())) {
    ok("a device with a mouse does not see the hint");
  } else {
    fail("the hint is shown on a device with a mouse");
  }
  await mouse.close();

  console.log("\nThe reduced motion");
  const still = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  await still.emulateMedia({ reducedMotion: "reduce" });
  await still.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await still.waitForSelector(".card .state", { timeout: 10_000 });
  await still.waitForTimeout(400);
  await still.evaluate(() => document.querySelector("#card-airplay").focus());
  await still.waitForTimeout(400);
  const motion = await still.evaluate(() => {
    const style = getComputedStyle(document.querySelector("#card-airplay"));
    return {
      transform: style.transform,
      outline: style.outlineStyle,
      width: Number.parseFloat(style.outlineWidth),
      duration: getComputedStyle(document.documentElement).getPropertyValue("--duration-base").trim(),
    };
  });
  if (motion.transform === "none" && motion.outline !== "none" && motion.width >= 3) {
    ok(`the ring stays, the movement stops (${motion.outline} ${motion.width} px, transform ${motion.transform})`);
  } else {
    fail(`the focus under reduced motion is ${motion.outline} ${motion.width} px, transform ${motion.transform}`);
  }
  await still.close();
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
