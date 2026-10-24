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

### Added

- **The four protocol engines**, with the control plane of each protocol and
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

### Fixed

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

[Unreleased]: https://github.com/mazer666/PhairPlayRust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mazer666/PhairPlayRust/releases/tag/v0.1.0
