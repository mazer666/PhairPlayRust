# PhairPlayRust for newcomers — a guide

This document answers the questions that come up when you start working on
the project. It is deliberately short and points to the places where the
answers are maintained.

**If you want to change something, read
[`../WHERE_TO_CHANGE.md`](../WHERE_TO_CHANGE.md) first.** It names the right
entry point for every kind of change.

---

## 1. What is this project?

PhairPlayRust is a receiver for the local network. It speaks five protocols —
AirPlay 2, Miracast, Google Cast, DLNA/UPnP and Samsung Smart View — and it
does that in one shared Rust core, so that the native shells and the dashboard
all see the same state.

Three promises shape every decision:

1. **No account, no telemetry, no advertisement, no cloud.** The receiver
   only ever talks to the peers on your local network.
2. **Honest limits.** FairPlay DRM is not decryptable, the in-app audio of
   the Apple Music app is protected, a Miracast session needs the WFD
   permission of the platform, and the message format of the Smart View
   channel is proprietary. These are written down in the requirements
   (`EXC-001` … `EXC-011`) and in the user-facing documentation.
3. **Documentation is part of the product.** A change that makes a document
   wrong is an unfinished change.

Two more rules shape the way the product is used:

4. **The interface is a remote control.** The dashboard and the native shells
   are operated with the arrow keys; the focus is visible; the card of a
   protocol is the switch.
5. **The receiver runs headless.** The program starts the protocols itself,
   reads its configuration from a file and needs no screen.

---

## 2. What do I need?

For the core and for the engines:

- a stable Rust toolchain — the workspace pins `stable` in
  [`../rust-toolchain.toml`](../rust-toolchain.toml)
- `cargo`, `rustfmt` and `clippy` (the toolchain profile installs them)

For the native shells, additionally the platform toolchains: a JDK and the
Android SDK for Android, Xcode for macOS. The core needs neither.

You do **not** need an Apple device, a television or a special network to
work on the core and on the engines. The tests run on a laptop.

---

## 3. How is the project structured?

```text
apps/android   apps/macos   apps/web
Kotlin/Compose SwiftUI      Rust backend + static assets
      │            │               │
      └──── UniFFI ┘               │ HTTP/WebSocket, loopback only
               │                  │
        platform/phairplay-ffi ──┘
               │
        core/phairplay-core        the shared vocabulary
               ▲
               │
        engines/{airplay,miracast,cast,dlna,smartview,discovery}
               ▲
               │
        formats/{plist,h264,aac,alac}
```

Five ground rules that are never broken:

1. **The shell never talks to an engine.** Everything goes through the bridge
   in `platform/phairplay-ffi` or through the local web backend.
2. **The core is platform independent.** There is no `#[cfg(target_os = …)]`
   in the core.
3. **An engine never decodes.** It delivers encoded frames; the platform
   decodes.
4. **A format crate is pure.** Bytes in, bytes out. No I/O, no socket, no
   platform call.
5. **The web application never talks to an engine either.** It talks to the
   supervisor of the bridge, which owns the engines. The one place that
   builds the engines is the composition root,
   `platform/phairplay-ffi/src/composition.rs`.

The full description, including the crate graph and the data flow of every
protocol, is in [`../ARCHITECTURE.md`](../ARCHITECTURE.md).

---

## 4. How do I build and test?

```bash
# Build every crate
cargo build --workspace

# Run the whole test suite
cargo test --workspace

# Run the tests of one crate
cargo test -p phairplay-core
cargo test -p phairplay-airplay

# The strict lint of the workspace
cargo clippy --workspace --all-targets -- -D warnings

# The format check
cargo fmt --all --check

# The API documentation
cargo doc --workspace --no-deps
```

The shortcuts live in the `Makefile`:

| Command | What it does |
|---|---|
| `make build` | build every crate |
| `make test` | run the whole test suite |
| `make fmt` | format the sources |
| `make lint` | the strict lint |
| `make docs` | build the API documentation |
| `make docs-links` | check that every internal documentation link resolves |
| `make check` | everything that a change must pass |
| `make clean` | remove the build artefacts |

`make check` is the command to run before you hand a change over.

---

## 5. How do I run the receiver on a device without a screen?

The receiver is a program, not a window. On a Raspberry Pi, on a NAS or in a
container there is nobody who opens a browser, so the program starts the
protocols itself:

```bash
# The receiver with the profile of a Raspberry Pi
cargo run -p phairplay-web -- --profile raspberry-pi

# The receiver with a configuration file of your own
cargo run -p phairplay-web -- --config /etc/phairplayrust/config.toml

# The receiver without the panel — the API stays available
cargo run -p phairplay-web -- --no-dashboard

# The complete command line
cargo run -p phairplay-web -- --help
```

The program prints what it does, one line per protocol: the ones that run and
the ones that do not, with the reason. A protocol that cannot start — a
container without multicast, a board without a Wi-Fi Direct radio — does not
stop the other protocols.

A **device profile** is the configuration of a class of devices. The profiles
live in [`../config/profiles/`](../config/profiles/README.md): a
single-board computer, a NAS, a container, a desktop, and the neutral
fallback for a device nobody described. The name of a profile is the name of
its file, and a profile is a complete configuration that `--config` accepts
unchanged.

The installation as a service is in [`../deploy/`](../deploy/install.sh): a
systemd unit that runs the receiver as its own unprivileged user, and a
container that shares the network of the host — a bridge network routes
neither the mDNS nor the SSDP multicast, so a sender would find nothing.

The guide for the devices and the installation step by step is in
[`guides/OTHER-DEVICES.md`](guides/OTHER-DEVICES.en.md).

---

## 6. How do I operate the interface with a remote control?

The dashboard and the native shells are built for a television:

| Key | What it does |
|---|---|
| the arrow keys | move the focus inside the group of the cards and leave the group at its edge |
| `OK`, `Enter`, `Space` | switch the protocol of the card that holds the focus |
| `Home`, `End` | jump to the first or the last card of the group |
| `Tab` | leave the group and reach the next control of the page |
| `Escape` | close the banner of a problem |

The card of a protocol **is** the switch — there is no smaller control to hit
first. The focus frame is drawn by the project itself and its contrast is
measured; the evidence is in
[`audit/tv-web-dashboard.md`](audit/tv-web-dashboard.md),
[`audit/tv-android-shell.md`](audit/tv-android-shell.md) and
[`audit/tv-macos-shell.md`](audit/tv-macos-shell.md).

The Android shell is a real television application: it carries the
`LEANBACK_LAUNCHER` category and a banner, so it appears in the list of the
television.

---

## 7. How do I see what the workspace depends on?

```bash
# The tree of the dependencies
cargo tree --workspace

# The metadata of every member
cargo metadata --no-deps --format-version 1

# The dependencies of one crate
cargo tree -p phairplay-airplay
```

The direction of the dependency is strict and is described in
[`../ARCHITECTURE.md`](../ARCHITECTURE.md) §2. A `formats/*` crate must not
depend on an `engines/*` crate; an `engines/*` crate must not depend on a
`platform/*` or an `apps/*` crate; `core/*` must not depend on any other
crate of the workspace.

---

## 8. Where do the limits and the defaults live?

In three files under [`../config/`](../config/README.md):

| File | Owns |
|---|---|
| `app-defaults.toml` | the advertised identity, the protocol switches, the session rules, the dashboard bind address |
| `performance-budgets.toml` | the latency, the frame rate, the memory, the CPU, the queue budgets, the input limits |
| `security-policy.toml` | the threat model, the privacy promise, the parser rules, the callback guard, the pairing policy |
| `profiles/` | the configuration of a class of devices: a single-board computer, a NAS, a container, a desktop, a device nobody described |

The typed mirror lives in `core/phairplay-core/src/config.rs`. A change
touches both in the same change.

An engine reads its limits from the configuration model — it never hard-codes
a message size, a header count or a queue size.

---

## 9. What does the receiver do with a frame?

```text
sender ──RTP/HTTP──► engine ──encoded frame──► sink ──► decoder ──► screen
                        │
                        └─ the engine never decodes
```

The sink is bounded twice: by an item count and by a byte budget. A sink
that cannot keep up returns `MediaError::QueueFull` instead of growing. A
stalled decoder must never become an out-of-memory kill of the whole
receiver.

---

## 10. Where do the errors come from?

One hierarchy in the core, `core/phairplay-core/src/error.rs`:

```text
CoreError
├── Config      NotFound · Parse · Invalid{field,reason} · Io
├── Engine      Start · NotRunning · AlreadyRunning · Protocol · LimitExceeded · Stopped
├── Media       QueueFull · Closed · Rejected · Unsupported
├── Session     NoActiveSession · Duplicate · AlreadyActive · NotFound · Ended
├── Network     Bind · Io · NoInterface · PeerOutsideSubnet
├── Capability  Query · Inconsistent
└── Aborted
```

Every error is classified into an `ErrorKind` and mapped onto a numeric
`ErrorCode`. The numeric code is the part that crosses the bridge, and it is
part of the public contract of the bridge: it is not renumbered without a
major version bump. The shells branch on the code, never on the text of a
message.

---

## 11. How do I watch the traffic?

The receiver is a network service, and the fastest way to understand a
sender is to watch what it sends. On a machine with the usual tools
installed:

```bash
# Everything that arrives on the interface of the receiver
sudo tcpdump -i any -n -A
```

The filters for the discovery traffic and for the RTSP control plane — and
the ports of the five protocols that they use — are in
[`guides/PROTOCOL_OVERVIEW.md`](guides/PROTOCOL_OVERVIEW.md) §7. That guide
is the single place where the ports are written down. The port of the API of
the Samsung Smart View receiver is in
[`../core/phairplay-core/src/config/ports.rs`](../core/phairplay-core/src/config/ports.rs),
which is the owner of every port of the workspace.

A sender that finds no receiver usually means one of three things:

1. the multicast of the network is blocked (the router filters it),
2. the service is advertised under the wrong name — AirPlay needs **both**
   `_airplay._tcp` and `_raop._tcp`,
3. the receiver and the sender are in different subnets.

---

## 12. How do I know that a test ran for real?

A test that cannot fail is not a test. The rules of the build contract:

1. **Every public function has a unit test.**
2. **Every hardening measure has a negative test** that really runs the
   attack: a property list with a cycle, the SPS overflow vector of
   `FR-056`, a SOAP body with a `DOCTYPE`, a GENA callback that points at
   `169.254.169.254`.
3. **No test is skipped, ignored or commented out** to make a build green.
4. **No stub returns a fake success.** A function that returns a fixed value
   is a lie that the acceptance process catches.

---

## 13. How do I change the behaviour of the product?

1. Find the requirement in [`../REQUIREMENTS.md`](../REQUIREMENTS.md).
2. If the behaviour really changes, write a change request — the template is
   [`cr/CR_TEMPLATE.md`](cr/CR_TEMPLATE.md).
3. Update the gate of the milestone in [`../ROADMAP.md`](../ROADMAP.md).
4. Then implement, with the test and the documentation in the same change.

---

## 14. How do I add a protocol, a codec or a setting?

| I want to add | The steps |
|---|---|
| A protocol | A new crate in `engines/`, an entry in `ProtocolKind`, a service type in the discovery engine and in the configuration, the registration in the composition root, an ADR, a requirement, a milestone slot, a device test plan row |
| A codec | A new crate in `formats/` or an addition to an existing one, an entry in `VideoCodec` or `AudioCodec`, the platform capability that reports the decoder |
| A setting | The TOML file in `config/` **and** the typed model in the core, in the same change |
| A device class | A profile in `config/profiles/` and an entry in `DeviceProfile` — the values belong in the file, the name belongs in the core |

The details are in [`../WHERE_TO_CHANGE.md`](../WHERE_TO_CHANGE.md).

The fifth protocol — Samsung Smart View — is the proof that the first row
works: a crate in `engines/`, an entry in `ProtocolKind`, a service type and
a port in the configuration. The core needed no new module.

---

## 15. Which documents must I not forget?

Before you hand a change over:

- [ ] The requirement and the architecture that the change touches are
      updated, if the change touches them.
- [ ] The configuration is updated, if a default, a limit or a budget changed.
- [ ] The tests are there, including the negative test of a new hardening
      measure.
- [ ] `make check` is green.
- [ ] The known limitations of the change are named.

The full list is in [`../CONTRIBUTING.md`](../CONTRIBUTING.md).

---

## 16. Where are the risks and the open questions?

The significant risks are maintained in the risk register under
`docs/project/`. The risks that dominate today come from the threat model: a
hostile device in the local network, a sender that sends a flood, a parser
that misreads a length field. Their mitigations are the limits of the
configuration, the negative tests and the bounded queues.

---

## 17. The terms

Every protocol term — AirPlay, RAOP, FairPlay, WFD, Miracast, SSDP, GENA,
SOAP, UPnP, DLNA, SDP, RTP, RTCP, NTP, DACP, DMAP, EDID, HDCP, ALAC,
AAC-ELD, HLS, UniFFI, Tokio, mDNS, DNS-SD, SRP, Ed25519, X25519 — is
explained in [`../GLOSSARY.md`](../GLOSSARY.md).

The terms of this round are there too: **Samsung Smart View**, **MSF**, the
**service type**, **Leanback**, the **D-pad**, the **roving tabindex**, the
**focus frame**, the **headless** build, the **device profile**, the
**composition root** and the **supervisor**.
