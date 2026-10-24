/**
 * The contrast check of the dashboard, measured in a real browser.
 *
 * WHAT: reads the colours that the browser really paints — the focus ring
 * of a card, the title of a card, the badge of a state, the counters — and
 * computes the contrast ratio of every pair with the formula of WCAG 2.1.
 * WHY: a contrast that is only claimed is a contrast that is not checked.
 * The values of the design tokens are read from the document, the
 * backgrounds that are translucent are composed over the surface below
 * them, the colours of the focused card are the ones that the filter of
 * the card paints, and the result is a number that can be compared with
 * the thresholds of WCAG: 4.5:1 for a text, 3:1 for a boundary and for a
 * graphic.
 * HOW: `node apps/web/tools/check-contrast.mjs`. The script starts the
 * mock receiver, opens the dashboard in the Chromium of Playwright in both
 * themes and prints a table.
 *
 * EXAMPLE:
 *
 * ```sh
 * node apps/web/tools/check-contrast.mjs
 * ```
 */

import { brighten, compose, contrast, launch, serve } from "./lib.mjs";

const PORT = Number(process.env.PHAIRPLAY_WEB_PORT ?? 8097);
const BASE = `http://127.0.0.1:${PORT}`;

/** The results of the check. */
const problems = [];

/**
 * Reads the colours of the dashboard out of the browser.
 *
 * @param {import("playwright").Page} page
 * @param {string} theme the theme to read
 * @returns {Promise<object>}
 */
async function measure(page, theme) {
  await page.evaluate((name) => document.documentElement.setAttribute("data-theme", name), theme);
  await page.waitForTimeout(300);
  return page.evaluate(() => {
    const read = (value) => {
      const parts = value.match(/[\d.]+/g) ?? [];
      return [
        Number(parts[0]),
        Number(parts[1]),
        Number(parts[2]),
        parts[3] === undefined ? 1 : Number(parts[3]),
      ];
    };
    const style = (selector, property) =>
      read(getComputedStyle(document.querySelector(selector))[property]);
    const card = document.querySelector("#card-airplay");
    // WHY every card: the four receivers are in four different states, and
    // every state has its own colour and its own wash.
    const badges = [...document.querySelectorAll("#protocol-grid .card")].map((element) => {
      const badge = element.querySelector(".state");
      return {
        protocol: element.dataset.protocol,
        state: badge.dataset.state,
        text: read(getComputedStyle(badge).color),
        background: read(getComputedStyle(badge).backgroundColor),
        card: read(getComputedStyle(element).backgroundColor),
      };
    });
    return {
      page: style("body", "backgroundColor"),
      card: read(getComputedStyle(card).backgroundColor),
      cardBorder: style("#card-airplay", "borderTopColor"),
      title: style("#card-airplay .card__name", "color"),
      detail: style("#card-airplay .card__fact dd", "color"),
      label: style("#card-airplay .card__fact dt", "color"),
      reason: style("#card-airplay .state__reason", "color"),
      counter: style("#card-airplay .counter__value", "color"),
      hint: style("#remote-hint", "color"),
      hintBackground: style("#remote-hint", "backgroundColor"),
      sectionNote: style(".section__note", "color"),
      switchLabel: style("#card-airplay .switch__label", "color"),
      badges,
    };
  });
}

/**
 * Reads the colours of the focused card, with the factor of its filter.
 *
 * @param {import("playwright").Page} page
 * @returns {Promise<object>}
 */
async function measureFocused(page) {
  return page.evaluate(() => {
    const read = (value) => {
      const parts = value.match(/[\d.]+/g) ?? [];
      return [
        Number(parts[0]),
        Number(parts[1]),
        Number(parts[2]),
        parts[3] === undefined ? 1 : Number(parts[3]),
      ];
    };
    const card = document.querySelector("#card-airplay");
    const style = getComputedStyle(card);
    const factor = Number((style.filter.match(/brightness\(([\d.]+)\)/) ?? [, 1])[1]);
    const of = (selector, property) =>
      read(getComputedStyle(document.querySelector(selector))[property]);
    return {
      factor,
      card: read(style.backgroundColor),
      title: of("#card-airplay .card__name", "color"),
      label: of("#card-airplay .card__fact dt", "color"),
      detail: of("#card-airplay .card__fact dd", "color"),
      badgeText: of("#card-airplay .state", "color"),
      badgeBackground: of("#card-airplay .state", "backgroundColor"),
      ring: read(style.outlineColor),
    };
  });
}

const server = await serve(PORT);
const browser = await launch();

/**
 * Waits until the ring of the focus has come to rest.
 *
 * WHY: the card is scaled and brightened over a short transition, and the
 * computed style of an element in a transition is the value of the moment.
 * The factor of the filter has to be the one of the rule, not the one of
 * a frame in between.
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

try {
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  await page.goto(`${BASE}/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(".card .state", { timeout: 10_000 });
  await page.waitForTimeout(600);

  console.log("The focus ring of the card");
  for (const theme of ["dark", "light"]) {
    await page.evaluate((name) => document.documentElement.setAttribute("data-theme", name), theme);
    await page.evaluate(() => document.querySelector("#card-airplay").focus());
    await page.waitForTimeout(200);
    await settle(page);
    const ring = await page.evaluate(() => {
      const read = (value) => {
        const parts = value.match(/[\d.]+/g) ?? [];
        return [
          Number(parts[0]),
          Number(parts[1]),
          Number(parts[2]),
          parts[3] === undefined ? 1 : Number(parts[3]),
        ];
      };
      const card = document.querySelector("#card-airplay");
      const style = getComputedStyle(card);
      return {
        ring: read(style.outlineColor),
        width: Number.parseFloat(style.outlineWidth),
        card: read(style.backgroundColor),
        page: read(getComputedStyle(document.body).backgroundColor),
        halo: read(style.boxShadow.match(/rgba?\([^)]+\)/)?.[0] ?? "rgba(0,0,0,0)"),
      };
    });
    const againstCard = contrast(ring.ring, ring.card);
    const againstPage = contrast(ring.ring, ring.page);
    const againstHalo = contrast(ring.ring, compose(ring.halo, ring.page));
    console.log(
      `  ${theme}: ring rgb(${ring.ring.slice(0, 3).join(", ")}) at ${ring.width} px — ` +
        `${againstCard.toFixed(2)}:1 against the card, ${againstPage.toFixed(2)}:1 against the page, ` +
        `${againstHalo.toFixed(2)}:1 against its halo`,
    );
    if (againstCard < 3) {
      problems.push(`${theme}: the focus ring carries only ${againstCard.toFixed(2)}:1 against the card`);
    }
    if (againstPage < 3) {
      problems.push(`${theme}: the focus ring carries only ${againstPage.toFixed(2)}:1 against the page`);
    }
  }

  console.log("\nThe texts of the card");
  for (const theme of ["dark", "light"]) {
    const values = await measure(page, theme);
    const hintBackground = compose(values.hintBackground, values.page);
    console.log(`  ${theme}:`);
    const rows = [
      ["the title of the card", values.title, values.card, 4.5],
      ["the value of a fact", values.detail, values.card, 4.5],
      ["the name of a fact", values.label, values.card, 4.5],
      ["the reason of the state", values.reason, values.card, 4.5],
      ["the value of a counter", values.counter, values.card, 4.5],
      ["the label of the switch", values.switchLabel, values.card, 4.5],
      ["the note of the section", values.sectionNote, values.page, 4.5],
      ["the hint for the remote", values.hint, hintBackground, 4.5],
      ["the border of the card", values.cardBorder, values.card, 3],
    ];
    for (const [name, foreground, background, threshold] of rows) {
      const ratio = contrast(foreground, background);
      const verdict = ratio >= threshold ? "ok  " : "FAIL";
      console.log(`    ${verdict} ${name.padEnd(26)} ${ratio.toFixed(2)}:1 (needed ${threshold}:1)`);
      if (ratio < threshold) {
        problems.push(`${theme}: ${name} carries only ${ratio.toFixed(2)}:1, needed ${threshold}:1`);
      }
    }
    for (const badge of values.badges) {
      const background = compose(badge.background, badge.card);
      const ratio = contrast(badge.text, background);
      const verdict = ratio >= 4.5 ? "ok  " : "FAIL";
      console.log(`    ${verdict} ${`the badge "${badge.state}"`.padEnd(26)} ${ratio.toFixed(2)}:1 (needed 4.5:1)`);
      if (ratio < 4.5) {
        problems.push(`${theme}: the badge "${badge.state}" carries only ${ratio.toFixed(2)}:1`);
      }
    }
  }

  console.log("\nThe focused card, painted with brightness(1.06)");
  for (const theme of ["dark", "light"]) {
    await page.evaluate((name) => document.documentElement.setAttribute("data-theme", name), theme);
    await page.evaluate(() => document.querySelector("#card-airplay").focus());
    await page.waitForTimeout(200);
    await settle(page);
    const painted = await measureFocused(page);
    const card = brighten(painted.card, painted.factor);
    const badgeBackground = brighten(compose(painted.badgeBackground, painted.card), painted.factor);
    const rows = [
      ["the title of the card", brighten(painted.title, painted.factor), card, 4.5],
      ["the name of a fact", brighten(painted.label, painted.factor), card, 4.5],
      ["the value of a fact", brighten(painted.detail, painted.factor), card, 4.5],
      ["the badge of the state", brighten(painted.badgeText, painted.factor), badgeBackground, 4.5],
      ["the focus ring", brighten(painted.ring, painted.factor), card, 3],
    ];
    for (const [name, foreground, background, threshold] of rows) {
      const ratio = contrast(foreground, background);
      const verdict = ratio >= threshold ? "ok  " : "FAIL";
      console.log(`    ${verdict} ${theme}: ${name.padEnd(22)} ${ratio.toFixed(2)}:1 (needed ${threshold}:1)`);
      if (ratio < threshold) {
        problems.push(`${theme}: the focused ${name} carries only ${ratio.toFixed(2)}:1`);
      }
    }
  }
} finally {
  await browser.close();
  server.close();
}

console.log("");
if (problems.length > 0) {
  console.log("Problems:");
  for (const problem of problems) console.log(`  - ${problem}`);
  process.exit(1);
}
console.log("every measured pair reaches its threshold");
