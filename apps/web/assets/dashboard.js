/**
 * The PhairPlayRust dashboard.
 *
 * WHAT: reads the local receiver API and paints the control panel — the
 * five receivers, the live sessions, the throughput graph and the codec
 * reference.
 * WHY: the panel is the only place where the state of the receiver is
 * visible, so it has to be unambiguous. "Stopped" and "switched off" are
 * different statements and are shown as different things, and a receiver
 * that cannot be reached says so instead of showing zeroes. On a television
 * the panel is read from three metres away and operated with a remote
 * control, so the cards are the controls and the focus is always visible.
 * HOW: a poll loop refreshes the REST endpoints, the event channel pushes
 * changes as soon as they happen, and every value is normalised in
 * normalize.js before it reaches this file. The wording comes from the
 * language file, the tab order from the focus group of the cards.
 */

import { PROTOCOLS, ReceiverApi } from "./api.js";
import { TimeSeriesChart } from "./chart.js";
import { renderCodecs } from "./codecs.js";
import { formatBytes, formatClock, formatCount, formatDuration, formatFps, formatRate } from "./format.js";
import { FocusGroup } from "./focus-group.js";
import { t } from "./i18n.js";
import { ProtocolCards } from "./protocol-cards.js";
import { RateTracker } from "./rates.js";
import { watchRemoteHint } from "./remote-hint.js";
import { renderSessions } from "./sessions.js";

const api = new ReceiverApi();
const tracker = new RateTracker();

let protocols = new Map();
let counters = {};
let chartScope = "all";
let connection = "connecting";
let eventSocket = null;
let refreshPending = false;
/** The message the user has closed, so that it does not come back. */
let dismissed = null;

const chart = new TimeSeriesChart(document.getElementById("chart"), {
  formatValue: (value) => (chart.metric === "bytes" ? formatRate(value) : formatFps(value)),
  formatAxis: (value) => (chart.metric === "bytes" ? formatBytes(value) : String(Math.round(value))),
  formatTime: formatClock,
});

const cards = new ProtocolCards({
  grid: document.getElementById("protocol-grid"),
  template: document.getElementById("protocol-card-template"),
  api,
  onError: (message) => showBanner(message),
  onEscape: () => closeBanner(),
});

/**
 * The two buttons of the metric of the graph are one widget for the
 * keyboard.
 *
 * WHY a group and not two tab stops: a remote control has no Tab key, and
 * the two buttons belong together — the left one and the right one of a
 * pair. The arrow keys move between them, the Tab key reaches the pair as
 * a whole.
 *
 * WHY `activate: false`: the buttons are real buttons. The browser presses
 * them on Enter and on the space key, and the group must not press them a
 * second time.
 */
const metricGroup = new FocusGroup(document.getElementById("metric-group"), {
  activate: false,
});

/**
 * Shows the banner that reports a problem with the local API.
 *
 * WHY the dismissed text: a message that the user has closed must not come
 * back on the next reading of the API. The panel reads every two seconds,
 * and a message that pops up again and again is a message that cannot be
 * closed — on a television, where the banner is the only thing between the
 * user and the panel, that would be a dead end. The dismissal is dropped
 * as soon as a reading succeeds, so a problem that returns is reported
 * again.
 *
 * @param {string|null} message
 * @returns {boolean} whether the banner is shown
 */
function showBanner(message) {
  const banner = document.getElementById("api-banner");
  const text = document.getElementById("api-banner-text");
  if (!message) {
    banner.hidden = true;
    dismissed = null;
    return false;
  }
  if (dismissed === message) return false;
  text.textContent = message;
  banner.hidden = false;
  return true;
}

/**
 * Hides the banner and remembers that the user closed it.
 *
 * WHY the escape key belongs to the banner: a message that blocks the panel
 * and cannot be dismissed is a dead end on a television, where there is no
 * right mouse button to click.
 *
 * @returns {boolean} whether a message was closed
 */
function closeBanner() {
  const banner = document.getElementById("api-banner");
  if (banner.hidden) return false;
  dismissed = document.getElementById("api-banner-text").textContent;
  banner.hidden = true;
  return true;
}

/**
 * Updates the indicator that tells the user how fresh the data is.
 */
function renderConnection() {
  const pill = document.getElementById("live-pill");
  const label = document.getElementById("live-label");
  if (connection === "live") {
    pill.dataset.link = "open";
    label.textContent = t("header.connection.live");
  } else if (connection === "polling") {
    pill.dataset.link = "open";
    label.textContent = t("header.connection.polling");
  } else if (connection === "offline") {
    pill.dataset.link = "lost";
    label.textContent = t("header.connection.offline");
  } else {
    pill.dataset.link = "lost";
    label.textContent = t("header.connection.connecting");
  }
}

/**
 * Reads everything that can change and paints it.
 */
async function refresh() {
  // WHY the number: the reading below is requested now, and a reading that
  // is older than the last change of the user must not switch a card back.
  const revision = cards.revision;
  try {
    const [status, sessions, metrics] = await Promise.all([
      api.status(),
      api.sessions(),
      api.metrics(),
    ]);
    const next = new Map();
    for (const entry of status.protocols) {
      next.set(entry.id, { ...entry, counters: metrics.protocols[entry.id] ?? null });
    }
    for (const protocol of PROTOCOLS) {
      if (!next.has(protocol.id)) {
        next.set(protocol.id, {
          id: protocol.id,
          name: null,
          state: "unknown",
          enabled: false,
          tcpPort: null,
          udpPort: null,
          peers: 0,
          lastError: null,
          counters: metrics.protocols[protocol.id] ?? null,
        });
      }
    }
    protocols = next;
    counters = metrics.protocols;
    // The server reports the cumulative counters; the rate of the graph is
    // the difference between two readings.
    tracker.update(metrics.protocols);
    cards.update(protocols, counters, revision);
    renderSessions(sessions, counters);
    renderChart();
    renderUptime(status.uptimeMs);
    if (connection !== "live") connection = "polling";
    showBanner(null);
  } catch (error) {
    connection = "offline";
    showBanner(t("error.apiUnreachableWithReason", { reason: error.message }));
  }
  renderConnection();
}

/**
 * Shows how long the receiver has been running.
 *
 * @param {number|null} uptimeMs
 */
function renderUptime(uptimeMs) {
  const element = document.getElementById("build-uptime");
  if (!element) return;
  element.textContent =
    typeof uptimeMs === "number" && Number.isFinite(uptimeMs)
      ? formatDuration(uptimeMs / 1000)
      : t("value.unknown");
}

/**
 * Draws the graph with the samples of the selected scope.
 */
function renderChart() {
  const summary = chart.render(tracker.samplesFor(chartScope));
  const format = chart.metric === "bytes" ? formatRate : formatFps;
  const write = (id, value) => {
    document.getElementById(id).textContent = value === null ? t("value.unknown") : format(value);
  };
  write("chart-current", summary.current);
  write("chart-max", summary.max);
  write("chart-mean", summary.mean);
}

/**
 * Switches the metric of the graph.
 *
 * @param {"fps"|"bytes"} metric
 */
function selectMetric(metric) {
  chart.setMetric(metric);
  document.getElementById("metric-fps").setAttribute("aria-pressed", String(metric === "fps"));
  document
    .getElementById("metric-bytes")
    .setAttribute("aria-pressed", String(metric === "bytes"));
  document.getElementById("chart-note").textContent = t(
    metric === "bytes" ? "chart.note.bytes" : "chart.note.fps",
  );
  renderChart();
}

/**
 * Switches the theme and remembers the choice.
 */
function toggleTheme() {
  const root = document.documentElement;
  const next = root.getAttribute("data-theme") === "light" ? "dark" : "light";
  root.setAttribute("data-theme", next);
  try {
    localStorage.setItem("phairplayrust-theme", next);
  } catch (error) {
    /* A blocked storage must not break the switch. */
  }
  const label = document.getElementById("theme-toggle-label");
  label.textContent = t(next === "light" ? "header.theme.toDark" : "header.theme.toLight");
  document.getElementById("theme-toggle").setAttribute("title", label.textContent);
}

/**
 * Reads the build metadata of the receiver.
 */
async function loadBuildInfo() {
  try {
    const info = await api.buildInfo();
    const version = info.version ?? info.app_version;
    const commit = info.commit ?? info.git_commit ?? info.commit_hash;
    const target = info.target ?? info.target_triple;
    const write = (id, value) => {
      document.getElementById(id).textContent = value ? String(value) : t("value.unknown");
    };
    write("build-version", version);
    write("build-commit", commit ? String(commit).slice(0, 7) : null);
    write("build-target", target);
  } catch (error) {
    /* The header keeps its placeholders; the banner reports the problem. */
  }
}

/**
 * Reads the name the receiver advertises and shows it in the header.
 */
async function loadDeviceName() {
  try {
    const config = await api.config();
    const device = config?.device ?? {};
    const name = device.name ?? device.device_name;
    if (name) {
      document.getElementById("brand-device").textContent = name;
      document.getElementById("brand-tagline").textContent = t("brand.taglineWithDevice", { name });
    }
  } catch (error) {
    /* Without a configuration the generic tagline stays. */
  }
}

/**
 * Closes the banner when the user presses the escape key.
 *
 * WHY on the document and not only on the banner: the focus can be on a
 * card, on a button or nowhere at all, and the escape key has to close the
 * message in every case.
 *
 * @param {KeyboardEvent} event
 */
function onEscape(event) {
  if (event.key !== "Escape") return;
  if (closeBanner()) event.preventDefault();
}

/**
 * Starts the panel: cards, first reading, poll loop and event channel.
 */
function start() {
  cards.build();
  metricGroup.setItems([...document.querySelectorAll("#metric-group [data-focus-item]")]);
  renderConnection();
  renderCodecs();
  loadBuildInfo();
  loadDeviceName();
  watchRemoteHint(document.getElementById("remote-hint"));

  document.getElementById("theme-toggle").addEventListener("click", toggleTheme);
  document.getElementById("banner-dismiss").addEventListener("click", closeBanner);
  document.getElementById("metric-fps").addEventListener("click", () => selectMetric("fps"));
  document.getElementById("metric-bytes").addEventListener("click", () => selectMetric("bytes"));
  document.getElementById("chart-scope").addEventListener("change", (event) => {
    chartScope = event.target.value;
    renderChart();
  });
  document.addEventListener("keydown", onEscape);

  refresh();
  window.setInterval(refresh, 2000);

  try {
    eventSocket = api.events(
      () => {
        if (refreshPending) return;
        refreshPending = true;
        window.setTimeout(() => {
          refreshPending = false;
          refresh();
        }, 250);
      },
      (state) => {
        if (state === "open") {
          connection = "live";
        } else if (connection !== "offline") {
          connection = "polling";
        }
        renderConnection();
      },
    );
  } catch (error) {
    connection = "polling";
    renderConnection();
  }
}

start();
