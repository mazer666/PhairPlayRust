# PhairPlayRust Roadmap

**Roadmap version:** 0.1
**Current milestone:** M0 — Foundation (**IN PROGRESS**)

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

# M0 — Foundation *(Status: `in_progress`)*

## Purpose

Prove that the architecture works before a single protocol is implemented:
one workspace, one shared core, one build contract, one central
configuration, one documentation landscape, one gate.

## Deliver

- [x] Cargo workspace with eleven members: the core, five engine crates, four
      format crates and the platform bridge
- [x] The shared core: the protocol trait, the session registry, the media
      sinks, the capabilities, the configuration, the identity, the metrics,
      the error hierarchy, the job executor
- [x] The build contract `docs/design/BUILD-CONTRACT.md`
- [x] The central configuration: defaults, performance budgets, security
      policy
- [x] The licence, the reuse metadata and the licence inventory directory
- [ ] The repository gate: a script that checks the documentation links, the
      crate boundaries, the format, the lint and the tests
- [ ] The documentation set in the house format: the root documents, the
      `docs/` tree, the templates and the guides
- [ ] The continuous integration with the jobs of the quality gate

## Explicit non-goals

- No protocol is implemented in M0. A stub does not satisfy a gate.
- No native shell exists. The shells arrive in M3.
- No packaging, no signing, no store release.

## Gates

### Functional

- The core compiles and its unit tests are green:
  `cargo test -p phairplay-core`.
- A default configuration is valid, an unknown key is rejected and a
  round trip through TOML preserves the value.
- The session registry folds a started and an ended event and keeps a newer
  session alive when a late end event arrives.
- The error hierarchy classifies every variant of the core into a kind, and
  the numeric codes are the documented ones.

### Technical

- `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings` and `cargo fmt --check` are green.
- The workspace denies `unsafe`, `unwrap()` and `expect()`.
- The dependency direction is strict and is checked mechanically.
- The documentation link check is green: `bash scripts/check-docs-links.sh`.

### Performance

- The budgets of the core are the documented ones. The check of the mirror
  between `config/performance-budgets.toml` and the typed model is green.

### Usability

- A newcomer who follows [`docs/GETTING-STARTED.md`](docs/GETTING-STARTED.md)
  builds the workspace and runs the tests without tribal knowledge.

### Security and privacy

- The security policy of [`config/security-policy.toml`](config/security-policy.toml)
  is consistent with the code: the telemetry switch is off, the outbound
  policy forbids a connection outside the local subnet, the XML rules forbid
  a `DOCTYPE`, and the callback guard lists the blocked addresses.

### Interoperability

- Not applicable. No protocol exists yet.

### Documentation

- Every document of the set exists, every internal link resolves, and every
  number exists in exactly one place.

### QA evidence

- The output of the four Cargo commands and of the documentation check, in the
  acceptance record of M0.

## Release decision

M0 ships when the gate is green and the acceptance record names every command
that was run. M0 produces no binary for a user.

---

# M1 — AirPlay 2 receiver *(Status: `planned`)*

## Purpose

The flagship path. A sender that a user actually owns — an iPhone, an iPad
or a Mac — mirrors its picture and its sound to the receiver, and the
receiver behaves like a device that has always been there.

## Sub-milestones

### M1.0 — The format layer *(Status: `planned`)*

Deliver:

- [ ] `formats/plist`: the Apple property list in the XML **and** the binary
      format, both directions, with the hardening of the length fields, the
      recursion depth and the object references
- [ ] `formats/h264`: the SPS and the PPS parser, the NAL unit splitter, the
      Annex-B handling and the RTP FU-A reassembly
- [ ] `formats/aac`: the ADTS header, the AAC-ELD framing and the sample
      counting
- [ ] `formats/alac`: the native Rust port of the Apple Lossless decoder
      with the attribution header in every derived file

### Gate

- Every public function of the four crates has a unit test.
- Every hardening measure has a negative test: a property list with a cyclic
  object graph, a declared length that exceeds the limit, the SPS overflow
  vector of `FR-056`, a fragment overflow.
- The ALAC port is either complete or marked as partial with an exact list of
  the open parts. A partial port is never presented as a complete one.

### M1.1 — Discovery *(Status: `planned`)*

Deliver:

- [ ] The DNS-SD advertisement of `_airplay._tcp` **and** `_raop._tcp`
- [ ] The SSDP browsing of the DLNA path
- [ ] The sanitising and the length cap of the service instance name
- [ ] The re-advertisement after a network change

### Gate

- A sender finds the receiver in the device picker of a real Apple device.
- The instance name contains no control character and respects the DNS-SD
  length cap.
- The advertisement comes back after a Wi-Fi reconnect without a restart.

### M1.2 — The AirPlay engine *(Status: `planned`)*

Deliver:

- [ ] The RTSP reader and the answer builder for the methods of the
      mirroring and the audio session
- [ ] The HTTP-like routes of the receiver
- [ ] The SDP parser: the media sections, the H.264 parameter sets, the ALAC
      parameters, the key material
- [ ] The RTP reassembly and the delivery into the sinks
- [ ] The state machine of the engine with the single-peer rule
- [ ] The pairing: the Ed25519/X25519 procedure and the legacy SRP PIN
- [ ] The FairPlay handshake and the decryption of the session key
- [ ] The DACP reverse remote, the DMAP metadata, the NTP timing, the photo
      transfer

### Gate

- An iPhone or a Mac completes the handshake, and the picture and the sound
  reach the sink.
- A second peer is refused and the first session continues.
- A malformed SDP, an oversized message and a header flood are refused
  without an allocation that follows an unvalidated length.
- The session key of a known handshake is derived correctly.
- The audio and the video stay inside the synchronisation budget of
  [`config/performance-budgets.toml`](config/performance-budgets.toml) over a
  recording of at least one minute.

### M1.3 — The evidence *(Status: `planned`)*

Deliver:

- [ ] The acceptance record of M1 in the format of
      [`docs/acceptance/ACCEPTANCE_TEMPLATE.md`](docs/acceptance/ACCEPTANCE_TEMPLATE.md)
- [ ] The device test plan with the real senders of the matrix

### Gate

- Every statement in the acceptance record names the test or the command
  that produced it.
- Every sender of the matrix is marked as tested, or named with the reason
  why it could not be tested.

## Explicit non-goals

- No FairPlay DRM content (`EXC-001`).
- No Apple Music in-app audio (`EXC-002`).
- No buffered audio playback (`EXC-007`).
- No native shell. The picture is proven in the test harness of the engine and
  on the device in M3.

---

# M2 — Protocol breadth and local control *(Status: `planned`)*

## Purpose

The receiver becomes a *device* and not a single-protocol experiment: the
other three protocols work, and a local dashboard shows and controls the
state.

## Sub-milestones

### M2.0 — The Google Cast receiver *(Status: `planned`)*

Deliver:

- [ ] The TLS listener, the CastV2 framing, the heartbeat and the
      application identifier handling
- [ ] The media channels and the media receiver status
- [ ] The multi-peer exception, documented in exactly one place

### Gate

- Two Cast senders stream at the same time; no other protocol accepts a
  second peer.
- A frame with an oversized length is refused.
- The baseline codecs of the protocol are the advertised ones, and a receiver
  without them does not advertise the service.

### M2.1 — The Miracast receiver *(Status: `planned`)*

Deliver:

- [ ] The WFD control plane, the information element parser, the sink
      discovery answer
- [ ] The baseline codecs H.264 and linear PCM
- [ ] The report of the missing WFD permission of the platform

### Gate

- A Windows sender connects and streams.
- A platform without the WFD permission does not advertise the service and
  says why.
- An information element with a broken length is refused.

### M2.2 — The DLNA media renderer *(Status: `planned`)*

Deliver:

- [ ] The SSDP answer and the notification
- [ ] The device description and the service description documents
- [ ] The SOAP dispatcher with the ConnectionManager, the AVTransport and the
      RenderingControl
- [ ] The GENA events

### Gate

- A DLNA controller finds the device and starts a playback.
- A SOAP body with a `DOCTYPE` and a GENA callback that points at
  `169.254.169.254` are both refused, and the receiver never contacts the
  refused address.
- The renderer advertises no video rendering.

### M2.3 — The platform bridge *(Status: `planned`)*

Deliver:

- [ ] The data transfer objects of the bridge, the receiver facade and the
      error mapping with the stable numeric code
- [ ] The generation of the Kotlin and the Swift bindings
- [ ] The round trip test per exported type

### Gate

- A value that goes into the bridge comes out of it unchanged.
- No panic crosses the bridge; every failure is a typed error with a code.
- The generated bindings are reproducible: a regeneration produces no diff.

### M2.4 — The local dashboard *(Status: `planned`)*

Deliver:

- [ ] The Rust backend with the status, the session, the metric, the
      configuration and the build information endpoint, the switch of a
      protocol and the event channel
- [ ] The dashboard with one card per protocol, the live counters and the
      accessibility

### Gate

- The backend binds on the loopback interface only; a request from a foreign
  address is refused.
- Every value of the dashboard comes from the core; the backend invents
  nothing.
- The dashboard is operable with the keyboard, meets the contrast
  requirement, respects the reduced-motion preference and loads no external
  resource.

## Explicit non-goals

- No Widevine and no PlayReady (`EXC-004`).
- No transcode (`EXC-009`).
- No remote access to the dashboard.

---

# M3 — Native shells *(Status: `planned`)*

## Purpose

The receiver becomes an application that a user installs, starts and
forgets. The shell shows the state, the switch and the session — and nothing
else.

## Deliver

- [ ] The Android shell: the foreground service, the persistent notification,
      the status surface with one card per protocol, the switch per protocol
- [ ] The minimal permission list of the Android manifest
- [ ] The macOS shell: the same information architecture, natively
- [ ] The generated bindings for both platforms, without a hand edit

## Gates

### Functional

- The state of every protocol is visible in both shells, and the four states
  — disabled, advertising, streaming, error — are distinguishable. "Stopped"
  and "disabled" are not the same message.
- A protocol can be switched on and off from the shell, and the change is
  visible in the sender's device picker.

### Technical

- The shell reaches the core only through the bridge. The boundary check is
  green.
- The Android manifest contains no location, no microphone and no contact
  permission.

### Performance

- The resident memory and the CPU usage of the shell stay inside the budgets
  of [`config/performance-budgets.toml`](config/performance-budgets.toml) with
  a running session.

### Usability

- A session survives the backgrounding of the interface, and the notification
  is present while the service runs.
- The texts of the interface are in the resources, not in the code.

### Security and privacy

- The application asks for no permission that the receiver does not need.

### Interoperability

- The picture of a real sender reaches the display of the real device.

### Documentation

- The screenshots and the steps of the device test plan are part of the
  acceptance record.

## Explicit non-goals

- No store release. The signing and the notarisation are part of M4.
- No tablet-specific layout beyond what the phone layout scales to.

---

# M4 — Hardening, interoperability and release readiness *(Status: `planned`)*

## Purpose

The receiver is ready to be handed to a user: the attacks are closed, the
gate is real, the licences are in order and the limitations are written
down.

## Deliver

- [ ] The hardening of every parser against the attacks of the threat model
- [ ] The masking of the logs
- [ ] The dependency policy with the pinned version of the licence gate
- [ ] The complete continuous integration with the licence, the audit and the
      structure jobs
- [ ] The acceptance record of the whole workspace
- [ ] The licence inventory of the third-party components
- [ ] The privacy and the security statement for the users
- [ ] The build from source and the release notes

## Gates

### Functional

- Every requirement of [`REQUIREMENTS.md`](REQUIREMENTS.md) is either
  demonstrated by a test or named in the known limitations of the acceptance
  record.

### Technical

- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy
  --workspace --all-targets -- -D warnings` and `cargo fmt --check` are green.
- The documentation check, the boundary check and the licence check are green.
- No test is skipped, ignored or commented out. No `todo!()`, no
  `unimplemented!()`.
- No file is longer than the hard limit of
  [`config/performance-budgets.toml`](config/performance-budgets.toml).

### Performance

- The measured latency, frame rate, memory and CPU are inside the budgets,
  and every number in the acceptance record names the command that produced
  it.

### Usability

- The dashboard and the shells reach the common tasks in a comparable number
  of steps, and the comparison is written down.

### Security and privacy

- Every attack of the threat model has a negative test that really runs it.
- The mask of the logs is proven by a test that searches a rendered line.
- The dependency licences are compatible with the AGPL, and the attribution
  of the ALAC port is in place.
- No telemetry, no account, no outbound connection outside the local subnet.

### Interoperability

- The device test plan is executed and every sender of the matrix has a
  result.
- The limitations of the interoperability — the FairPlay DRM, the Apple Music
  in-app audio, the HDCP sessions, the Cast application identifier — are
  written in the user-facing documentation with the same words as in the
  requirements.

### Documentation

- Every document of the set is current. No statement of the documentation
  contradicts the code.

### QA evidence

- The acceptance record of the workspace, with the evidence of every gate.

## Explicit non-goals

- No public publication of a binary. The distribution decision is not part of
  this milestone.

---

# Later tracks

These are accepted directions that are deliberately not scheduled. They start
only when the milestone before them is green.

| Track | Content | Entry condition |
|---|---|---|
| Linux shell | A native Linux application | A green M4 |
| Windows shell | A native Windows application | A green Linux track |
| Transcoding | A format conversion in the receiver | A green M4 and a decision about the CPU budget |
| Remote control | A control channel from outside the local network | A decision about the privacy promise — the current promise forbids it |

A track that is not listed here does not exist. A feature that is not written
down does not happen.
