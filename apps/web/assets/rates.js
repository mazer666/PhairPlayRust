/**
 * The rate of the running streams.
 *
 * WHAT: the time series of the graph. It carries the frames per second and
 * the bytes per second of every protocol.
 * WHY: the receiver reports *cumulative* counters — frames delivered, bytes
 * received — and not a rate. The rate is the difference between two
 * readings, and the panel is the only place that sees two readings. So the
 * panel computes it: the numbers of the graph are derived from the real
 * counters of the server, not invented.
 * HOW: every reading is compared with the one before it. The series lives in
 * the browser and is lost when the page is reloaded — a server-side series
 * would survive that, and is documented as an open point in the deliverable.
 */

/** How many readings the series keeps. */
const MAX_SAMPLES = 720;

/** How many readings the graph shows at most. */
const MAX_POINTS = 180;

export class RateTracker {
  constructor() {
    /** @type {Map<string, {t: number, videoFrames: number, bytesReceived: number}>} */
    this.previous = new Map();
    /** @type {Array<{t: number, fps: number, bytesPerSecond: number, protocol: string}>} */
    this.series = [];
  }

  /**
   * Folds a new reading of the counters into the series.
   *
   * WHY only from the second reading on: a rate is the difference between
   * two readings. The first one has no difference yet, and a graph that
   * starts at zero would claim a measurement that does not exist.
   *
   * @param {Record<string, {videoFrames: number, bytesReceived: number}>} counters
   * @param {number} [now] the time of the reading, in milliseconds
   * @returns {Array<{t: number, fps: number, bytesPerSecond: number, protocol: string}>}
   */
  update(counters, now = Date.now()) {
    const samples = [];
    for (const [id, stats] of Object.entries(counters ?? {})) {
      const previous = this.previous.get(id);
      this.previous.set(id, {
        t: now,
        videoFrames: stats.videoFrames,
        bytesReceived: stats.bytesReceived,
      });
      if (!previous || now <= previous.t) continue;
      const seconds = (now - previous.t) / 1000;
      samples.push({
        t: now,
        fps: Math.max(0, (stats.videoFrames - previous.videoFrames) / seconds),
        bytesPerSecond: Math.max(0, (stats.bytesReceived - previous.bytesReceived) / seconds),
        protocol: id,
      });
    }
    this.series.push(...samples);
    if (this.series.length > MAX_SAMPLES) {
      this.series = this.series.slice(this.series.length - MAX_SAMPLES);
    }
    return samples;
  }

  /**
   * The samples of one protocol, or the sum over all of them.
   *
   * WHY the sum: the scope "all receivers" asks for the traffic of the
   * whole receiver, and four lines in one chart would be a puzzle.
   *
   * @param {string} protocolId the protocol, or "all"
   * @returns {Array<{t: number, fps: number, bytesPerSecond: number}>}
   */
  samplesFor(protocolId) {
    if (protocolId !== "all") {
      return this.series
        .filter((sample) => sample.protocol === protocolId)
        .slice(-MAX_POINTS)
        .map((sample) => ({
          t: sample.t,
          fps: sample.fps,
          bytesPerSecond: sample.bytesPerSecond,
        }));
    }
    const byTime = new Map();
    for (const sample of this.series) {
      const entry = byTime.get(sample.t) ?? { t: sample.t, fps: 0, bytesPerSecond: 0 };
      entry.fps += sample.fps;
      entry.bytesPerSecond += sample.bytesPerSecond;
      byTime.set(sample.t, entry);
    }
    return [...byTime.values()].sort((left, right) => left.t - right.t).slice(-MAX_POINTS);
  }
}
