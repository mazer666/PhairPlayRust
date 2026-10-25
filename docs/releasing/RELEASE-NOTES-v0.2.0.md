# PhairPlayRust 0.2.0 — the release notes

> **What this document is:** the description of the release **v0.2.0** — what
> it contains, what was measured before it was published, and what it does
> **not** claim.
>
> **Date:** 2026-10-24 — the date of the tag `v0.2.0`.
> **Revision:** the commit `2d0c4b4` of the branch `feat/other-devices` — the
> commit that the tag `v0.2.0` points at.
>
> **Status:** a **pre-release**. The code of five milestones is delivered and
> the gate of the Rust workspace is green, but:
>
> - **the device tests have not been run** — no sender, no television and no
>   device was available;
> - the acceptance records of the milestones **M1, M2 and M4 are missing**;
> - the acceptance record of **M3 says NOT ACCEPTED at the revision of this
>   release**: the Kotlin of the Android shell and the kit of the macOS
>   shell do not compile here (§4 and §6.4);
> - the acceptance record of **M5 is ACCEPTED with a documented limitation**
>   — the discovery works, the picture does not flow (`EXC-011`).
>
> The record of M0 is the oldest one that exists, and it names its own
> reservations.
>
> **The rule of these notes:** every statement names its source. A number
> without a source is not in these notes, and a thing that was not measured
> is written as **NOT RUN** with the reason.

---

## 1. The short version

PhairPlayRust is a receiver for AirPlay 2, Miracast, Google Cast, DLNA/UPnP
and Samsung Smart View. It runs on the local network, without an account,
without telemetry and without a cloud.

**v0.2.0 is the first release in which the protocol engines are
implemented.** The baseline v0.1.0 was the foundation: the workspace, the
shared core, the configuration and the documentation — with the protocol
engines, the format crates and the bridge as stubs. The source is the
section of v0.1.0 in [`../../CHANGELOG.md`](../../CHANGELOG.md).

This release carries:

- the **control plane of five protocols** — the part of each protocol that
  a sender talks to, implemented and covered by tests;
- the **four format crates** — the property list, the H.264 bitstream, the
  AAC framing and a native Rust port of the Apple Lossless decoder;
- the **platform bridge** and the **local dashboard**;
- the **headless receiver** — the program starts the protocols itself, reads
  its configuration from a file and knows the profiles of the classes of
  devices;
- the **interface as a remote control** — the dashboard and the native
  shells are built for an arrow pad; the dashboard is green here, the shells
  do not build (§6.4);
- **Samsung Smart View** — the discovery and the handshake of the
  MultiScreen Framework.

**What it does not carry:** a picture from a Galaxy device. The message
format of the Smart View channel is proprietary and is not implemented
(`EXC-011`). It is written down as a limit, not as a defect.

**And it does not carry a buildable native shell.** At the revision of this
release the Kotlin of the Android shell and the kit of the macOS shell do
not compile — four errors, one cause: the bridge gained a protocol kind and
a limits field in the milestone M5 and the shells were not updated with it.
The errors, their cause and their repair are in the acceptance record of M3
(§6.4). **The code of the shells was green at the commit of M3 and is
byte-identical here** — the break came with M5, and it is repaired by the
milestone that brought it.

---

## 2. What is in this release

### 2.1 The five protocol engines

The five engines live under [`../../engines/`](../../engines) and follow the
build contract of [`../design/BUILD-CONTRACT.md`](../design/BUILD-CONTRACT.md):
one peer at a time, a deadline on every blocking read, a limit on every
input.

| Engine | What is implemented | The limit |
|---|---|---|
| AirPlay 2 (`engines/airplay`) | the RTSP reader and the answer builder, the SDP parser, the RTP reassembly, the pairing (HomeKit-style and the legacy SRP PIN), the FairPlay handshake and the session key, the DACP reverse remote, the DMAP metadata, the NTP timing, the photo transfer | FairPlay **DRM** content is not decryptable (`EXC-001`); the in-app audio of the Apple Music app is protected (`EXC-002`) |
| Miracast (`engines/miracast`) | the WFD control plane, the information element parser, the answer of the sink discovery | the group formation of Wi-Fi Direct needs a system API of the host, and the WFD permission of the platform (`EXC-003`) |
| Google Cast (`engines/cast`) | the TLS listener, the CastV2 framing, the heartbeat, the channel management, the media receiver status | the end-to-end test needs an application identifier that is registered with Google; the project ships none and fakes none (`EXC-010`) |
| DLNA / UPnP (`engines/dlna`) | the SSDP answer and the notification, the device and service description, the SOAP dispatcher, the GENA events | the renderer serves audio and pictures, not video |
| Samsung Smart View (`engines/smartview`) | the SSDP answer of the MultiScreen search, the description document, the HTTP API below `/api/v2/`, the handshake of the sender, the WebSocket upgrade | **the message format of the channel is proprietary and is not implemented** (`EXC-011`) — a Galaxy device finds the receiver and connects to it, but no picture flows yet |

The discovery layer (`engines/discovery`) announces the services with DNS-SD
and answers with SSDP.

### 2.2 The four format crates

| Crate | What it is |
|---|---|
| `formats/plist` | the Apple property list in the XML **and** in the binary format, both directions |
| `formats/h264` | the SPS and the PPS parser, the NAL unit splitter, the Annex-B handling, the RTP FU-A reassembly |
| `formats/aac` | the ADTS header, the AAC-ELD framing, the counting of the samples |
| `formats/alac` | a native Rust port of the Apple Lossless decoder, with the attribution of the reference decoder in every derived file |

Every public function of the four crates has a unit test, and every hardening
measure has a negative test — the per-crate results are in
[`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) §1.

### 2.3 The platform bridge and the dashboard

The **bridge** (`platform/phairplay-ffi`) carries the values of the core
into the native shells through generated UniFFI bindings. A value that goes
into the bridge comes out of it unchanged — the round-trip tests are in the
record of M0. Every failure crosses the bridge as a typed error with a
stable numeric code.

The **dashboard** (`apps/web`) is a local panel with an HTTP API and a
WebSocket channel. It binds the loopback interface only, and a bind to
another address is announced with a warning before anything is started.

### 2.4 The receiver runs headless

The receiver no longer waits for a click in a browser. The start of the
process is the start of the protocols, and the report of the start names
every protocol with its state and with the reason of a failure.

```bash
# the receiver with the profile of a Raspberry Pi
phairplay-web --profile raspberry-pi

# the receiver with a configuration file of your own — the path of the installer
phairplay-web --config /etc/phairplayrust/config.toml

# the receiver without the panel — the API stays available
phairplay-web --profile generic --no-dashboard --port 7879
```

The device profiles — `raspberry-pi`, `nas`, `docker`, `desktop`, `generic` —
live in [`../../config/profiles/`](../../config/profiles/README.md), and the
guide for the devices that carry the receiver is in
[`../guides/OTHER-DEVICES.en.md`](../guides/OTHER-DEVICES.en.md).

With the receiver come the **systemd unit** and the **container**
([`../../deploy/`](../../deploy/README.md)). The container shares the network
of the host, because a bridge network routes neither the mDNS nor the SSDP
multicast.

### 2.5 The interface as a remote control

The dashboard and the native shells are built for a device with an arrow
pad, an *OK* button and no tab key.

- The group of the receiver cards is **one tab stop** with a roving
  tabindex; the arrow keys move the focus inside the group, and at the edge
  of the group they leave it for the next control of the page.
- The **focus frame is drawn by the project** and measured against the card
  and against the page.
- The **card is the switch** — the card is a button, and `Enter` or `Space`
  switches that receiver through the real API.
- The Android shell is a **real television application** with the
  `LEANBACK_LAUNCHER` category; the macOS shell has the same information
  architecture, natively.

The requirements are `FR-096` to `FR-099` in
[`../../REQUIREMENTS.md`](../../REQUIREMENTS.md).

**The state of the shells at the revision of this release:** the sources of
both shells are byte-identical to the commit of M3 and were green there; the
generated bindings of the bridge changed with M5, and the two shells were
not updated with them. **The shells do not build here** — the acceptance
record of M3 names the four errors, their cause and their repair (§6.4).
The dashboard of the browser is not affected: it is a set of static files,
and its checks are green (§3).

### 2.6 Samsung Smart View

The fifth protocol of the vocabulary: `ProtocolKind::SmartView`, the service
type `_samsungmsf._tcp`, the port of the API and the crate
`engines/smartview`. The engine answers the search of the MultiScreen
framework, serves the description document, answers the HTTP API, accepts the
handshake of the sender and answers the WebSocket upgrade of the channel.

The report of the engine, with every test named, is in
[`../audit/smartview-engine.md`](../audit/smartview-engine.md).

**The known limit, in the words of the project:** a Galaxy device finds the
receiver and connects to it, but no picture flows yet. The message format of
the channel is proprietary and is not publicly documented; it is not
implemented and it is not claimed (`EXC-011`).

### 2.7 The gates

The repository carries its own gate: the format check, the lint of the
workspace, the tests, the crate boundaries, the length of the source files,
the licence headers, the links of the documentation, the licence audit of the
dependencies with a pinned cargo-deny, and the audit of the advisories. The
gate is proved to be a gate — the self-test breaks every rule on purpose and
proves that the gate goes red. The evidence is in
[`../audit/2026-10-24-gate-self-test.md`](../audit/2026-10-24-gate-self-test.md).

---

## 3. The measured evidence

Every number of this table names the file that carries it. The state of the
project is machine-readable in
[`../../project/project.yaml`](../../project/project.yaml), and the raw
output of every measurement is in [`../audit/`](../audit/).

| What | The value | The source |
|---|---|---|
| Tests of the Rust workspace | 1224 passed, 0 failed | `project/project.yaml` → `verification.tests_passed`; the run is quoted in [`../audit/docs-truthfulness.md`](../audit/docs-truthfulness.md) §4.3 |
| Crates of the workspace | 13 members | [`../../Cargo.toml`](../../Cargo.toml) → `[workspace] members` |
| Protocol engines | 5, plus the discovery | `project/project.yaml` → `architecture.engines` |
| Format crates | 4 | `project/project.yaml` → `architecture.formats` |
| The checks of the dashboard | 35 keyboard checks, 30 layout checks, 38 measured colour pairs — 0 failures | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §10; the raw output in [`../audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) |
| The Android shell | at the commit of M3 the Kotlin compiles and its 39 tests are green; **at the revision of this release the module does not compile** — 2 errors | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1 and §1a |
| The macOS shell | at the commit of M3 the kit runs 101 tests with 0 failures; **at the revision of this release the kit does not compile** — 2 errors | the same record, §1 and §1a |

> **The two shells compile after this release.** The notes of a release
> describe the release and are not rewritten afterwards — the two rows above
> are the state of the revision `2d0c4b4`. The defect was fixed afterwards:
> the bridge of M5 requires the byte budget of the audio queue and neither
> shell handed it over. The measurement of the repair is in
> [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §10 —
> `BUILD SUCCESSFUL in 4m 48s` and 39 tests on Android, `Build complete!`
> and 102 tests on macOS — and both shells are built by the job *Shells
> (Android, macOS)* of the continuous integration on every push since.
| The Smart View engine | 112 tests of the crate, 0 failed | [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §10; the raw output in [`../audit/smartview-engine-test.log`](../audit/smartview-engine-test.log) |
| The milestone M5 | **ACCEPTED with a documented limitation** | [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §10 |
| The milestone M3 | **NOT ACCEPTED at the revision of this release** | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §10 |
| The links of the documentation | green — every internal link of the tree resolves | `bash scripts/check-docs-links.sh`; the output of the run of the day is in [`../audit/releasing.md`](../audit/releasing.md) |

**The continuous integration** runs the same gate on the GitHub runners: the
*Quality Gate*, the *Licenses*, the *Security Audit* and the *Structure*. The
workflow is [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml); the
acceptance record of M0 reports a green run of all four jobs on the commit of
that record. The state of the runners on the commit of this release is the
state of step 1 of the release process
([`RELEASING.md`](RELEASING.md) §3.2) — it is not verifiable from the
working tree, and these notes do not claim it.

---

## 4. What is **not** in this release

These rows are **NOT RUN** — and one of them is a build that **fails**.
Most of them are the measurements that only a real device, a real sender or a
real television can produce; the build of the native shells needs a build
machine, and it is red at the revision of this release. The plan that closes
the rows of the first kind is
[`../guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md).

| What | Why it is not measured | Where it is written down |
|---|---|---|
| The interoperability of the five protocols — an Apple sender, a Windows sender, a Cast sender, a DLNA control point, a Galaxy device | the sandbox has no sender and no television | [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) §7; the rows of the device test plan |
| The latency to the first picture, the frame rate, the memory and the CPU of the running receiver | a measurement on a build server measures the build server; the budgets are owned by [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml) and are **budgets**, not measurements | [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) §3; `project/project.yaml` → `verification.performance_budgets_measured` |
| The panel on a television with a real remote control, in a bright room | the contrast is measured with the formula of WCAG in a browser; the room is not | [`../audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §8 |
| The APK of the Android shell | the cross-build of the native bridge needs the NDK of the version that the Android Gradle plugin wants, and it is not installed on the machine of the audit | [`../audit/tv-android-shell.md`](../audit/tv-android-shell.md) §9; [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §9 |
| The SwiftUI part of the macOS shell | the machine of the audit has no macOS SDK | [`../audit/tv-macos-shell.md`](../audit/tv-macos-shell.md) §7 |
| The picture of a Galaxy device | the message format of the Smart View channel is proprietary — this is a limit, not a missing test | `EXC-011`; [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §8 |
| **The native shells build** | the Kotlin and the Swift do not compile at the revision of this release — four errors, one cause (§6.4) | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1a, §8 (limitation 1) and §10 |
| The service type `_samsungmsf._tcp` is published | the vocabulary of the core claims the type, no component of the receiver publishes an mDNS record for it; the discovery of the fifth protocol is SSDP-only | [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §8 (L-2) |
| The Smart View scenario of the device test plan | the plan has no section for it — the sections 2.1 to 2.4 cover AirPlay 2, Miracast, Cast and DLNA | [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §8 (L-3) |
| The acceptance records of the milestones M1, M2 and M4 | they are written after the device tests | the paths `../acceptance/M1-ACCEPTANCE.md`, `../acceptance/M2-ACCEPTANCE.md`, `../acceptance/M4-ACCEPTANCE.md`, relative to this file — the record of each milestone. They do not exist yet. |

**One open point of the release (severity S1):** in the default configuration
the fifth protocol does not start — the DLNA renderer and the Smart View
responder want the same SSDP port, and the renderer holds it first. The
acceptance record of M5 names the mechanism: the DLNA responder binds its
socket without `SO_REUSEADDR`, the Smart View responder with it, and on
Linux two UDP sockets may share an address and a port only when **both** set
the option.

> **This open point was closed after this release.** The notes of a release
> describe the release and are not rewritten afterwards — the run below is
> the run of the revision `2d0c4b4`. The conflict was resolved by giving
> the process **one** SSDP listener that every service registers itself with
> ([`../adr/ADR-0007-one-ssdp-listener-for-the-whole-receiver.md`](../adr/ADR-0007-one-ssdp-listener-for-the-whole-receiver.md));
> the current run of the receiver reports **5 of 5 protocols are running**.
> See the acceptance record of M5, §8 (L-1).

The run of the receiver at the revision of this release shows it:

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
```

With `[protocols.dlna] enabled = false` the fifth protocol reaches the
advertising state. **The conflict belongs to the engines, not to the
documentation** — it is an open point for the owner, and it is carried by
the milestone M5 in [`../../ROADMAP.md`](../../ROADMAP.md). The source of the
run is [`../audit/docs-truthfulness.md`](../audit/docs-truthfulness.md) §4.1;
the limitation is **L-1** of
[`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §8, and the
decision belongs to the owner: either the DLNA responder sets the option as
well, or one shared responder in `engines/discovery` serves the group for
every protocol of the host.

---

## 5. The limits of the product

The limits are not a postscript of these notes; they are part of the
product and they are listed in one place:
[`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) carries the exclusion list
`EXC-001` to `EXC-011`. The three that a user meets first:

| The limit | The identifier |
|---|---|
| FairPlay DRM content cannot be played — no open-source receiver can decrypt it | `EXC-001` |
| The in-app audio of the Apple Music app is protected on every AirPlay path; the system audio output of the sender is the workaround | `EXC-002` |
| The message format of the Samsung Smart View channel is proprietary and is not implemented | `EXC-011` |

The privacy promise and the threat model live in
[`../../PRIVACY.md`](../../PRIVACY.md) and in
[`../../SECURITY.md`](../../SECURITY.md). The receiver talks to nothing but
the local network, keeps no account and sends nothing to anybody.

---

## 6. The defects that this release repairs

### 6.1 The adversarial review of the first implementation

The review found real defects in the ALAC decoder and in the AirPlay engine.
Each of them has a test that fails without the fix:

- a lost packet in the TAP-A path of the decoder;
- a panic in the Golomb coder on 33-bit samples;
- an overflow in the shift of an element, and a stale shift buffer in the
  escape path — a *remote* panic, and a wrong picture without an error;
- an index out of bounds in the stereo path;
- a lost frame in the AirPlay engine, and a collision of two tests on one
  port.

The source is the *Fixed* section of
[`../../CHANGELOG.md`](../../CHANGELOG.md) and the baseline of the test suite
in [`../audit/2026-10-24-baseline.md`](../audit/2026-10-24-baseline.md).

### 6.2 The composition root

Since v0.1.0 the bridge owned a supervisor with an **empty** engine table:
nothing ever registered an engine. The consequence was not a compile error
but a product that starts, lists the protocols in its dashboard and answers
every attempt to start one with *"no engine implementation is registered for
the protocol"* — the native shells could display the state of a receiver that
could not receive.

The composition root now lives in the bridge, and the web backend reaches the
engines through it. The decision and its context are in
[`../adr/ADR-0006-the-composition-root-lives-in-the-bridge.md`](../adr/ADR-0006-the-composition-root-lives-in-the-bridge.md).

### 6.3 The licence gate on a clean checkout

The licence gate failed on a clean checkout; the fix is in the commit
`2d0c4b4` that this release names. The gate of a release is only a gate if
it passes on a clean checkout of the tag.

### 6.4 The defect that this release brings with it — the native shells do not build

This is not a repair but a **break**, and it is the most important line of
these notes.

At the revision of this release the Kotlin of the Android shell and the kit
of the macOS shell **do not compile**:

| The shell | The error |
|---|---|
| Android | `ReceiverMapping.kt:91` — the `when` over the protocol kind is not exhaustive, the fifth kind is missing; `ReceiverDefaults.kt:104` — the new limits field has no value |
| macOS | `ReceiverMapping.swift:83` — the `switch` over the protocol kind is not exhaustive, the case `.smartView` is missing; `ReceiverDefaults.swift:124` — the new limits field has no value |

**The cause is one thing:** the commit `86c1b81` (the milestone M5) added a
protocol kind and a limits field to the bridge and regenerated the bindings.
The four files of the shells are **byte-identical** to the commit of M3 —
the shells were not updated with the new contract.

**What it means for a user of this release:** the Rust receiver, the
dashboard, the engines and the formats are what this release delivers; the
native applications are not buildable from this revision. The repair is two
branches per shell and belongs to the milestone that broke the build. The
complete record — with the reproduction, the localisation and the conditions
for the acceptance — is in
[`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1a and
§10.

**The second finding of that record belongs here too:** the native shells
are outside every continuous gate of the repository. The CI runs the Rust
gate and the four structural scripts; **no Gradle task and no Swift test is
part of it**. That is why the break could reach the branch, and it is the
second condition that the record of M3 names for its acceptance.

---

## 7. The upgrade from 0.1.0

| | v0.1.0 | v0.2.0 |
|---|---|---|
| The protocols | stubs — no protocol was implemented | the control plane of five protocols, with tests |
| The receiver | no program — the baseline had no application to start | the program starts the protocols itself |
| The configuration | the typed model and the files of `config/` | the same, plus the file of the user and the profiles of the classes of devices |
| The shells | written, not built here | written, with the remote control interface and the Leanback entry — but **they do not compile at this revision** (§6.4) |

There is **no migration of a data format**: the project keeps no data, no
account and no state that outlives the process. A user of v0.1.0 has nothing
to convert — v0.1.0 could not receive anything.

A user who wants to run the receiver of this release follows the quickstart of
[`../../README.md`](../../README.md) or the guide for the devices that carry
the receiver, [`../guides/OTHER-DEVICES.en.md`](../guides/OTHER-DEVICES.en.md).

---

## 8. The milestones and their evidence

| Milestone | The code | The evidence | The record |
|---|---|---|---|
| M0 — Foundation | delivered | the gate is green | [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) — **accepted with reservations** |
| M1 — AirPlay 2 receiver | delivered | the tests of the engine and of the four format crates | the record of this milestone — **missing** |
| M2 — Protocol breadth and local control | delivered | the tests of the three engines, of the bridge and of the dashboard | the record of this milestone — **missing** |
| M3 — Native shells | delivered at the commit of M3 | the audits of the three shells | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) — **NOT ACCEPTED at the revision of this release** (§6.4) |
| M4 — Hardening, interoperability and release readiness | largely delivered | the structural gates, the licence gate, the continuous integration | the record of this milestone — **missing** |
| M5 — Smart View | delivered | the tests of the engine | [`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) — **accepted with a documented limitation** (`EXC-011`, L-1) |

"Delivered" is not "accepted". A milestone becomes `done` when its record is
filed and green. The state of the milestones is in
[`../../ROADMAP.md`](../../ROADMAP.md) and the machine-readable state is in
[`../../project/project.yaml`](../../project/project.yaml).

---

## 9. How to check this release yourself

```bash
# the shared library of the bridge — the test of the foreign ABI loads it,
# and `cargo test` builds the test harness, not the shared library
cargo build -p phairplay-ffi

# the workspace compiles and its tests are green
cargo build --workspace
cargo test --workspace

# the lint and the format of the gate
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# the structural gate and the links of the documentation
bash scripts/check-crate-boundaries.sh
bash scripts/check-file-size.sh
bash scripts/check-license-headers.sh
bash scripts/check-docs-links.sh
bash scripts/audit-licenses.sh

# the gate as a whole, and the proof that the gate refuses its violations
bash scripts/local-ci.sh
bash scripts/local-ci.sh --self-test
```

The version of the tree is in [`../../VERSION`](../../VERSION) and in
[`../../Cargo.toml`](../../Cargo.toml); the notes of the release process are
in [`RELEASING.md`](RELEASING.md).

**These commands check the Rust tree.** The native shells do not compile at
this revision (§6.4): `cd apps/android && ./gradlew :app:compileDebugKotlin`
and `cd apps/macos && swift test` both fail here, and the reasons and the
repair are in [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md)
§1a.

---

## 10. The provenance of these notes

| The part of these notes | The source it comes from |
|---|---|
| The list of the changes | the *Unreleased* section of [`../../CHANGELOG.md`](../../CHANGELOG.md), which becomes the section of this release |
| The acceptance of a milestone | [`../acceptance/`](../acceptance/ACCEPTANCE_TEMPLATE.md) — one record per milestone: M0, M3 and M5 |
| The state of the milestones | [`../../ROADMAP.md`](../../ROADMAP.md) |
| The machine-readable state | [`../../project/project.yaml`](../../project/project.yaml) |
| The measured numbers | [`../audit/`](../audit/), one file per measurement |
| The limits of the product | the exclusion list in [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) |
| The privacy promise and the threat model | [`../../PRIVACY.md`](../../PRIVACY.md), [`../../SECURITY.md`](../../SECURITY.md) |
| The criteria of a release | [`../../PROJECT_MANAGEMENT.md`](../../PROJECT_MANAGEMENT.md) §12 |
| The rules of the evidence | [`../../QUALITY.md`](../../QUALITY.md), [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §5 |
| The process that produced this release | [`RELEASING.md`](RELEASING.md) |
