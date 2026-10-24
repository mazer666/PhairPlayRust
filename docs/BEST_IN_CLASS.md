# Best-in-Class Evaluation Framework

The "best in class" goal of PhairPlayRust is an **engineering measurement
standard**, not a marketing slogan and not a feature-count checklist.

A claim is only allowed when it names a measurement, a device, an input and a
command. The numeric budgets live in
[`../config/performance-budgets.toml`](../config/performance-budgets.toml);
this document names the keys and never repeats the values.

---

## 1. The reference products

Depending on the workflow and the platform, the receiver is compared with:

- **AirPlay receivers:** the receiver of Apple (Apple TV), and the open
  reference implementations that are known to the project
- **Miracast sinks:** the sinks that ship with the operating systems of the
  desktop and the television
- **Cast receivers:** the official Cast receiver and the open custom receivers
- **DLNA renderers:** the media renderers of the television platforms and of
  the desktop players such as VLC

The comparison is about the **experience of the user**, not about the number
of features. A reference product is named with its version and the date of
the comparison.

---

## 2. The evaluation grid

Every milestone and every major workflow is evaluated across the seven
dimensions below. Each dimension names the metric, the way it is measured and
the document that owns the budget.

### 2.1 Latency

| Metric | How it is measured | Budget owner |
|---|---|---|
| Time from the acceptance of the peer to the first picture | A timestamp in the engine, a timestamp at the output surface, the difference | `latency.first_frame_ms` |
| Time from the start of the application to the first advertisement | The timestamp of the registration of the service | `latency.*`, the advertisement path |
| Deviation between the audio and the video | The presentation timestamps of the received packets over a recording of at least one minute | `latency.audio_video_sync_ms` |
| Time between the loss of the network and the recovery of the advertisement | The timestamp of the interface change and the timestamp of the re-registration | `discovery.re_advertise_on_network_change` |
| Time from the start of the process to the advertisement of the receiver | The timestamp of the start of the process and the timestamp of the registration of the service — this is the path of the **headless** build, where nobody clicks anything | `latency.*`, the advertisement path |

**The rule:** the latency is measured on the path to the *screen*, not on the
path to the buffer. A receiver that receives fast and displays late has a bad
latency.

### 2.2 Frame rate and continuity

| Metric | How it is measured | Budget owner |
|---|---|---|
| Sustained frame rate of a mirroring session | The delivered frames over the measurement window, divided by the window | `throughput.minimum_fps` |
| The share of the dropped frames | The counters of the core: `video_frames` and `video_frames_dropped` | `queues.*` |
| The stability of the frame rate over a long session | The same measurement over a session of at least the measurement window | `throughput.measurement_window_seconds` |
| The packets that the sender reports as lost | The RTCP report of the sender, counted by the receiver | `metrics::StreamCounters` |

**The rule:** a frame rate that is high on average and collapses every ten
seconds is not a frame rate. The measurement covers the whole window.

### 2.3 Stability

| Metric | How it is measured | Budget owner |
|---|---|---|
| The behaviour against a hostile peer | The negative test suite: a flood, a length that lies, a cycle, an entity | `limits.*` |
| The recovery after a peer disappears | The idle timeout and the state of the engine | `latency.idle_timeout_ms` |
| The behaviour when the network changes | The re-advertisement after an interface change | `discovery.*` |
| The behaviour when a decoder stalls | The bounded queue and the drop counter | `queues.*` |
| The behaviour when the platform refuses a capability | The missing-capability report of the engine | `capabilities.*` |

**The rule:** a receiver that survives an attack is not the same as a receiver
that survives a bug. Both are measured.

### 2.4 Interoperability

| Metric | How it is measured | Budget owner |
|---|---|---|
| The senders of the matrix | The device test plan, one row per sender | `docs/guides/` |
| The formats that a reference implementation reads | The corpus tests of the format crates | `formats/*` |
| The receiver without a component of the vendor of the sender | The fallback behaviour of the Cast engine | the requirements |
| The deviations from the expectation of the sender | The acceptance record of the milestone | `docs/acceptance/` |
| The fifth protocol — Samsung Smart View | The search of the MultiScreen Framework, the description document, the handshake of the sender | the requirements, `FR-104` … `FR-107` |

**The rule:** a sender that could not be tested is named with the reason. A
silent gap in the matrix is a false claim of interoperability.

The fifth protocol carries a limit that is **not** a missing test: the
message format of the Smart View channel is proprietary and not implemented
(`EXC-011`). The discovery and the handshake are measured; the picture of a
Galaxy device is not claimed at all.

### 2.5 User experience

| Metric | How it is measured | Budget owner |
|---|---|---|
| The steps for the common task | The walkthrough of the task, counted | the acceptance record |
| The time until the device appears in the picker of the sender | The registration of the service | the advertisement path |
| The clarity of the error message | The review of the message against the action the user can take | the error hierarchy |
| The distinction of the states | "disabled", "advertising", "streaming", "error" — the review of the interface | the shells, the dashboard |
| The accessibility of the dashboard | The keyboard, the contrast, the reduced motion | the acceptance record |
| **The operation with a remote control** | The keyboard check of the dashboard: the arrow keys, the tab order of the group, the switch of the card | the audits in `docs/audit/` |
| **The visibility of the focus** | The measured contrast of the focus frame against the card, against the page and against the halo | the audits in `docs/audit/` |
| **The operation without a screen** | The report of the start of the receiver: every protocol is named with its state | the acceptance record of the headless build |

**The rule:** the receiver must be understandable without a manual. A user
who cannot find the switch in five seconds has a usability defect, not a
documentation defect.

**The rule of the remote control:** a control that a remote control cannot
reach does not exist. The target of a switch is the card itself, the focus
frame is drawn by the project and measured, and the state is carried by a
word and a shape and not by a colour alone.

### 2.6 Resources

| Metric | How it is measured | Budget owner |
|---|---|---|
| The resident memory of the process | The operating system, during a session of at least the measurement window | `resources.max_memory_mb` |
| The CPU usage of the process | The CPU time over the wall-clock time of the window | `resources.max_cpu_percent` |
| The growth of the memory over a long session | Two measurements, at the beginning and at the end | `resources.max_memory_mb` |
| The behaviour of the battery and the thermal state on Android | The platform measurement during a session | the acceptance record |

**The rule:** the memory must not grow with the length of the session. A
receiver that leaks slowly is a receiver that fails on a long evening.

### 2.7 Documentation and developer experience

| Metric | How it is measured | Budget owner |
|---|---|---|
| Every internal link resolves | `bash scripts/check-docs-links.sh` | the documentation policy |
| Every number exists in exactly one place | The comparison of the documented budgets with the typed model | the documentation policy |
| A newcomer finds the right entry point | `WHERE_TO_CHANGE.md` against the change that is being made | the documentation policy |
| The build works in a clean checkout | `make check` | the Makefile |
| The documentation matches the code | The review of the change against the requirements | the review matrix |

---

## 3. The grade of a milestone

| Grade | Meaning |
|---|---|
| **A** | Every budget of the dimension is met on the reference device, and the evidence is filed |
| **B** | Every budget is met, but one measurement was taken on a different device than the reference |
| **C** | One budget is missed, the deviation is explained and the decision is recorded |
| **D** | More than one budget is missed, or an interoperability row is open |

A milestone of grade **C** or **D** does not ship without a documented
decision of the project owner.

---

## 4. The rules of the comparison

1. **A comparison names the version and the date.** A statement about a
   product from an unknown version is worthless.
2. **A comparison is measured, not estimated.** "Feels snappier" is not a
   result.
3. **The same input, the same device, the same network.** A comparison that
   changes the conditions measures the conditions.
4. **The disadvantage is written down.** A review that only lists advantages
   is marketing.
5. **A follow-up is a task with an owner**, not a wish.

---

## 5. What is explicitly not part of the comparison

- the number of supported codecs as such — a codec that the device cannot
  decode is a liability, not a feature
- the number of menus and options
- the visual similarity to a competitor
- the presence of a feature that is out of scope, such as a DRM that nobody
  may implement (`EXC-001`)

---

## 6. What the project has measured — and what it has not

The grid above is the target. This section is the honest state of the
measurements: what a number exists for today, and where it comes from.

| What | Measured | Where the evidence is |
|---|---|---|
| The dashboard with the keyboard only | the checks of the keyboard, of the layout and of the contrast — every one of them in a real browser | [`docs/audit/tv-web-dashboard.md`](audit/tv-web-dashboard.md) |
| The order of the focus of the Android shell | the unit tests of the focus graph | [`docs/audit/tv-android-shell.md`](audit/tv-android-shell.md) |
| The focus frame and the keyboard of the macOS shell | the test suite of the kit | [`docs/audit/tv-macos-shell.md`](audit/tv-macos-shell.md) |
| The discovery and the handshake of Samsung Smart View | the tests of the engine, over a real socket | [`docs/audit/smartview-engine-test.log`](audit/smartview-engine-test.log) |
| The whole workspace | the build, the lint and the test suite | the acceptance record of M0 and [`../project/project.yaml`](../project/project.yaml) |

**What is NOT RUN** — and stays open until somebody measures it on a
device:

| What | Why it is not measured |
|---|---|
| The time to the first picture, the frame rate, the memory and the CPU of the running receiver | a measurement on a build server measures the build server; the budgets live in [`../config/performance-budgets.toml`](../config/performance-budgets.toml) and are proved with a real sender |
| The interoperability of every protocol | the device test plan needs a sender, a television and a device |
| The panel on a television in a bright room, and a television with a real remote control | the contrast is measured with the formula of WCAG; the room is not |
| The picture of a Galaxy device | the message format of the Smart View channel is proprietary and not implemented (`EXC-011`) |

The rule of this section: a number that is not measured is not written down.
The row above names the *reason* of the gap, not a guess.
