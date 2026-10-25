# PhairPlayRust Roadmap

**Roadmap version:** 0.2
**Current milestone:** M5 — Smart View (**IN PROGRESS** — the code and the
acceptance record stand; the device tests are missing)

The roadmap is milestone-driven. A milestone is not complete because "the
picture appears"; it is complete only when its functional, technical,
performance, usability, security/privacy, interoperability and documentation
gates are satisfied and the evidence is in
[`docs/acceptance/`](docs/acceptance/ACCEPTANCE_TEMPLATE.md).

The machine-readable state of the project lives in
[`project/project.yaml`](project/project.yaml). The numeric budgets of every
gate live in [`config/performance-budgets.toml`](config/performance-budgets.toml)
and are not repeated here.

---

## The state of the milestones

| Milestone | State | The code | The evidence | What is missing |
|---|---|---|---|---|
| M0 — Foundation | `done` | delivered | [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md) — the gate is green | the device tests of the interoperability |
| M1 — AirPlay 2 receiver | `in_progress` | delivered | the tests of the engine and of the four format crates | the acceptance record, the device tests |
| M2 — Protocol breadth and local control | `in_progress` | delivered | the tests of the three engines, of the bridge and of the dashboard | the acceptance record, the device tests |
| M3 — Native shells | `in_progress` | delivered | the audits of the three shells in [`docs/audit/`](docs/audit/tv-android-shell.md) | the acceptance record, the device tests |
| M4 — Hardening, interoperability and release readiness | `in_progress` | largely delivered | the structural gates, the licence audit, the continuous integration | the acceptance record of the workspace, the device tests, the release notes |
| M5 — Smart View | `in_progress` | delivered | the tests of the engine | the acceptance record, the device tests, the message format of the channel |

**"Delivered" is not "accepted."** A checkbox `[x]` in this document means
that the thing is written and covered by a test. A milestone becomes `done`
only when its acceptance record is filed in
[`docs/acceptance/`](docs/acceptance/ACCEPTANCE_TEMPLATE.md).

A gate that needs a television, a sender or a device **cannot** be closed by
a test on a build server. Every such gate is written down as **NOT RUN** with
the exact reason, and the plan that closes it is
[`docs/guides/DEVICE_TEST_PLAN.md`](docs/guides/DEVICE_TEST_PLAN.md).

---

## The milestone model

Every milestone has:

- a purpose
- a list of deliverables
- explicit non-goals
- functional acceptance criteria
- technical acceptance criteria
- performance gates
- usability criteria
- security and privacy criteria
- interoperability criteria
- documentation criteria
- QA evidence
- a release decision

The quality model behind these gates is described in
[`QUALITY.md`](QUALITY.md). The severity levels that decide whether a
milestone may ship are described there as well.

The milestone states are:

| State | Meaning |
|---|---|
| `planned` | The scope is agreed, the work has not started |
| `in_progress` | The work runs |
| `blocked` | The work cannot continue without a decision or an external input |
| `acceptance` | The work is done, the evidence is being reviewed |
| `done` | The gates are green and the evidence is filed |

**"Done" requires evidence, not confidence.**

---

# M0 — Foundation *(Status: `done`)*

> **The gate of M0 is green.** The acceptance record is in
> [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md) and
> names every command that was run. The record is explicit about what it does
> **not** prove: every interoperability row and every performance row needs a
> real sender on a real device and is written down there as **NOT RUN**.

## Purpose

Prove that the architecture works before a single protocol is implemented:
one workspace, one shared core, one build contract, one central
configuration, one documentation landscape, one gate.

## Deliver

- [x] Cargo workspace with thirteen members: the core, six engine crates, four
      format crates, the platform bridge and the web backend
- [x] The shared core: the protocol trait, the session registry, the media
      sinks, the capabilities, the configuration, the identity, the metrics,
      the error hierarchy, the job executor
- [x] The build contract `docs/design/BUILD-CONTRACT.md`
- [x] The central configuration: defaults, performance budgets, security
      policy
- [x] The licence, the reuse metadata and the licence inventory directory
- [x] The repository gate: a script that checks the documentation links, the
      crate boundaries, the format, the lint and the tests
- [x] The documentation set in the house format: the root documents, the
      `docs/` tree, the templates and the guides
- [x] The continuous integration with the jobs of the quality gate —
      `.github/workflows/ci.yml`

## Explicit non-goals

- No protocol is implemented in M0. A stub does not satisfy a gate.
- No native shell exists. The shells arrive in M3.
- No packaging, no signing, no store release.

## Gates

### Functional

- The core compiles and its unit tests are green:
  `cargo test -p phairplay-core`. **Green.**
- A default configuration is valid, an unknown key is rejected and a
  round trip through TOML preserves the value. **Green.**
- The session registry folds a started and an ended event and keeps a newer
  session alive when a late end event arrives. **Green.**
- The error hierarchy classifies every variant of the core into a kind, and
  the numeric codes are the documented ones. **Green.**

### Technical

- `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings` and `cargo fmt --check` are green. **Green.**
- The workspace denies `unsafe`, `unwrap()` and `expect()`. **Green.**
- The dependency direction is strict and is checked mechanically. **Green.**
- The documentation link check is green: `bash scripts/check-docs-links.sh`.
  **Green.**

### Performance

- The budgets of the core are the documented ones. The check of the mirror
  between `config/performance-budgets.toml` and the typed model is green.
  **Green** — this is a check of the *values*, not a measurement of the
  running receiver.

### Usability

- A newcomer who follows [`docs/GETTING-STARTED.md`](docs/GETTING-STARTED.md)
  builds the workspace and runs the tests without tribal knowledge.
  **Green.**

### Security and privacy

- The security policy of [`config/security-policy.toml`](config/security-policy.toml)
  is consistent with the code: the telemetry switch is off, the outbound
  policy forbids a connection outside the local subnet, the XML rules forbid
  a `DOCTYPE`, and the callback guard lists the blocked addresses.
  **Green.**

### Interoperability

- Not applicable. No protocol exists yet.

### Documentation

- Every document of the set exists, every internal link resolves, and every
  number exists in exactly one place. **Green.**

### QA evidence

- The output of the four Cargo commands and of the documentation check, in
  the acceptance record of M0:
  [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md).

## Release decision

M0 ships when the gate is green and the acceptance record names every command
that was run. M0 produces no binary for a user.

**Result: accepted.** The record was filed on the date it carries. The
reservations of the record — the device tests and the unmeasured budgets —
stay open and are carried by the milestones that follow.

---

# M1 — AirPlay 2 receiver *(Status: `in_progress` — the code is delivered, the acceptance record is missing)*

> **What is delivered:** the four format crates and the AirPlay engine are
> written and covered by tests. The proof of the milestone is missing: there
> is no acceptance record for M1 in
> [`docs/acceptance/`](docs/acceptance/ACCEPTANCE_TEMPLATE.md), and the gates
> that need a sender are **NOT RUN**.

## Purpose

The flagship path. A sender that a user actually owns — an iPhone, an iPad
or a Mac — mirrors its picture and its sound to the receiver, and the
receiver behaves like a device that has always been there.

## Sub-milestones

### M1.0 — The format layer *(Status: `in_progress` — delivered)*

Deliver:

- [x] `formats/plist`: the Apple property list in the XML **and** the binary
      format, both directions, with the hardening of the length fields, the
      recursion depth and the object references
- [x] `formats/h264`: the SPS and the PPS parser, the NAL unit splitter, the
      Annex-B handling and the RTP FU-A reassembly
- [x] `formats/aac`: the ADTS header, the AAC-ELD framing and the sample
      counting
- [x] `formats/alac`: the native Rust port of the Apple Lossless decoder
      with the attribution header in every derived file

### Gate

- Every public function of the four crates has a unit test. **Green** — the
  per-crate results are in the acceptance record of M0,
  [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md) §1.
- Every hardening measure has a negative test: a property list with a cyclic
  object graph, a declared length that exceeds the limit, the SPS overflow
  vector of `FR-056`, a fragment overflow. **Green** — the same record.
- The ALAC port is either complete or marked as partial with an exact list of
  the open parts. A partial port is never presented as a complete one.
  **Green** — the port is complete and carries the attribution of the
  reference decoder.

### M1.1 — Discovery *(Status: `in_progress` — delivered)*

Deliver:

- [x] The DNS-SD advertisement of `_airplay._tcp` **and** `_raop._tcp`
- [x] The SSDP browsing of the DLNA path
- [x] The sanitising and the length cap of the service instance name
- [x] The re-advertisement after a network change

### Gate

- A sender finds the receiver in the device picker of a real Apple device.
  **NOT RUN** — this gate needs a **device test**: an Apple device and a
  receiver on the same network. The plan is
  [`docs/guides/DEVICE_TEST_PLAN.md`](docs/guides/DEVICE_TEST_PLAN.md) §2.1.
- The instance name contains no control character and respects the DNS-SD
  length cap. **Green** — a unit test of the discovery crate.
- The advertisement comes back after a Wi-Fi reconnect without a restart.
  **NOT RUN** — this gate needs a **device test**: it needs a network whose
  interface is taken down and brought up again.

### M1.2 — The AirPlay engine *(Status: `in_progress` — delivered)*

Deliver:

- [x] The RTSP reader and the answer builder for the methods of the
      mirroring and the audio session
- [x] The HTTP-like routes of the receiver
- [x] The SDP parser: the media sections, the H.264 parameter sets, the ALAC
      parameters, the key material
- [x] The RTP reassembly and the delivery into the sinks
- [x] The state machine of the engine with the single-peer rule
- [x] The pairing: the Ed25519/X25519 procedure and the legacy SRP PIN
- [x] The FairPlay handshake and the decryption of the session key
- [x] The DACP reverse remote, the DMAP metadata, the NTP timing, the photo
      transfer

### Gate

- An iPhone or a Mac completes the handshake, and the picture and the sound
  reach the sink. **NOT RUN** — this gate needs a **device test** with an
  Apple sender.
- A second peer is refused and the first session continues. **Green** — a
  unit test of the engine.
- A malformed SDP, an oversized message and a header flood are refused
  without an allocation that follows an unvalidated length. **Green** — the
  negative tests of the engine.
- The session key of a known handshake is derived correctly. **Green** — a
  unit test of the engine.
- The audio and the video stay inside the synchronisation budget of
  [`config/performance-budgets.toml`](config/performance-budgets.toml) over a
  recording of at least one minute. **NOT RUN** — this gate needs a
  **device test**: it is a measurement of a running session.

### M1.3 — The evidence *(Status: `in_progress` — open)*

Deliver:

- [ ] The acceptance record of M1 in the format of
      [`docs/acceptance/ACCEPTANCE_TEMPLATE.md`](docs/acceptance/ACCEPTANCE_TEMPLATE.md)
- [x] The device test plan with the real senders of the matrix —
      [`docs/guides/DEVICE_TEST_PLAN.md`](docs/guides/DEVICE_TEST_PLAN.md)

### Gate

- Every statement in the acceptance record names the test or the command
  that produced it. **Open** — the record does not exist yet.
- Every sender of the matrix is marked as tested, or named with the reason
  why it could not be tested. **Open** — the plan names the senders, but no
  row of it has been executed on a device.

## Explicit non-goals

- No FairPlay DRM content (`EXC-001`).
- No Apple Music in-app audio (`EXC-002`).
- No buffered audio playback (`EXC-007`).
- No native shell. The picture is proven in the test harness of the engine and
  on the device in M3.

---

# M2 — Protocol breadth and local control *(Status: `in_progress` — the code is delivered, the acceptance record is missing)*

> **What is delivered:** the three engines, the bridge and the dashboard are
> written and covered by tests, and the dashboard is operable with a remote
> control — the proof is in
> [`docs/audit/tv-web-dashboard.md`](docs/audit/tv-web-dashboard.md). What is
> missing is the acceptance record of M2 and the device tests of the
> interoperability.

## Purpose

The receiver becomes a *device* and not a single-protocol experiment: the
other three protocols work, and a local dashboard shows and controls the
state.

## Sub-milestones

### M2.0 — The Google Cast receiver *(Status: `in_progress` — delivered)*

Deliver:

- [x] The TLS listener, the CastV2 framing, the heartbeat and the
      application identifier handling
- [x] The media channels and the media receiver status
- [x] The multi-peer exception, documented in exactly one place — `EXC-008`
      in [`REQUIREMENTS.md`](REQUIREMENTS.md)

### Gate

- Two Cast senders stream at the same time; no other protocol accepts a
  second peer. **NOT RUN** — this gate needs a **device test** with two
  Cast senders.
- A frame with an oversized length is refused. **Green** — the hardening
  tests of the engine.
- The baseline codecs of the protocol are the advertised ones, and a receiver
  without them does not advertise the service. **Green** — the capability
  test of the core.

### M2.1 — The Miracast receiver *(Status: `in_progress` — delivered)*

Deliver:

- [x] The WFD control plane, the information element parser, the sink
      discovery answer
- [x] The baseline codecs H.264 and linear PCM
- [x] The report of the missing WFD permission of the platform

### Gate

- A Windows sender connects and streams. **NOT RUN** — this gate needs a
  **device test** with a Windows sender.
- A platform without the WFD permission does not advertise the service and
  says why. **Green** — a unit test of the engine.
- An information element with a broken length is refused. **Green** — a
  negative test of the engine.

### M2.2 — The DLNA media renderer *(Status: `in_progress` — delivered)*

Deliver:

- [x] The SSDP answer and the notification
- [x] The device description and the service description documents
- [x] The SOAP dispatcher with the ConnectionManager, the AVTransport and the
      RenderingControl
- [x] The GENA events

### Gate

- A DLNA controller finds the device and starts a playback. **NOT RUN** —
  this gate needs a **device test** with a control point such as VLC.
- A SOAP body with a `DOCTYPE` and a GENA callback that points at
  `169.254.169.254` are both refused, and the receiver never contacts the
  refused address. **Green** — the hardening tests of the engine.
- The renderer advertises no video rendering. **Green** — a unit test of the
  core.

### M2.3 — The platform bridge *(Status: `in_progress` — delivered)*

Deliver:

- [x] The data transfer objects of the bridge, the receiver facade and the
      error mapping with the stable numeric code
- [x] The generation of the Kotlin and the Swift bindings
- [x] The round trip test per exported type

### Gate

- A value that goes into the bridge comes out of it unchanged. **Green** —
  the round-trip tests of the bridge.
- No panic crosses the bridge; every failure is a typed error with a code.
  **Green** — the error tests of the bridge.
- The generated bindings are reproducible: a regeneration produces no diff.
  **Green** — the generator is
  [`scripts/generate-bindings.sh`](scripts/generate-bindings.sh).

### M2.4 — The local dashboard *(Status: `in_progress` — delivered)*

Deliver:

- [x] The Rust backend with the status, the session, the metric, the
      configuration and the build information endpoint, the switch of a
      protocol and the event channel
- [x] The dashboard with one card per protocol, the live counters and the
      accessibility
- [x] The dashboard as a **remote control interface**: the arrow keys move
      the focus inside the group of the cards (a roving tabindex — one tab
      stop for the group), the focus frame is visible and measured, and the
      card itself is the switch
- [x] The **headless** mode of the backend: the API stays available, the
      files of the panel are left out (`--no-dashboard`)

### Gate

- The backend binds on the loopback interface only; a request from a foreign
  address is refused. **Green** — a unit test of the backend. A bind to
  another address is possible, but it prints a warning on stderr before
  anything is started.
- Every value of the dashboard comes from the core; the backend invents
  nothing. **Green** — the design of the backend; the status is read from
  the supervisor.
- The dashboard is operable with the keyboard, meets the contrast
  requirement, respects the reduced-motion preference and loads no external
  resource. **Green for the keyboard, the contrast and the motion** — the
  three checks of the dashboard are in
  [`docs/audit/tv-web-dashboard.md`](docs/audit/tv-web-dashboard.md), with
  the measured values. **NOT RUN** — a television with a real remote
  control, and the reading of the panel on a real television in a bright
  room: both need a **device test**.

## Explicit non-goals

- No Widevine and no PlayReady (`EXC-004`).
- No transcode (`EXC-009`).
- No remote access to the dashboard.

---

# M3 — Native shells *(Status: `in_progress` — the code is delivered, the evidence is in `docs/audit/`, the acceptance record is missing)*

> **What is delivered:** the Android shell and the macOS shell are written,
> and their evidence is filed in [`docs/audit/`](docs/audit/tv-android-shell.md):
> the Kotlin of the Android shell compiles, its unit tests are green, the
> lint is green; the kit of the macOS shell has a green test suite. What is
> missing is the acceptance record of M3 and every gate that needs a device.

## Purpose

The receiver becomes an application that a user installs, starts and
forgets. The shell shows the state, the switch and the session — and nothing
else.

## Deliver

- [x] The Android shell: the foreground service, the persistent notification,
      the status surface with one card per protocol, the switch per protocol
- [x] The minimal permission list of the Android manifest
- [x] The macOS shell: the same information architecture, natively
- [x] The generated bindings for both platforms, without a hand edit
- [x] The shells as **remote control interfaces**: the D-pad moves the focus
      through a deterministic focus graph, the focus frame is visible, the
      card is the switch
- [x] The Android shell as a **television application**: the
      `LEANBACK_LAUNCHER` category, the banner of the launcher and the
      `uses-feature` of the television

## Gates

### Functional

- The state of every protocol is visible in both shells, and the four states
  — disabled, advertising, streaming, error — are distinguishable. "Stopped"
  and "disabled" are not the same message. **Green in the code** — the
  mapping of the bridge into the model of the shell is covered by the tests
  of the Android shell and of the macOS kit. **NOT RUN** — on a device.
- A protocol can be switched on and off from the shell, and the change is
  visible in the sender's device picker. **NOT RUN** — this gate needs a
  **device test** with a sender.

### Technical

- The shell reaches the core only through the bridge. The boundary check is
  green. **Green.**
- The Android manifest contains no location, no microphone and no contact
  permission. **Green** — the merged manifest is in the audit of the Android
  shell.

### Performance

- The resident memory and the CPU usage of the shell stay inside the budgets
  of [`config/performance-budgets.toml`](config/performance-budgets.toml) with
  a running session. **NOT RUN** — this gate needs a **device test** with a
  running session.

### Usability

- A session survives the backgrounding of the interface, and the notification
  is present while the service runs. **NOT RUN** — this gate needs a
  **device test**.
- The texts of the interface are in the resources, not in the code. **Green**
  — the audit of the Android shell shows three locales with the same keys and
  no literal in the layer of the interface; the audit of the macOS shell
  shows the same for the string catalogue of the kit.

### Security and privacy

- The application asks for no permission that the receiver does not need.
  **Green** — the merged manifest of the Android shell is in its audit.

### Interoperability

- The picture of a real sender reaches the display of the real device.
  **NOT RUN** — this gate needs a **device test**.

### Documentation

- The screenshots and the steps of the device test plan are part of the
  acceptance record. **Open** — the acceptance record of M3 does not exist
  yet, and a screenshot needs a device.

## Explicit non-goals

- No store release. The signing and the notarisation are part of M4.
- No tablet-specific layout beyond what the phone layout scales to.

---

# M4 — Hardening, interoperability and release readiness *(Status: `in_progress` — largely delivered, the device tests and the release notes are missing)*

> **What is delivered:** the hardening of the parsers, the masked logs, the
> dependency policy, the continuous integration, the licence inventory, the
> security and the privacy statements, the device profiles and the headless
> operation of the receiver. What is missing: the acceptance record of the
> whole workspace, the device tests of the interoperability and the release
> notes.

## Purpose

The receiver is ready to be handed to a user: the attacks are closed, the
gate is real, the licences are in order and the limitations are written
down.

## Deliver

- [x] The hardening of every parser against the attacks of the threat model
- [x] The masking of the logs
- [x] The dependency policy with the pinned version of the licence gate —
      [`deny.toml`](deny.toml) and [`scripts/audit-licenses.sh`](scripts/audit-licenses.sh)
- [x] The complete continuous integration with the licence, the audit and the
      structure jobs — [`.github/workflows/ci.yml`](.github/workflows/ci.yml)
- [ ] The acceptance record of the whole workspace — the record of M0 exists
      ([`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md));
      the records of M1 to M5 are missing
- [x] The licence inventory of the third-party components —
      [`THIRD_PARTY.md`](THIRD_PARTY.md)
- [x] The privacy and the security statement for the users —
      [`PRIVACY.md`](PRIVACY.md) and [`SECURITY.md`](SECURITY.md)
- [x] The build from source — the quickstart of
      [`README.md`](README.md) and the installer
      [`deploy/install.sh`](deploy/install.sh)
- [ ] The release notes — `docs/releasing/` is empty
- [x] The **other devices**: the device profiles of
      [`config/profiles/`](config/profiles/README.md), the headless operation
      of the receiver, the installation as a service (systemd) and as a
      container ([`deploy/`](deploy/install.sh))

## Gates

### Functional

- Every requirement of [`REQUIREMENTS.md`](REQUIREMENTS.md) is either
  demonstrated by a test or named in the known limitations of the acceptance
  record. **Open** — the record of M0 covers the requirements of that day;
  the requirements that came with the remote control interface, the headless
  operation and Smart View are not in a record yet.

### Technical

- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy
  --workspace --all-targets -- -D warnings` and `cargo fmt --check` are green.
  **Green** — the acceptance record of M0 and the continuous integration.
- The documentation check, the boundary check and the licence check are green.
  **Green** — the same record.
- No test is skipped, ignored or commented out. No `todo!()`, no
  `unimplemented!()`. **Green.**
- No file is longer than the hard limit of
  [`config/performance-budgets.toml`](config/performance-budgets.toml).
  **Green** — `bash scripts/check-file-size.sh`.

### Performance

- The measured latency, frame rate, memory and CPU are inside the budgets,
  and every number in the acceptance record names the command that produced
  it. **NOT RUN** — this gate needs a **device test**: the budgets are
  properties of the running receiver with a real sender.

### Usability

- The dashboard and the shells reach the common tasks in a comparable number
  of steps, and the comparison is written down. **Open** — the comparison
  needs a device test on both sides.
- The receiver is usable **without a screen**: the program starts the
  protocols itself and reports the outcome of every one of them on the
  console. **Green** — the tests of the command line and of the composition
  root of the web backend.

### Security and privacy

- Every attack of the threat model has a negative test that really runs it.
  **Green** — the negative tests of the format crates and of the engines.
- The mask of the logs is proven by a test that searches a rendered line.
  **Green** — the privacy tests of the core.
- The dependency licences are compatible with the AGPL, and the attribution
  of the ALAC port is in place. **Green** — the licence gate and the header
  of the ALAC crate.
- No telemetry, no account, no outbound connection outside the local subnet.
  **Green** — the policy of
  [`config/security-policy.toml`](config/security-policy.toml) and the tests
  of the core.

### Interoperability

- The device test plan is executed and every sender of the matrix has a
  result. **NOT RUN** — this gate needs a **device test**; the plan is
  [`docs/guides/DEVICE_TEST_PLAN.md`](docs/guides/DEVICE_TEST_PLAN.md) and
  every row of it is still open.
- The limitations of the interoperability — the FairPlay DRM, the Apple Music
  in-app audio, the HDCP sessions, the Cast application identifier, the
  proprietary message format of the Smart View channel — are written in the
  user-facing documentation with the same words as in the requirements.
  **Green** — the exclusion list of
  [`REQUIREMENTS.md`](REQUIREMENTS.md) and the section of
  [`README.md`](README.md).

### Documentation

- Every document of the set is current. No statement of the documentation
  contradicts the code. **Green** — the documentation gate of the continuous
  integration, and the change that this milestone carried.

### QA evidence

- The acceptance record of the workspace, with the evidence of every gate.
  **Open** — the record of M0 exists; the records of M1 to M5 are missing.

## Explicit non-goals

- No public publication of a binary. The distribution decision is not part of
  this milestone.

---

# M5 — Smart View *(Status: `in_progress` — the code and the acceptance record are delivered, the device tests are missing)*

> **What is delivered:** the fifth protocol of the vocabulary
> (`ProtocolKind::SmartView`), the service type `_samsungmsf._tcp`, the port
> of the API and the engine crate `engines/smartview` with the discovery, the
> description document, the HTTP API and the handshake of the sender.
>
> **What is not delivered:** the message format of
> the channel. That format is proprietary and not publicly documented, so the
> content of the channel is not implemented: a Galaxy device finds the
> receiver and connects to it, but no picture flows yet. The limit is written
> down as `EXC-011` in [`REQUIREMENTS.md`](REQUIREMENTS.md).

## Purpose

The screen mirroring of the Galaxy phones and tablets. The protocol is the
one that Samsung calls *Smart View*, and its framework is the *MultiScreen
Framework* (MSF).

## Deliver

- [x] The protocol in the vocabulary of the core: `ProtocolKind::SmartView`,
      the display name, the service type `_samsungmsf._tcp` and the port of
      the API — `core/phairplay-core/src/protocol.rs` and
      `core/phairplay-core/src/config/`
- [x] The engine crate `engines/smartview`: the SSDP answer of the
      MultiScreen search, the announcement and the farewell
- [x] The description document that the `LOCATION` header of the answer
      points at
- [x] The HTTP API below `/api/v2/`: the root, the resource description, the
      base URL and the handshake of the sender
- [x] The WebSocket upgrade of the channel and the single-session rule
- [x] The registration of the engine in the composition root
      (`platform/phairplay-ffi/src/composition.rs`) — the receiver starts it with the other
      protocols and reports it in the status; the conflict of the SSDP port
      is named under the gates
- [x] The card of the fifth protocol in the dashboard — the panel of the
      browser lists five protocols, the state badge of a card in a row of
      five got a tighter tracking, and the grid carries five columns on a
      television (`apps/web/assets/api.js`, `apps/web/assets/cards.css`)
- [x] The card of the fifth protocol in the native shells — the Android shell
      and the macOS shell name the fifth protocol in their enumeration of the
      protocols, map it from the kind of the bridge, read its configuration
      from the defaults of the shell and draw it in the grid of the cards
      (`apps/android/app/src/main/java/org/phairplay/receiver/model/ProtocolId.kt`,
      `apps/macos/Sources/PhairPlayMacKit/ReceiverModel.swift`)
- [ ] The message format of the channel — **not implemented**, the format is
      proprietary (`EXC-011`)

## Gates

### Functional

- A Galaxy device finds the receiver in the list of *Smart View* and
  connects to it. **NOT RUN** — this gate needs a **device test** with a
  Galaxy phone or tablet.
- The description document is served and the handshake of the sender is
  answered. **Green** — the tests of the API of the engine.
- The engine starts, advertises, stops and frees its ports. **Green** — the
  tests of the engine.
- The protocol is among the protocols that the receiver starts. **Green** —
  the run of the receiver shows it:

  ```text
  $ phairplay-web --profile generic --no-dashboard --port 7879
  phairplay-web: the receiver starts 5 protocols:
    AirPlay 2        advertising
    Miracast         advertising
    Google Cast      advertising
    DLNA / UPnP      advertising
    Samsung Smart View advertising
  phairplay-web: 5 of 5 protocols are running.
  ```

  The conflict of the two responders is resolved. DLNA and Smart View are
  both discovered over SSDP, and SSDP lives on one multicast group — and
  **Linux allows only one socket per multicast group**. The receiver
  therefore owns **one** listener for the whole process, and every service
  registers itself with it
  ([`ADR-0007`](docs/adr/ADR-0007-one-ssdp-listener-for-the-whole-receiver.md)).
  The proof is in the network and not only in the status: a search for the
  renderer is answered by the renderer and by nothing else, a search for the
  framework is answered by the framework and by nothing else, and a search
  for `ssdp:all` is answered by both — once per service type, as the UPnP
  device architecture asks.

  **The user decides which protocols run.** The switch of a protocol is the
  key `enabled` in its section of the configuration, and switching one of
  the two off keeps the other one running — the listener belongs to the
  process, not to one protocol.

### Technical

- The engine is a `ProtocolEngine` like every other engine, and it keeps the
  shape of the build contract: one peer at a time, every read has a
  deadline, every input is length-limited. **Green** — the tests of the
  engine and of the API.
- The crate pulls no web framework into the workspace: the API of the engine
  is a small hand-written HTTP/1.1 server. **Green** — the dependencies of
  `engines/smartview/Cargo.toml`.

### Performance

- The time from the start of the process to the advertisement of the receiver
  stays inside the budget of the advertisement path. **NOT RUN** — this gate
  needs a **device test**.

### Usability

- The name of the device in the list of a Galaxy device is the configured
  name and nothing else. **Green** — the test of the description document.

### Security and privacy

- The engine opens no connection outside the local subnet and reads no
  configuration of the user. **Green** — the design of the engine; the
  limits of the reader are the limits of the configuration.

### Interoperability

- The picture of a Galaxy device reaches the display of the receiver.
  **NOT RUN and not implementable yet** — the message format of the channel
  is proprietary. This gate stays open until the format is documented by
  Samsung or until another implementation of it exists under a licence that
  allows the port.

### Documentation

- The honest limit is written down in the requirements, in the glossary, in
  the architecture and in the user-facing documentation. **Green** —
  `EXC-011`.

### QA evidence

- The acceptance record of M5. **Green** —
  [`docs/acceptance/M5-ACCEPTANCE.md`](docs/acceptance/M5-ACCEPTANCE.md). The
  record names the tests, the measurements and every gate that is **NOT RUN**
  with the exact reason.
- The gate of the project. **Green** — 1237 tests of the workspace, 0 fail.
  The two native shells have their own gate since the job *Shells (Android,
  macOS)*: 101 tests of the kit of the macOS shell and 39 unit tests of the
  Android shell, 0 failures.
- The licence audit and the advisory audit. **Green** — the jobs *Licenses
  (cargo-deny, pinned)* and *Security Audit (cargo-audit)*.

## Explicit non-goals

- No invention of the message format of the channel. A format that nobody
  documents is not guessed.
- No DRM of any kind: Smart View is an open feature of the Galaxy devices,
  not a DRM pipeline like FairPlay (`EXC-001`).

---

# The release plan

> **What this section is:** the plan of the releases — when each kind of
> release happens, what triggers it, and which platform ships a binary with
> it.
>
> **The rule that governs this section:** a release is triggered by a
> **state**, not by a date. The project has no deadline and promises none.
> What is written here is the *condition* under which a release happens —
> and a condition that is met can be checked by anybody, while a date can
> only be waited for.
>
> **The process that owns the steps** is
> [`docs/releasing/RELEASING.md`](docs/releasing/RELEASING.md). The process
> says *how*; this section says *when* and *what*.

## The three kinds of a release

| Kind | What it means | What it ships |
|---|---|---|
| **Alpha** | the code of the milestones that are delivered is in the tree and the gate of the workspace is green; the device evidence is missing | the **source** and the **headless receiver** as a binary for the server platforms. **No** binary of a shell |
| **Beta** | the shells build and their tests are green; the device tests of **some** milestones are run | the source, the headless receiver, and the **first binary of a shell** — the platform whose device test is green |
| **Release** | every milestone the release carries has a green acceptance record, the device tests are run, and the budgets are measured | the source, the headless receiver, and the **binaries of every shell** whose device test is green |

**A binary is never published before its own device test is green.** The
rule of [`docs/releasing/RELEASING.md`](docs/releasing/RELEASING.md) §8: a
binary that nobody checked is a promise that nobody made. The trigger of a
binary is therefore **its own platform**, not the release as a whole.

## The plan

| Release | The trigger — the condition that has to be met | The platforms that ship a binary | The artefacts |
|---|---|---|---|
| **v0.3.0-alpha.1** | M5 is delivered — the fifth protocol is in the vocabulary of the core, in the bridge and in the dashboard; the gate of the workspace is green; the acceptance record of M5 is written and names its open points | **Linux (x86-64, aarch64)**, **Windows (x86-64)** — the headless receiver | the source, the tarball of the receiver with its `systemd` unit and its compose file, the checksums |
| **v0.3.0-beta.1** | the first device test of a shell is green — the shell is built from the tag, installed on a real device and operated with a remote control | **Android TV / Fire TV (arm64-v8a)** — the first binary of a shell — plus **Linux** and **Windows** | the APK with the native bridge, the headless receiver, the checksums |
| **v0.3.0-beta.2** | the device test of the second shell is green | **macOS 13+ (Apple silicon)** — the `.app` — plus the artefacts of beta.1 | the signed and notarised `.app` in the `.dmg`, the headless receiver, the checksums |
| **v0.4.0** | the picture of the Smart View channel flows — the message format is implemented, or the format stays an exclusion and the release says so | **Android**, **macOS**, **Linux**, **Windows** | the source, every binary, the checksums, the signatures |
| **v1.0.0** | the milestones **M0 to M5** are `done`; every acceptance record is green; every budget is measured; the device test plan is complete | **Android**, **macOS**, **Linux**, **Windows** | the source, every binary, the checksums, the signatures, the signatures of the licences |

**Why the numbers move.** The version is not decided in advance — it is the
consequence of the state. `0.3` is the first version after the `0.2`
pre-release, `0.4` carries the picture of the fifth protocol, and `1.0` is
the version that says: the milestones are done. The single owner of the
number is [`VERSION`](VERSION) and the decision belongs to the owner
([`docs/releasing/RELEASING.md`](docs/releasing/RELEASING.md) §4).

## What each platform needs before it ships a binary

The trigger above is the same for every platform — the device test of *that*
platform. What differs is what has to be true before the binary can be
built at all.

| Platform | The binary | What has to be true before it is built | What has to be true before it is published |
|---|---|---|---|
| **Linux** (x86-64, aarch64) | `phairplay-web` — the headless receiver with the dashboard | the crate builds for the target with the pinned dependencies of [`Cargo.lock`](Cargo.lock) | the binary is started on the target, the protocols advertise themselves, and the install script of [`deploy/`](deploy/) works |
| **Windows** (x86-64) | `phairplay-web.exe` | the crate builds for the Rust target of Windows on the 64-bit architecture; the service installation is written down | the binary is started on the target and the protocols advertise themselves |
| **Android** (arm64-v8a, API 33+) | the APK with the native bridge of [`platform/phairplay-ffi/`](platform/phairplay-ffi/) | the NDK builds `libphairplay_ffi.so` for `aarch64-linux-android`; the Kotlin compiles; the unit tests are green | the APK is installed on a **Google TV or Fire TV**; a sender connects and a picture flows; the remote control operates the shell |
| **macOS** (13+, Apple silicon) | the `.app` of [`apps/macos/`](apps/macos/) | the Swift package builds on a machine with the macOS SDK; the kit tests are green; the application is signed | the `.app` is notarised and opens on a Mac; a sender connects and a picture flows; the remote control operates the shell |

**The binaries of a release are built from the tag**, with the pinned
dependencies of [`Cargo.lock`](Cargo.lock) and with the toolchain that the
tag names. The notes of the release name the toolchain of every binary — a
binary without its toolchain is not reproducible.

## What is not in the plan

- **A binary of a shell without its device test.** The device test is the
  trigger; there is no other path.
- **A date.** The project promises no date and this plan promises none. The
  condition is checked; the date is waited for.
- **A binary of a platform whose milestone is not done.** The Linux shell is
  a later track
  ([`PROJECT_MANAGEMENT.md`](PROJECT_MANAGEMENT.md)); the headless receiver
  on Linux is not that track.

---

# Later tracks

These are accepted directions that are deliberately not scheduled. They start
only when the milestone before them is green.

| Track | Content | Entry condition |
|---|---|---|
| Linux shell | A native Linux application | A green M4 |
| Windows shell | A native Windows application | A green Linux track |
| Transcoding | A format conversion in the receiver | A green M4 and a decision about the CPU budget |
| Control from outside the local network | A control channel that reaches the receiver from another network — **not** the remote control interface of the dashboard and of the shells, which runs inside the local network | a decision about the privacy promise — the current promise forbids it |

A track that is not listed here does not exist. A feature that is not written
down does not happen.
