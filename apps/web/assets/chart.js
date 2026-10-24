/**
 * The throughput graph of the dashboard.
 *
 * WHAT: a small line graph that draws the frame rate or the throughput of the
 * running session over the last minutes.
 * WHY: the receiver already knows the numbers; the graph is what turns them
 * into the one thing a person notices — "is the stream healthy, or is it
 * stuttering?".
 * HOW: plain SVG built with the DOM, no library, no canvas. The drawing uses
 * the design tokens of theme.css, so the graph follows the theme without a
 * second palette. The wording comes from the language file.
 */

import { t } from "./i18n.js";

/** The geometry of the drawing area inside the 640 × 280 view box. */
const BOX = { width: 640, height: 280, left: 74, right: 18, top: 18, bottom: 30 };

/**
 * Rounds a maximum up to a readable grid value.
 *
 * WHY the fine steps: a frame rate of 61 fps has to fit under a maximum of
 * 80, not under 100 — otherwise the line runs through the middle of the
 * plot and the graph flattens the signal it is supposed to show.
 *
 * @param {number} value
 * @returns {number}
 */
function niceMax(value) {
  if (!Number.isFinite(value) || value <= 0) return 1;
  const exponent = Math.floor(Math.log10(value));
  const base = 10 ** exponent;
  const scaled = value / base;
  const steps = [1, 1.2, 1.5, 2, 2.5, 3, 4, 5, 6, 8, 10];
  const step = steps.find((candidate) => scaled <= candidate) ?? 10;
  return step * base;
}

/**
 * Escapes text that is written into the markup of the graph.
 *
 * @param {string} value
 * @returns {string}
 */
function escapeText(value) {
  return String(value).replace(/[&<>"']/g, (character) => {
    const entities = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };
    return entities[character];
  });
}

/**
 * The graph of one metric over time.
 */
export class TimeSeriesChart {
  /**
   * @param {SVGElement} svg the empty svg element of the panel
   * @param {{formatValue: (value: number) => string, formatAxis: (value: number) => string,
   *   formatTime: (ms: number) => string}} options
   */
  constructor(svg, options) {
    this.svg = svg;
    this.formatValue = options.formatValue;
    this.formatAxis = options.formatAxis ?? options.formatValue;
    this.formatTime = options.formatTime;
    this.metric = "fps";
    this.samples = [];
    this.summary = { current: null, max: null, mean: null, count: 0 };
  }

  /**
   * Chooses the metric that is drawn.
   *
   * @param {"fps"|"bytes"} metric
   */
  setMetric(metric) {
    this.metric = metric;
  }

  /**
   * Reads the value of the active metric from a sample.
   *
   * @param {{fps: number, bytesPerSecond: number}} sample
   * @returns {number}
   */
  valueOf(sample) {
    return this.metric === "bytes" ? sample.bytesPerSecond : sample.fps;
  }

  /**
   * Draws the graph and computes the summary of the visible data.
   *
   * @param {Array<{t: number, fps: number, bytesPerSecond: number}>} samples
   * @returns {{current: number|null, max: number|null, mean: number|null, count: number}}
   */
  render(samples) {
    this.samples = Array.isArray(samples) ? samples.slice(-180) : [];
    const values = this.samples.map((sample) => this.valueOf(sample));
    const count = values.length;
    if (count === 0) {
      this.summary = { current: null, max: null, mean: null, count: 0 };
      this.svg.innerHTML = this.emptyMarkup();
      return this.summary;
    }
    const max = niceMax(Math.max(...values, 0));
    const sum = values.reduce((total, value) => total + value, 0);
    this.summary = {
      current: values[values.length - 1],
      max: Math.max(...values),
      mean: sum / count,
      count,
    };
    this.svg.innerHTML = this.markup(values, max);
    return this.summary;
  }

  /**
   * Builds the markup of the graph.
   *
   * @param {number[]} values
   * @param {number} max
   * @returns {string}
   */
  markup(values, max) {
    const plotWidth = BOX.width - BOX.left - BOX.right;
    const plotHeight = BOX.height - BOX.top - BOX.bottom;
    const lastIndex = values.length - 1;
    const xAt = (index) =>
      lastIndex === 0
        ? BOX.left + plotWidth / 2
        : BOX.left + (index / lastIndex) * plotWidth;
    const yAt = (value) => BOX.top + plotHeight - (value / max) * plotHeight;

    const points = values.map((value, index) => `${xAt(index)},${yAt(value)}`);
    const line = `M${points.join(" L")}`;
    const area = `${line} L${xAt(lastIndex)},${BOX.top + plotHeight} L${xAt(0)},${
      BOX.top + plotHeight
    } Z`;

    const gridSteps = [0, 0.25, 0.5, 0.75, 1];
    const peak = Math.max(...values, 0);
    // WHY the special case: a receiver that runs but does not stream has a
    // rate of zero everywhere. A scale of 0 … 1 with the labels 0, 0, 1, 1, 1
    // would be a lie about the resolution — one label is the truth.
    const steps = peak === 0 ? [0] : gridSteps;
    const grid = steps
      .map((step) => {
        const y = yAt(max * step);
        return `<line x1="${BOX.left}" y1="${y}" x2="${BOX.width - BOX.right}" y2="${y}" />`;
      })
      .join("");

    const labels = steps
      .map((step) => {
        const value = max * step;
        const y = yAt(value);
        const label = escapeText(this.formatAxis(value));
        return `<text x="${BOX.left - 8}" y="${y + 3}" text-anchor="end">${label}</text>`;
      })
      .join("");

    const first = this.samples[0];
    const last = this.samples[lastIndex];
    const axis =
      labels +
      `<text x="${BOX.left}" y="${BOX.height - 10}">${escapeText(this.formatTime(first.t))}</text>` +
      `<text x="${BOX.width - BOX.right}" y="${BOX.height - 10}" text-anchor="end">${escapeText(
        this.formatTime(last.t),
      )}</text>`;

    return (
      `<defs><linearGradient id="phr-chart-fill" x1="0" y1="0" x2="0" y2="1">` +
      `<stop offset="0%" stop-color="var(--accent)" stop-opacity="0.30" />` +
      `<stop offset="100%" stop-color="var(--accent)" stop-opacity="0.02" />` +
      `</linearGradient></defs>` +
      `<g class="chart__grid">${grid}</g>` +
      `<path class="chart__area" d="${area}" />` +
      `<path class="chart__line" d="${line}" />` +
      `<circle class="chart__dot" cx="${xAt(lastIndex)}" cy="${yAt(values[lastIndex])}" r="3.5" />` +
      `<g class="chart__axis">${axis}</g>`
    );
  }

  /**
   * Builds the markup of an empty graph.
   *
   * @returns {string}
   */
  emptyMarkup() {
    const middle = BOX.height / 2;
    return (
      `<g class="chart__grid"><line x1="${BOX.left}" y1="${BOX.top + BOX.height - BOX.top - BOX.bottom}" ` +
      `x2="${BOX.width - BOX.right}" y2="${BOX.top + BOX.height - BOX.top - BOX.bottom}" /></g>` +
      `<text class="chart__empty" x="${BOX.width / 2}" y="${middle}" text-anchor="middle">` +
      `${escapeText(t("chart.empty"))}</text>`
    );
  }
}
