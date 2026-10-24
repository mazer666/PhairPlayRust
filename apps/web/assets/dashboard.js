/**
 * The PhairPlayRust dashboard.
 *
 * WHAT: reads the local receiver API and paints the control panel — the
 * four receivers, the live sessions, the throughput graph and the codec
 * reference.
 * WHY: the panel is the only place where the state of the receiver is
 * visible, so it has to be unambiguous. "Stopped" and "switched off" are
 * different statements and are shown as different things, and a receiver
 * that cannot be reached says so instead of showing zeroes.
 * HOW: a poll loop refreshes the REST endpoints, the event channel pushes
 * changes as soon as they happen, and every value is normalised in
 * normalize.js before it reaches this file.
 */

import { PROTOCOLS, ReceiverApi } from "./api.js";
import { TimeSeriesChart } from "./chart.js";
import { renderCodecs } from "./codecs.js";
import { RateTracker } from "./rates.js";
import { renderSessions } from "./sessions.js";
import { resolveState } from "./states.js";
import { formatBytes, formatClock, formatCount, formatDuration, formatFps, formatRate } from "./format.js";

const api = new ReceiverApi();
const cards = new Map();
const tracker = new RateTracker();

let protocols = new Map();
let counters = {};
let chartScope = "all";
let connection = "connecting";
let eventSocket = null;
let refreshPending = false;

const chart = new TimeSeriesChart(document.getElementById("chart"), {
  formatValue: (value) => (chart.metric === "bytes" ? formatRate(value) : formatFps(value)),
  formatAxis: (value) => (chart.metric === "bytes" ? formatBytes(value) : String(Math.round(value))),
  formatTime: formatClock,
});

/**
 * Reads a named field of an element.
 *
 * @param {Element} root
 * @param {string} name
 * @returns {Element|null}
 */
function field(root, name) {
  return root.querySelector(`[data-field="${name}"]`);
}

/**
 * Sets the text of a named field.
 *
 * @param {Element} root
 * @param {string} name
 * @param {string} value
 */
function setField(root, name, value) {
  const element = field(root, name);
  if (element) element.textContent = value;
}

/**
 * Builds the four receiver cards from the template in the document.
 */
function buildCards() {
  const template = document.getElementById("protocol-card-template");
  const grid = document.getElementById("protocol-grid");
  const fragment = document.createDocumentFragment();
  for (const protocol of PROTOCOLS) {
    const card = template.content.firstElementChild.cloneNode(true);
    card.dataset.protocol = protocol.id;
    setField(card, "name", protocol.name);
    card.querySelector("use").setAttribute("href", protocol.glyph);
    const reason = field(card, "reason");
    reason.id = `reason-${protocol.id}`;
    const toggle = card.querySelector(".switch");
    toggle.setAttribute("aria-label", `${protocol.name} receiver`);
    toggle.setAttribute("aria-describedby", reason.id);
    toggle.addEventListener("click", () => onToggle(protocol.id, toggle));
    cards.set(protocol.id, card);
    fragment.append(card);
  }
  grid.append(fragment);
}

/**
 * Paints one receiver card.
 *
 * @param {string} id
 * @param {object} data the normalised status of the receiver
 * @param {object|null} counter the live counters of the receiver
 */
function renderCard(id, data, counter) {
  const card = cards.get(id);
  if (!card || !data) return;
  const view = resolveState(data);
  const badge = field(card, "state");
  badge.dataset.state = view.tone;
  setField(card, "state-label", view.label);
  const reason = field(card, "reason");
  // WHY the class: the server reports a protocol that is enabled but has no
  // engine as the state `idle` with the reason in `last_error`. The reason
  // is the failure — it is shown in the colour of a failure.
  reason.className = data.lastError ? "state__reason state__reason--error" : "state__reason";
  reason.textContent = data.lastError
    ? `${view.reason} Last error: ${data.lastError}`
    : view.reason;

  const port = field(card, "port");
  port.textContent = data.tcpPort ? String(data.tcpPort) : "—";
  if (data.udpPort) {
    const suffix = document.createElement("span");
    suffix.className = "subtle";
    suffix.textContent = ` · UDP ${data.udpPort}`;
    port.append(suffix);
  }

  setField(card, "peers", formatCount(data.peers));

  const toggle = card.querySelector(".switch");
  toggle.setAttribute("aria-checked", String(data.enabled));
  setField(card, "switch-label", data.enabled ? "On" : "Off");

  if (counter) {
    setField(card, "frames", formatCount(counter.videoFrames));
    setField(card, "bytes", formatBytes(counter.bytesReceived));
    const loss = field(card, "loss");
    loss.textContent = formatCount(counter.packetsLost);
    loss.dataset.tone = counter.packetsLost > 0 ? "danger" : "";
  }
}

/**
 * Switches a receiver on or off and reports the result.
 *
 * @param {string} id
 * @param {HTMLButtonElement} toggle
 */
async function onToggle(id, toggle) {
  const data = protocols.get(id);
  if (!data) return;
  const wanted = !data.enabled;
  toggle.disabled = true;
  try {
    const status = await api.setProtocolEnabled(id, wanted);
    // WHY optimistic: the server answers with the status after the action.
    // It is used right away; the next poll confirms it.
    data.enabled = wanted;
    if (status) {
      Object.assign(data, status);
    }
    renderCard(id, data, counters[id]);
  } catch (error) {
    showBanner(
      `The receiver could not be switched ${wanted ? "on" : "off"}: ${error.message}`,
    );
  } finally {
    toggle.disabled = false;
  }
}

/**
 * Shows the banner that reports a problem with the local API.
 *
 * @param {string|null} message
 */
function showBanner(message) {
  const banner = document.getElementById("api-banner");
  const text = document.getElementById("api-banner-text");
  if (message) {
    text.textContent = message;
    banner.hidden = false;
  } else {
    banner.hidden = true;
  }
}

/**
 * Updates the indicator that tells the user how fresh the data is.
 */
function renderConnection() {
  const pill = document.getElementById("live-pill");
  const label = document.getElementById("live-label");
  if (connection === "live" || connection === "polling") {
    pill.dataset.link = "open";
    label.textContent = connection === "live" ? "Live" : "Polling";
  } else if (connection === "offline") {
    pill.dataset.link = "lost";
    label.textContent = "Offline";
  } else {
    pill.dataset.link = "lost";
    label.textContent = "Connecting";
  }
}

/**
 * Reads everything that can change and paints it.
 */
async function refresh() {
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
    for (const protocol of PROTOCOLS) {
      const data = protocols.get(protocol.id);
      renderCard(protocol.id, data, data.counters);
    }
    renderSessions(sessions, counters);
    renderChart();
    renderUptime(status.uptimeMs);
    if (connection !== "live") connection = "polling";
    showBanner(null);
  } catch (error) {
    connection = "offline";
    showBanner(
      `The dashboard cannot reach the local API (${error.message}). ` +
        "Start the receiver and reload this page.",
    );
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
      : "—";
}

/**
 * Draws the graph with the samples of the selected scope.
 */
function renderChart() {
  const summary = chart.render(tracker.samplesFor(chartScope));
  const format = chart.metric === "bytes" ? formatRate : formatFps;
  const write = (id, value) => {
    document.getElementById(id).textContent = value === null ? "—" : format(value);
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
  document.getElementById("chart-note").textContent =
    metric === "bytes"
      ? "Bytes per second of the running session, derived from the counters of the receiver."
      : "Frames per second of the running session, derived from the counters of the receiver.";
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
  label.textContent = next === "light" ? "Switch to the dark theme" : "Switch to the light theme";
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
      document.getElementById(id).textContent = value ? String(value) : "—";
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
      document.getElementById("brand-tagline").textContent =
        `${name} · AirPlay 2 · Miracast · Google Cast · DLNA — on your network only`;
    }
  } catch (error) {
    /* Without a configuration the generic tagline stays. */
  }
}

/**
 * Starts the panel: cards, first reading, poll loop and event channel.
 */
function start() {
  buildCards();
  renderConnection();
  renderCodecs();
  loadBuildInfo();
  loadDeviceName();

  document.getElementById("theme-toggle").addEventListener("click", toggleTheme);
  document.getElementById("metric-fps").addEventListener("click", () => selectMetric("fps"));
  document.getElementById("metric-bytes").addEventListener("click", () => selectMetric("bytes"));
  document.getElementById("chart-scope").addEventListener("change", (event) => {
    chartScope = event.target.value;
    renderChart();
  });

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
