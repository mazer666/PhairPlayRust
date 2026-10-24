# PhairPlayRust

**PhairPlayRust** is a free, open-source, privacy-first receiver for **AirPlay 2**,
**Miracast**, **Google Cast**, **DLNA/UPnP** and **Samsung Smart View**. It has
**no account, no telemetry, no advertising and no cloud dependency** — it only
ever speaks to devices on your local network.

> **Product principle:** A receiver that does not want anything from you.

The receiver is written in Rust. The native shells are Kotlin/Compose
(Android) and SwiftUI (macOS); the local dashboard is a small Rust web
backend with static assets. One shared Rust core carries the protocol logic,
so the five protocols behave the same way and the logic is testable without a
network and without a decoder.

---

## Current project status

- **Baseline:** Documentation Baseline 0.1
- **Current milestone:** **M5** — the fifth protocol, Smart View (the code
  stands; the acceptance record is missing — see
  [`ROADMAP.md`](ROADMAP.md))
- **Shared core:** Rust (`core/phairplay-core`)
- **Protocol engines:** `engines/{airplay,miracast,cast,dlna,smartview,discovery}`
- **Format layer:** `formats/{plist,h264,aac,alac}`
- **Platform bridge:** UniFFI (`platform/phairplay-ffi`)
- **License:** AGPL-3.0-or-later, DCO, no CLA
- **Machine-readable state:** [`project/project.yaml`](project/project.yaml)

### What is in the repository

| Part | State |
|---|---|
| Cargo workspace with thirteen crates | in place, compiles clean |
| Shared core: protocol trait, session registry, media sinks, capabilities, configuration, identity, metrics, error hierarchy, job executor, bounded queue, log masking | in place, with unit tests |
| Build contract for the crate interfaces | in place: [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) |
| Central configuration: defaults, performance budgets, security policy, device profiles | in place: [`config/`](config/README.md) |
| Documentation landscape in the house format | in place: [`docs/`](docs/README.md) |
| The five protocol engines and the four format crates | **implemented and tested** — the control plane of each protocol; the media path belongs to the platform layer. The Smart View engine implements the discovery and the handshake; the message format of its channel is proprietary and **not** implemented |
| The platform bridge and the local dashboard | **implemented and tested** — the bridge carries a value out and back, the dashboard answers its API |
| The headless receiver: the command line, the configuration file, the device profiles, the systemd unit, the container | **implemented and tested** — the program starts the protocols itself |
| The interface as a remote control: the arrow keys, the visible focus, the card as the switch, the Leanback launcher | **implemented** — the evidence is in [`docs/audit/`](docs/audit/tv-web-dashboard.md) |
| Native shells for Android and macOS | **written**; their Gradle and Xcode builds are a device test — see the acceptance record |
| Hardening, structural gates and the licence gate | **in place** — the gates are green, see [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md) |

**The gate of the workspace is green:** the workspace compiles, the lint is
clean under `-D warnings`, the whole test suite passes, and the crate
boundaries, the file-length limit, the licence headers, the documentation
links and the dependency policy are all checked by scripts that fail when a
rule is broken. The number of the tests and the state of every gate are
machine-readable in [`project/project.yaml`](project/project.yaml). What is
**not** proven is the behaviour on real hardware — the acceptance record names
every such row as NOT RUN and says where it will be measured.

The machine-readable state is
[`project/project.yaml`](project/project.yaml); the narrative of the
milestones is in [`ROADMAP.md`](ROADMAP.md).

The rules of the project are written down before the code exists. A milestone
is not "done" because the picture appears; it is done when its gates are
green and the evidence is in
[`docs/acceptance/`](docs/acceptance/ACCEPTANCE_TEMPLATE.md).

---

## The five protocols

| Protocol | Service | Picture | Sound | Control plane |
|---|---|---|---|---|
| AirPlay 2 | `_airplay._tcp` + `_raop._tcp` | yes | yes | implemented |
| Miracast / Wi-Fi Display | `_miracast._tcp` | yes | yes | implemented |
| Google Cast | `_googlecast._tcp` | yes | yes | implemented |
| DLNA / UPnP MediaRenderer | SSDP | no (audio and pictures) | yes | implemented |
| Samsung Smart View | `_samsungmsf._tcp` + the MultiScreen search | **no — the channel format is proprietary and not implemented** | **no** | discovery and handshake implemented |

"Control plane implemented" means the handshake, the negotiation, the
session lifecycle and the limits — the part of a protocol that a receiver
owns. The *media* path needs a decoder of the host device and belongs to
the native shell; a receiver that cannot decode a codec does not advertise
it.

The fifth protocol is the youngest and the smallest. The Samsung Smart View
engine answers the search of a Galaxy device, serves the description document
and the HTTP API below `/api/v2/`, and accepts the WebSocket upgrade of the
channel. **The content of that channel is not implemented**: the message
format of Smart View is proprietary and not publicly documented, so a Galaxy
device finds the receiver and connects to it, but no picture flows yet. The
exact limit is written down as `EXC-011` in
[`REQUIREMENTS.md`](REQUIREMENTS.md) and in the crate itself
(`engines/smartview`).

The engine of the fifth protocol is registered with the supervisor like every
other engine: the receiver starts it with the others and reports it in the
status. One conflict is written down and not glossed over: the DLNA renderer
and the Smart View responder both want the SSDP port, and in the default
configuration the renderer holds it first — so the fifth protocol does not
reach the advertising state. The run that shows it is quoted in
[`ROADMAP.md`](ROADMAP.md) (M5). The **card of the fifth protocol in the
dashboard is still missing** — the panel of the browser lists four
protocols.

Every protocol can be switched on and off independently, and a disabled
protocol is neither advertised nor bound to a socket.

---

## The interface is a remote control

The dashboard and the native shells are built for a television and for a
remote control, not for a mouse:

- the **arrow keys** move the focus — the group of the receiver cards is one
  tab stop with a *roving tabindex*, and the arrow keys leave the group at
  its edge so that a remote control without a tab key does not come to a
  dead end;
- the **focus is visible** — a drawn frame around the focused surface, with a
  measured contrast against the card and against the page;
- the **card is the switch** — a small switch in the corner of a card is a
  target of a few pixels that nobody hits with a remote control, so the
  card itself is the control and `OK`, `Enter` and `Space` switch the
  protocol;
- the **Android shell is a television application** — it carries the
  `LEANBACK_LAUNCHER` category and a banner, so it appears in the list of
  the television.

The evidence is measured, not estimated: the checks of the dashboard, the
unit tests of the focus order of the Android shell and the test suite of the
macOS kit are in [`docs/audit/`](docs/audit/tv-web-dashboard.md).

---

## The receiver runs headless

`phairplay-web` is a program that runs on a device without a screen — a
Raspberry Pi, a NAS, a container on a server. It does not wait for a click in
a browser:

```bash
# The receiver with the profile of a Raspberry Pi
phairplay-web --profile raspberry-pi

# The receiver with a configuration file of your own
phairplay-web --config /etc/phairplayrust/config.toml

# The receiver without the panel — the API stays available
phairplay-web --no-dashboard

# The complete command line
phairplay-web --help
```

The command line has a help, a version, a configuration file, a device
profile, a port, an interface and a switch for the panel. The **profile** is
the configuration of a class of devices; the profile of the Raspberry Pi
switches Miracast off, because a board without a Wi-Fi Direct radio would
appear in the device list of a sender and fail on every attempt. The profile
files live in [`config/profiles/`](config/profiles/README.md).

The dashboard binds to the loopback interface. A `--bind` to another address
puts the control panel into the network, and the program says so on stderr
before it starts anything.

The installation as a service is in [`deploy/`](deploy/install.sh): a systemd
unit that runs the receiver as its own unprivileged user, and a container
that shares the network of the host, because a bridge network routes neither
the mDNS nor the SSDP multicast.

---

## What PhairPlayRust does **not** do

These are product boundaries, not open defects. They are stated with the same
words in the requirements (`EXC-001` … `EXC-011` in
[`REQUIREMENTS.md`](REQUIREMENTS.md)), in the acceptance records and in the
user-facing documentation.

- **FairPlay DRM content** (Apple TV+, Netflix, Disney+) is **not
  decryptable**. FairPlay Streaming is Apple's DRM; no open-source receiver
  can decrypt it without a licence from Apple. The *session* encryption of
  the AirPlay mirroring path is a different mechanism and is not affected.
- **Apple Music in-app audio** is protected on every AirPlay path. The
  workaround is the system audio output of the sender.
- **HDCP 2.x link protection** of Miracast is negotiated inside the WFD
  handshake and enforced by the hardware. The receiver does not implement the
  key exchange; a session that requires HDCP and that the platform cannot
  protect is refused cleanly.
- **The message format of the Samsung Smart View channel** is proprietary and
  not publicly documented. The discovery and the handshake are implemented;
  the content of the channel is not.
- **Widevine and PlayReady** belong to the DRM framework of the device, not
  to the receiver.
- **No cloud, no remote streaming, no internet exposure.** The receiver is a
  device on your local network.
- **No transcode.** The receiver decodes what it advertised.
- **A Google Cast application identifier is not invented.** The end-to-end
  test of the Cast path needs an identifier that is registered with Google.

The complete list is in the exclusion section of
[`REQUIREMENTS.md`](REQUIREMENTS.md).

---

## Quickstart

You need a stable Rust toolchain (the workspace pins `stable` in
[`rust-toolchain.toml`](rust-toolchain.toml)) and nothing else for the core.

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust

cargo build --workspace      # build every crate
cargo test  --workspace      # run the whole test suite
make check                   # format + lint + test + documentation check
```

The individual commands:

| Command | What it does |
|---|---|
| `make help` | list the developer commands |
| `make build` | `cargo build --workspace` |
| `make test` | `cargo test --workspace` |
| `make fmt` | `cargo fmt --all` |
| `make lint` | `cargo clippy --workspace --all-targets -- -D warnings` |
| `make docs` | `cargo doc --workspace --no-deps` |
| `make docs-links` | check that every internal link of the documentation resolves |
| `make check` | everything above that can fail a change |
| `make clean` | remove the build artefacts |

To see the receiver work on a device without a display:

```bash
cargo run -p phairplay-web -- --profile generic
```

A newcomer who wants the reasoning behind the layout starts with
[`docs/GETTING-STARTED.md`](docs/GETTING-STARTED.md).

---

## If you are new to coding: start here

You do not need to understand the whole codebase before changing one thing.

1. Read this file.
2. Read [`WHERE_TO_CHANGE.md`](WHERE_TO_CHANGE.md) when you want to change
   something specific — it names the right entry point for every kind of
   change.
3. Read [`ROADMAP.md`](ROADMAP.md) to understand what is being built now and
   what comes later.
4. Read [`REQUIREMENTS.md`](REQUIREMENTS.md) before changing behaviour.
5. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) before changing structure,
   dependencies or technical boundaries.
6. Read [`AGENTS.md`](AGENTS.md) if you are using a coding agent.
7. Run `make check` before you hand your change over.

### Golden rule

**Do not "just make it work" by bypassing the architecture, the requirements
or the tests.**

If a requirement is wrong, change the requirement through the documented
change process first. If an architecture rule blocks a correct
implementation, the rule is wrong — and it is changed in
`docs/design/BUILD-CONTRACT.md` first, through a change request.

---

## Repository map

```text
PhairPlayRust/
├─ README.md                  ← Start here
├─ ARCHITECTURE.md            ← Technical system design
├─ ROADMAP.md                 ← Milestones and gates
├─ REQUIREMENTS.md            ← Product requirements (source of truth)
├─ AGENTS.md                  ← Rules and roles for coding agents
├─ PROJECT_MANAGEMENT.md      ← How the project is managed
├─ QUALITY.md                 ← Acceptance and quality model
├─ WHERE_TO_CHANGE.md         ← "I want to change X — where do I go?"
├─ GLOSSARY.md                ← The terms of the protocols
├─ CONTRIBUTING.md            ← Contribution rules / DCO
├─ CHANGELOG.md               ← Human-readable project changes
├─ Makefile                   ← The developer commands
├─ VERSION                    ← The version of the workspace
│
├─ project/project.yaml       ← Machine-readable project state
├─ config/                    ← Central defaults, budgets, policy, profiles
├─ deploy/                    ← The installation as a service and as a container
├─ docs/                      ← Detailed living documentation
├─ scripts/                   ← Developer and validation helpers
│
├─ apps/                      ← Native apps and the dashboard
├─ core/phairplay-core/       ← The shared Rust core
├─ engines/                   ← The protocol state machines
├─ formats/                   ← The wire and media codecs
├─ platform/phairplay-ffi/    ← The UniFFI bridge to the native shells
└─ tests/                     ← Cross-crate and corpus suites
```

---

## Central configuration rule

Defaults, limits and policy must not be scattered through the codebase.

They belong under [`config/`](config/README.md), with a typed model in the
core that corresponds to them:

| File | Owns |
|---|---|
| [`config/app-defaults.toml`](config/app-defaults.toml) | the user-visible default behaviour |
| [`config/performance-budgets.toml`](config/performance-budgets.toml) | the latency, throughput, resource, queue and input limits |
| [`config/security-policy.toml`](config/security-policy.toml) | the threat model, the privacy promise and the parser rules |
| [`config/profiles/`](config/profiles/README.md) | the configuration of a class of devices |

User-specific secrets never belong there. Pairing keys, PINs and
certificates belong into the secure storage of the platform.

---

## Documentation is part of the product

**A code change is not finished if the documentation became wrong.**

Every change asks:

- Does a requirement change?
- Does the architecture change?
- Does a default, a limit or a policy change?
- Does the milestone or the state change?
- Does `WHERE_TO_CHANGE.md` need an update?

If yes, the documentation is updated **in the same change**. The rule and the
mechanics live in [`docs/DOCUMENTATION_POLICY.md`](docs/DOCUMENTATION_POLICY.md).

Every number and every rule is written down **exactly once**. The other
documents link to it. A budget that is written in two places is a budget that
drifts.

---

## Current roadmap

1. **M0 — Foundation** *(done)* — the workspace, the shared core, the build
   contract, the central configuration, the documentation landscape, the gate.
   The acceptance record is in
   [`docs/acceptance/M0-ACCEPTANCE.md`](docs/acceptance/M0-ACCEPTANCE.md).
2. **M1 — AirPlay 2 receiver** *(the code stands; the acceptance record is
   missing)* — the format layer, the discovery, the AirPlay engine
3. **M2 — Protocol breadth and local control** *(the code stands; the
   acceptance record is missing)* — Google Cast, Miracast, the DLNA renderer,
   the platform bridge, the local dashboard
4. **M3 — Native shells** *(the code stands; the evidence is in
   [`docs/audit/`](docs/audit/tv-android-shell.md); the acceptance record is
   missing)* — the Android and the macOS application
5. **M4 — Hardening, interoperability and release readiness** *(largely
   done; the device tests of the interoperability are NOT RUN)*
6. **M5 — Smart View** *(the code stands; the acceptance record is
   missing)* — the fifth protocol

The gates of every milestone are in [`ROADMAP.md`](ROADMAP.md), the quality
model in [`QUALITY.md`](QUALITY.md).

---

## Documentation index

| Document | What it is for |
|---|---|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | The layers, the crate graph, the data flow of every protocol, the threading model, the error and event flow |
| [`REQUIREMENTS.md`](REQUIREMENTS.md) | Every requirement with an identifier, a *How to Verify* and a *Pass Condition* |
| [`ROADMAP.md`](ROADMAP.md) | The milestones, the gates, the state |
| [`QUALITY.md`](QUALITY.md) | The six dimensions of the acceptance model, the severity levels |
| [`AGENTS.md`](AGENTS.md) | The roles, the universal rules, the review matrix, the definition of done |
| [`PROJECT_MANAGEMENT.md`](PROJECT_MANAGEMENT.md) | The sources of truth, the change requests, the ADRs, the archive |
| [`WHERE_TO_CHANGE.md`](WHERE_TO_CHANGE.md) | The navigation for a newcomer |
| [`GLOSSARY.md`](GLOSSARY.md) | AirPlay, RAOP, FairPlay, WFD, SSDP, GENA, SOAP, UPnP, DLNA, SDP, RTP, DACP, UniFFI, Smart View, Leanback and the rest |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | The DCO, the checklist before a change |
| [`CHANGELOG.md`](CHANGELOG.md) | What changed, in the order a human reads it |
| [`docs/README.md`](docs/README.md) | The index of the detailed living documentation |
| [`docs/GETTING-STARTED.md`](docs/GETTING-STARTED.md) | The guide for a newcomer |
| [`docs/BEST_IN_CLASS.md`](docs/BEST_IN_CLASS.md) | The measurable quality grid |
| [`docs/DOCUMENTATION_POLICY.md`](docs/DOCUMENTATION_POLICY.md) | The same-change rule and the single-source rule |
| [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) | The binding interfaces between the crates |
| [`docs/guides/OTHER-DEVICES.md`](docs/guides/OTHER-DEVICES.md) | The guide for the devices that carry the receiver — a Raspberry Pi, a NAS, a container |

The licence inventory of the third-party components is
[`THIRD_PARTY.md`](THIRD_PARTY.md). The user-facing security and privacy
statements are [`SECURITY.md`](SECURITY.md) and [`PRIVACY.md`](PRIVACY.md).

---

## License

PhairPlayRust is licensed under the **AGPL-3.0-or-later**. See
[`LICENSE`](LICENSE) and [`REUSE.toml`](REUSE.toml).

External contributions use the **Developer Certificate of Origin (DCO)** rather
than a CLA. Contributors retain their copyright. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

Third-party dependencies keep their own licenses and notices. The inventory of
the workspace is [`THIRD_PARTY.md`](THIRD_PARTY.md).
