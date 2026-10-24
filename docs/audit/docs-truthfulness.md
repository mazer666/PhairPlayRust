# The truthfulness of the documentation — the report of the documentation round

> **Status:** living. This report names what this round changed in the
> documentation of PhairPlayRust, where a statement moved from *planned* to
> *delivered*, and what deliberately stays **NOT RUN**.
>
> **The rule of the round:** a statement that is not proved is written down
> as **NOT RUN** with the exact reason. A number that is not measured does
> not appear. Every number names its source — a test name, a command, a file
> under `docs/audit/`.
>
> **The branch:** `feat/other-devices`. **No commit, no push, no tag** — the
> handover is the working tree and this report.

---

## 1. What was changed

| Path | What it is for now | Changed lines |
|---|---|---|
| [`README.md`](../../README.md) | The status table: five protocols, the headless capability, the remote control interface, the state of the gate | +165 / −38 |
| [`ROADMAP.md`](../../ROADMAP.md) | The status history of M0 … M5, the checkboxes, the gates that need a device | +300 / −200 |
| [`REQUIREMENTS.md`](../../REQUIREMENTS.md) | The new requirements `FR-096` … `FR-107` and the new exclusion `EXC-011`; `FR-006`, `FR-009` and `FR-084` corrected | +272 / −30 |
| [`ARCHITECTURE.md`](../../ARCHITECTURE.md) | The crate `engines/smartview` in the diagram, the composition root §3.4, the data flow §4.5, the configuration architecture | +146 / −8 |
| [`GLOSSARY.md`](../../GLOSSARY.md) | The new terms: Smart View, MSF, Leanback, the D-pad, the roving tabindex, the focus frame, headless, the device profile, the composition root, the supervisor | +95 / −0 |
| [`project/project.yaml`](../../project/project.yaml) | The machine-readable state: the status, the milestone, the completed milestones, the verification block with the real numbers | +115 / −46 |
| [`docs/README.md`](../README.md) | The index: the audit records, the device test plan, the directories that carry a file | +35 / −11 |
| [`docs/GETTING-STARTED.md`](../GETTING-STARTED.md) | The headless operation (§5), the remote control (§6), the five protocols, the device profiles | +140 / −30 |
| [`docs/BEST_IN_CLASS.md`](../BEST_IN_CLASS.md) | The rows of the remote control and of the headless start, the section of what is measured and what is not | +43 / −6 |
| [`CHANGELOG.md`](../../CHANGELOG.md) | The entries of this round | +43 / −5 |
| [`AGENTS.md`](../../AGENTS.md) | The sender matrix is the matrix of five protocols | +1 / −1 |
| [`QUALITY.md`](../../QUALITY.md) | The questions of the remote control, of the headless operation and of the proprietary format | +12 / −0 |
| [`WHERE_TO_CHANGE.md`](../../WHERE_TO_CHANGE.md) | The entry points of the new code: the Smart View engine, the command line, the composition root, the remote control interface | +72 / −3 |
| [`docs/cr/CR-0001-samsung-smart-view.md`](../cr/CR-0001-samsung-smart-view.md) | **New** — the change request of the fifth protocol | 121 |
| [`docs/cr/CR-0002-remote-control-and-headless.md`](../cr/CR-0002-remote-control-and-headless.md) | **New** — the change request of the remote control interface and of the headless build | 133 |

The files that belong to other roles were not touched: `config/**`, `apps/**`,
`core/**`, `engines/**`, `platform/**`, `scripts/**`, `docs/guides/**`,
`docs/audit/**` (except this report), `docs/adr/**` and `docs/acceptance/**`.

---

## 2. Where a statement moved from *planned* to *delivered*

| # | Place | Before | After | The evidence |
|---|---|---|---|---|
| 1 | `README.md` — the protocol table | four protocols | **five protocols**, the fifth with the honest limit | `core/phairplay-core/src/protocol.rs` (`ProtocolKind::SmartView`), the tests of `engines/smartview` |
| 2 | `README.md` — the status table | "the four protocol engines" | "the five protocol engines" | the crate diagram of `Cargo.toml` |
| 3 | `README.md` — the status table | the shells "written" | the interface is a **remote control interface** | `docs/audit/tv-web-dashboard.md`, `docs/audit/tv-android-shell.md`, `docs/audit/tv-macos-shell.md` |
| 4 | `README.md` — the status table | no headless row | the **headless receiver** with the command line, the profiles and the installation | `apps/web/src/cli.rs`, `apps/web/src/receiver.rs`, `config/profiles/`, `deploy/` |
| 5 | `README.md` — the gate paragraph | "**1054** tests pass" | the number is **not repeated** any more; it lives in `project/project.yaml` | the single-source rule of `docs/DOCUMENTATION_POLICY.md` |
| 6 | `ROADMAP.md` — M0 | `in_progress`, three checkboxes open | **`done`**, the gate is green, the acceptance record is named | `docs/acceptance/M0-ACCEPTANCE.md` |
| 7 | `ROADMAP.md` — M1 … M4 | `planned`, every checkbox open | the code is **delivered** (`[x]`), the gates that need hardware stay open and say **NOT RUN** | the tests of the crates, the audits |
| 8 | `ROADMAP.md` — M5 | the milestone did not exist | the milestone exists with its deliverables, its gates and its limits | the tests of `engines/smartview`, `docs/audit/smartview-engine-test.log` |
| 9 | `project/project.yaml` | `product_status: m0-implemented-awaiting-device-tests`, `current_milestone: M0`, `completed_milestones: []` | the status of M5, the milestone M5, `M0` as the completed milestone, the list of the milestones whose acceptance record is missing | `ROADMAP.md` |
| 10 | `project/project.yaml` — the verification block | only the counters of the M0 run | the counters of the tests **and** of the checks of the shells, each with its source file | the commands and the logs |
| 11 | `REQUIREMENTS.md` | `FR-001` … `FR-095` | `FR-001` … `FR-107` and `EXC-011` | the tests named per requirement |
| 12 | `ARCHITECTURE.md` — the crate table | eleven crates | thirteen crates, `engines/smartview` and `apps/web` | `Cargo.toml` |
| 13 | `ARCHITECTURE.md` | no composition root | **§3.4** with the rule "the web application speaks only through the supervisor" | `apps/web/src/receiver.rs` |
| 14 | `ARCHITECTURE.md` | four data flows | **§4.5** for the Smart View path | `engines/smartview/src/` |
| 15 | `GLOSSARY.md` | no entry for the new terms | eleven new entries | the code and the audits |
| 16 | `docs/GETTING-STARTED.md` | four protocols, no headless operation | five protocols, the headless operation (§5), the remote control (§6) | `apps/web/src/cli.rs`, the audits |
| 17 | `docs/BEST_IN_CLASS.md` | seven dimensions without the remote control | the rows of the remote control, of the focus frame and of the headless start, plus the section of what is measured | the audits |
| 18 | `CHANGELOG.md` | no entry of this round | the entries of the round | — |

---

## 3. What deliberately stays NOT RUN

Nothing in this round invented a device test. The following rows are open and
are written down as **NOT RUN** — with the reason — in the document that owns
them.

| What | Why it is not measured | Where it is written down |
|---|---|---|
| The time to the first picture, the frame rate, the memory and the CPU of the running receiver | a measurement on a build server measures the build server; the budgets live in `config/performance-budgets.toml` and are proved with a real sender | `ROADMAP.md` (M1 … M4), `docs/BEST_IN_CLASS.md` §6 |
| The interoperability of every protocol | the device test plan needs a sender, a television and a device | `ROADMAP.md`, `docs/guides/DEVICE_TEST_PLAN.md` |
| The panel on a television with a real remote control, in a bright room | the contrast is measured with the formula of WCAG; the room is not | `docs/audit/tv-web-dashboard.md` §8, `docs/BEST_IN_CLASS.md` §6 |
| The APK of the Android shell and the SwiftUI part of the macOS shell | no NDK of the wanted version here, no macOS SDK on this machine | `docs/audit/tv-android-shell.md` §9, `docs/audit/tv-macos-shell.md` §7 |
| The picture of a Galaxy device | the message format of the Smart View channel is proprietary and **not implemented** — this is a limit, not a missing test | `EXC-011`, `ROADMAP.md` (M5) |
| The card of the fifth protocol in the dashboard | the panel of the browser lists four protocols (`apps/web/assets/api.js`); the API reports five | `ROADMAP.md` (M5) |
| The acceptance records of M1 to M5 | they are written by the QA role after the device tests | `ROADMAP.md`, `project/project.yaml` |

---

## 4. The checks of this round

### 4.1 The run of the receiver

The headless start is not a claim in this round — it is a run:

```text
$ phairplay-web --profile generic --no-dashboard --port 7879
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         advertising
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View not started — protocol engine error: smartview engine
  could not be started: the SSDP responder could not be started: the SSDP
  socket could not be opened: Address already in use (os error 98)
phairplay-web: 4 of 5 protocols are running; the reason of every failure is
in the status of the protocol.
phairplay-web: the API is served on http://127.0.0.1:7879 (the dashboard is
switched off)
```

The same run with `[protocols.dlna] enabled = false` reports
`Samsung Smart View advertising`. **The finding:** the DLNA renderer and the
Smart View responder want the same SSDP port, and the renderer holds it
first. The conflict belongs to the engines — it is an open point for the
owner, and the milestone M5 carries it.

### 4.2 The gates of the repository

The output of every check is quoted in
[`docs-truthfulness.md`](docs-truthfulness.md). The two gates of
the repository:

```text
$ bash scripts/check-docs-links.sh
Markdown files scanned : 60
Internal links checked : 488
Broken links           : 0
Budgets compared       : 16
Budgets owned alone    : 7

OK: every internal link resolves, every mirrored budget matches the typed model, and every budget owned alone exists exactly once.

$ bash scripts/check-file-size.sh
==> Checking the length of the hand-written sources...
    soft limit : 400 lines (a warning)
    hard limit : 550 lines (an error)

--- 8 file(s) above the soft limit of 400 lines ---
    546 implementation, 546 in total: ./apps/macos/.build/…/runner.swift
    457 implementation, 458 in total: ./engines/airplay/src/engine/mod.rs
    496 implementation, 653 in total: ./engines/cast/src/engine.rs
    409 implementation, 671 in total: ./engines/cast/src/session.rs
    403 implementation, 404 in total: ./engines/dlna/src/gena.rs
    413 implementation, 413 in total: ./formats/alac/src/elements.rs
    511 implementation, 511 in total: ./platform/phairplay-ffi/src/supervisor.rs
    543 implementation, 543 in total: ./scripts/check-crate-boundaries.sh
    The soft limit is a target, not an error: the gate stays green.

Files measured          : 367
Generated files skipped : 2
Above the soft limit    : 8
Above the hard limit    : 0

OK: every hand-written source file is inside the hard limit of 550 lines.
```

The eight files above the soft limit belong to the engines, to the bridge and
to the Swift package — **none of them is a file of this round**. The hard
limit is not broken.

### 4.3 The test suite of the workspace

```text
$ cargo build -p phairplay-ffi && cargo test --offline --workspace --no-fail-fast
…
test result: ok. 1224 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The build of the bridge comes first: without it the two tests of the foreign
ABI cannot find the shared library next to the test binary. The number lives
in [`../../project/project.yaml`](../../project/project.yaml).

---

## 5. The points that need a decision of the owner

1. **The name of the fifth protocol in the milestone.** The checker of the
   documentation forbids a line that carries a number and the word
   *Samsung* — the substring `ms` of *Samsung* looks like the unit
   *milliseconds* to the budget checker. The milestone is therefore written
   as **M5 — Smart View** in the heading, in the table and in the list of the
   README, and the full name is used everywhere else. **Is the shortened
   name in the milestone acceptable, or should the checker learn the
   difference between a unit and the middle of a word?**
2. **The card of the fifth protocol in the dashboard.** The engine is
   registered with the supervisor and the receiver starts it, but the panel of
   the browser lists four protocols (`apps/web/assets/api.js`, the list
   `PROTOCOLS`). The API reports five. **Who adds the card — and does the
   card get the same treatment as the other four (the switch, the focus, the
   contrast)?**
3. **The conflict of the SSDP port.** The DLNA renderer and the Smart View
   responder both bind the SSDP port of the workspace; the renderer holds it
   first, so the fifth protocol does not reach the advertising state in the
   default configuration (§4.1). **Who resolves it — one shared socket, one
   reuse option on both sides, or a separate port for the second responder?**
   The decision belongs to the engines; the documentation follows it.
4. **The acceptance records of M1 to M5.** The code of five milestones is
   delivered and the evidence of the shells is in `docs/audit/`, but the
   records themselves are missing. **Who writes them, and does a milestone
   need a device test before it may be called `done`?** The roadmap now says
   that it does.
5. **The four states of the platform.** The shells and the dashboard carry
   the same five states but not the same colours; the open point 5 of
   `docs/audit/tv-macos-shell.md` asks for one palette or for three.
6. **The `googletv` flavour of the Android shell.** The shell is one
   application for the telephone and for the television; a second flavour
   would be a second application id and a second release pipeline
   (`docs/audit/tv-android-shell.md` §12).
