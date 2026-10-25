# Changelog

All notable changes to PhairPlayRust are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [semantic versioning](https://semver.org/) for the
version of the workspace.

The versions of the workspace live in [`VERSION`](VERSION) and in
`Cargo.toml`. The machine-readable state of the project lives in
[`project/project.yaml`](project/project.yaml).

---

## [Unreleased]

### Notes

- The section of the next release is written here. The section of the
  release **v0.3.0** is below.

---

## [0.3.0] — 2026-10-25


### Added

- **The fifth protocol: Samsung Smart View** — the MultiScreen Framework of
  the Galaxy phones and tablets. The engine announces the receiver with the
  search of the framework, serves the description document and the HTTP API
  below `/api/v2/`, answers the handshake of the sender and accepts the
  WebSocket upgrade of the channel. **The message format of the channel is
  proprietary and is not implemented** — the limit is written down as
  `EXC-011` in [`REQUIREMENTS.md`](REQUIREMENTS.md). The crate is
  `engines/smartview`; the vocabulary of the core knows the new kind, its
  service type and its port. **The native shells know the fifth protocol as
  well**: the Android shell and the macOS shell name it in their enumeration
  of the protocols, map it from the kind of the bridge, read its configuration
  from the defaults of the shell and draw it in the grid of the cards. The
  device profiles of [`config/profiles/`](config/profiles/README.md) describe
  it with the port of the framework.
- **The receiver runs headless.** `phairplay-web` starts the protocols that
  the configuration enables at the start of the process, reads its
  configuration from a file, knows the **device profiles** of
  [`config/profiles/`](config/profiles/README.md) and has a complete command
  line (`--config`, `--profile`, `--port`, `--bind`, `--no-dashboard`,
  `--version`, `--help`). A bind address outside the loopback interface is
  announced with a warning before anything is started.
- **The installation as a service and as a container** — a systemd unit that
  runs the receiver as its own unprivileged user, an installer and a compose
  file that shares the network of the host, because a bridge network routes
  neither the mDNS nor the SSDP multicast.
- **The interface as a remote control.** The dashboard and the native shells
  are operated with the arrow keys: the group of the receiver cards is one
  tab stop with a roving tabindex, the focus frame is drawn by the project
  and measured, the card itself is the switch, and the Android shell is a
  real television application with the `LEANBACK_LAUNCHER` category. The
  measured evidence is in [`docs/audit/`](docs/audit/tv-web-dashboard.md).
- **The four receiver engines** (AirPlay 2, Miracast, Google Cast,
  DLNA/UPnP), with the control plane of each protocol and
  the tests that prove it: AirPlay 2 (RTSP, SDP, RTP, HomeKit-style and
  legacy PIN pairing, FairPlay session keys, DACP, DMAP), Miracast (the WFD
  RTSP control plane and the information element), Google Cast (the CastV2
  handshake, the heartbeat, the channel management and the media status) and
  DLNA/UPnP (the description documents, the SOAP actions and GENA).
- **The discovery layer** — DNS-SD and SSDP, with the service types the
  senders actually look for.
- **The four format crates** — the property-list codec (XML and binary), the
  H.264 bitstream, the AAC framing and a native Rust port of the Apple
  Lossless decoder.
- **The platform bridge** — a UniFFI bridge with a round-trip test, and the
  local dashboard with its HTTP API and its WebSocket channel.
- **The native shells** for Android and macOS, and the binding generator.
- **The gates**: the structural checks, the licence audit with a *pinned*
  cargo-deny, the continuous integration, and the acceptance record.

### Changed

- The **dashboard is a television interface** and not a page for a mouse:
  the cards are one tab stop, the arrow keys move the focus, the edge of the
  group leads out of it, and a reading of the API that is older than the last
  change of the user no longer switches a card back.
- The **receiver starts itself**. The start of the process is the start of
  the protocols; the report of the start names every protocol with its state
  and with the reason of a failure.
- The **documentation** was brought back to the state of the code: the
  milestones, the requirements, the architecture, the glossary and the
  machine-readable state of the project name what is delivered and what is
  still NOT RUN.

### Added

- **The channel of a Samsung sender is served under the name that the sender
  gives it.** The client library of Samsung builds the URL of the channel as
  `ws://<host>/api/v2/channels/<name>` — the remote control of a television
  is `samsung.remote.control`. The receiver served a fixed path and was
  therefore **not found** by a sender that names its channel. The receiver
  now serves every name under that prefix, greets the sender with the event
  `ms.channel.connect`, and reads the events `ms.channel.ready`,
  `ms.channel.unauthorized` and the heartbeat `__ping`.
- **The keys of the remote control are understood.** The method
  `ms.remote.control` with the key in the parameter `DataOfCmd` — for example
  `KEY_POWER` — is read on the channel and handed to the platform.
- **The signing of the release of the Android shell.** The build reads the
  properties `phairplay.signing.*` from the machine of the owner of the key;
  the key does not live in the repository, and the build without a key still
  works and produces an APK that is **not** signed.
- **The release build of the Android shell is built by the continuous
  integration.** The debug variant does not build the native bridge for the
  device — and the release variant did, with three defects that the gate of
  the debug variant never saw.

### Fixed

**The exclusion of the message format of the Smart View channel was too
broad.** The messages of the channel are JSON objects with an `event` or a
`method` and a `data` or a `params`, and the names of the events and the
methods are observable on the network. The exclusion `EXC-011` names now
**the transport of the picture of the screen** of a Galaxy device — and
nothing else. A proprietary format is not an unimplementable one: the
Miracast, the Cast and the AirPlay receivers of this project are
implementations of formats that nobody documents either.

**The release build of the Android shell failed on three defects.** The
profile of Cargo was never given to the build, so the bridge of the shell
was always built in the debug profile while the release variant looked for
it in the release profile; the toolchain of the NDK was not in the path, so
a dependency with a C part did not find the archiver of the target; and
there was no way to sign an APK at all. The result is measured:
`BUILD SUCCESSFUL in 12m 39s`, with the library of the bridge, the Leanback
launcher and a signature in the package.

**The two protocols that share the discovery port kept each other out of the
registry.** The DLNA renderer and the Samsung Smart View receiver belong to
the **same** device and therefore carry the **same** unique name — the UDN
names the device, not the service. The registry of the shared SSDP listener
was keyed by that name, so the second service pushed the first one out of
it. Both protocols reported "advertising", and the defect was invisible in
the status: in the network the renderer answered for the framework and the
framework answered for the renderer. The registry is now keyed by the
service (`SsdpService::key`), and the test of it uses the unique name that
the receiver really produces.

**The framework of Samsung did not answer a search for `ssdp:all`.** The
UPnP device architecture says that a query for the wildcard is answered once
per service type of the device. The framework is a service type of this
receiver, so a control point that asks for everything has to see it —
otherwise the receiver is found only by a sender that already knows the name
of the framework.

**The bindings of the bridge were edited by hand instead of being
generated.** The bindings that are checked into the repository carried the
checksum `31533` for the constructor of the receiver; the library that is
built from the sources of the same revision reports `5570`. UniFFI refuses
to start when the two do not match — "UniFFI API checksum mismatch" — so
**every** native shell died at the start, before it could show anything. The
checksum covers the documentation of the function, because the documentation
is part of the metadata: the comment of the constructor was changed in the
Rust source, and the binding was edited by hand instead of being
regenerated. The bindings are generated again, and a new test instantiates
the receiver — which is what actually checks the checksum.

**The version of the workspace was never set.** The release v0.2.0 was
published with the file `VERSION` still carrying `0.1.0`, because nothing
compared the three places that carry the version. The new gate
`scripts/check-version-consistency.sh` compares them, and it runs in the
continuous integration and in the gate of the project.

The native shells did not compile against the bridge of the current core. The
bridge requires the byte budget of the audio queue since the limits of the
process were extended, and neither shell handed it over — the record of the
limits was built from a data structure without that field. **Both shells are
broken at the commit that the shells were last touched**, and the continuous
integration did not notice it, because the gate builds the workspace of Rust
and not the shells. The shells now hand the limit over, and the profiles of
the device describe the fifth protocol, so that a profile does not fall back
to the default of the core for a protocol that its owner reviewed.

The adversarial review of the first implementation found real defects. Each
of them now has a test that fails without the fix:

- A lost packet in the TAP-A path of the ALAC decoder.
- A panic in the Golomb coder of the ALAC decoder on 33-bit samples.
- An overflow in the shift of an ALAC element, and a stale shift buffer in
  the escape path — a *remote* panic, and a wrong picture without an error.
- An index out of bounds in the stereo path of the ALAC decoder.
- A lost frame in the AirPlay engine, and a collision of two tests on one
  port.

---

## [0.1.0] — 2026-10-24

The first baseline. It contains the foundation of the receiver: the
workspace, the shared core, the build contract and the documentation
landscape. **No protocol is implemented in this release** — see the status
table in [`README.md`](README.md).

### Added

- **The Cargo workspace** with twelve members: the shared core
  (`core/phairplay-core`), five engine crates (`engines/{discovery,airplay,
  miracast,cast,dlna}`), four format crates (`formats/{plist,h264,aac,
  alac}`) and the platform bridge (`platform/phairplay-ffi`).
- **The shared core** with the protocol abstraction
  (`protocol::ProtocolEngine`, `ProtocolKind`, `ProtocolState`,
  `ProtocolStatus`, `EngineContext`), the session model
  (`SessionEvent`, `SessionRegistry`, `SessionSnapshot`), the media
  abstractions (`VideoSink`, `AudioSink`, `EncodedVideoFrame`,
  `EncodedAudioFrame`), the capability model (`MediaCapabilities`,
  `VideoCodec`, `AudioCodec`, `PassthroughCodec`), the typed configuration
  (`AppConfig`, `DeviceConfig`, `ProtocolConfig`, `LimitsConfig`,
  `PerformanceConfig`), the advertised identity (`DeviceIdentity`,
  `sanitize_service_name`), the session counters (`StreamCounters`,
  `StreamStats`), the closed error hierarchy (`CoreError`, `ErrorKind`,
  `ErrorCode`) and the bounded job executor (`JobExecutor`, `JobRegistry`).
- **The build contract** `docs/design/BUILD-CONTRACT.md`: the fixed
  interfaces between the crates, the rules for the engines, the rules for the
  format crates, the testing contract and the code style.
- **The central configuration** under `config/`: the application defaults, the
  performance budgets and the security policy, with the typed mirror in the
  core.
- **The documentation landscape** in the house format: the root documents
  (`README`, `ARCHITECTURE`, `ROADMAP`, `REQUIREMENTS`, `AGENTS`,
  `PROJECT_MANAGEMENT`, `QUALITY`, `WHERE_TO_CHANGE`, `GLOSSARY`,
  `CONTRIBUTING`, `CHANGELOG`), the `docs/` tree with the policy, the
  best-in-class framework, the newcomer guide, the templates for the
  acceptance records, the ADRs and the change requests, and the guides.
- **The requirements** with sequential identifiers, a *How to Verify* and a
  *Pass Condition* per requirement, the non-functional requirements of the
  latency, the frame rate, the memory, the CPU, the security, the privacy
  and the interoperability, and the explicit exclusion list of the product
  boundaries.
- **The repository check** `scripts/check-docs-links.sh`: it verifies that
  every internal link of the documentation resolves and that the documented
  budgets agree with the typed model of the core.
- **The licence metadata**: the AGPL-3.0-or-later text, the licences of the
  third-party components and the `REUSE.toml` that carries the copyright and
  the license information of the tree.

### Notes

- The four protocol engines, the four format crates and the platform bridge
  are **stubs** in this baseline. A stub never satisfies an acceptance; the
  implementation is tracked in [`ROADMAP.md`](ROADMAP.md) (M1 to M4).
- The product boundaries are documented from the first line: FairPlay DRM is
  not decryptable, the in-app audio of the Apple Music app is protected, a
  Miracast session needs the WFD permission of the platform, and the
  receiver never talks to anything but the local network.

[Unreleased]: https://github.com/mazer666/PhairPlayRust/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/mazer666/PhairPlayRust/releases/tag/v0.3.0
[0.2.0]: https://github.com/mazer666/PhairPlayRust/releases/tag/v0.2.0
[0.1.0]: https://github.com/mazer666/PhairPlayRust/releases/tag/v0.1.0
