# Where to Change What

This file is written for someone who does **not** know the whole codebase.

It is deliberately written for someone who does **not** know the whole
codebase.

If you want to change something, find the section below. If your change is
not in this file, the file is incomplete — add the entry in the same change.

A path that carries a milestone in brackets does not exist yet; it is the
place that the named milestone fills. Every other path exists today.

---

## I want to change a default setting

Go to:

- [`config/app-defaults.toml`](config/app-defaults.toml),
  [`config/performance-budgets.toml`](config/performance-budgets.toml) or
  [`config/security-policy.toml`](config/security-policy.toml)
- the typed model in `core/phairplay-core/src/config.rs`

Do not hard-code a new default in a shell, in an engine or in a dashboard.

A change to a value touches the TOML file **and** the typed model in the same
change. The TOML file is the documented source; the Rust constant is the
typed mirror.

---

## I want to change the product behaviour

1. Find the requirement in [`REQUIREMENTS.md`](REQUIREMENTS.md).
2. If the behaviour really changes, write a change request in `docs/cr/` —
   the template is [`docs/cr/CR_TEMPLATE.md`](docs/cr/CR_TEMPLATE.md).
3. Update the acceptance criteria of the milestone in
   [`ROADMAP.md`](ROADMAP.md).
4. Then implement.

---

## I want to change the architecture or a boundary

1. Read the relevant section of [`ARCHITECTURE.md`](ARCHITECTURE.md).
2. If the change touches an interface between two crates, write an ADR in
   `docs/adr/` — the template is
   [`docs/adr/ADR_TEMPLATE.md`](docs/adr/ADR_TEMPLATE.md).
3. If the change touches the build contract, change
   [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) **first**,
   through a change request.

The direction of the dependency is strict:

```text
apps/*  →  platform/*  →  core/*
engines/*  →  core/*, formats/*
formats/*  →  (nothing of the workspace)
core/*  →  (nothing of the workspace)
```

---

## I want to work on the AirPlay 2 receiver

Read the M1 section of [`ROADMAP.md`](ROADMAP.md) first.

| Area | Where |
|---|---|
| The RTSP reader and the answer builder | `engines/airplay/src/` |
| The SDP parser | `engines/airplay/src/` |
| The RTP reassembly | `engines/airplay/src/` and `formats/h264/src/` |
| The pairing procedures | `engines/airplay/src/` |
| The FairPlay session key | `engines/airplay/src/` |
| The reverse remote, the metadata, the timing, the photos | `engines/airplay/src/` |
| The property list codec | `formats/plist/src/` |
| The AAC and the ALAC | `formats/aac/src/`, `formats/alac/src/` |

The state machine of the engine, the single-peer rule and the events are
described in [`ARCHITECTURE.md`](ARCHITECTURE.md) §4, §5 and §8.

The pairing material, the session key and the SRP procedure use the
cryptography crates of the workspace dependency table. The ALAC port carries
the attribution of the Apple reference decoder.

---

## I want to work on the Google Cast receiver

| Area | Where |
|---|---|
| The CastV2 framing, the heartbeat, the application identifier | `engines/cast/src/` |
| The media channels and the media receiver status | `engines/cast/src/` |
| The advertisement of the service | `engines/discovery/src/` |

Cast is the documented exception to the one-peer rule. The exception lives in
one place: the `session.multi_peer_protocols` list in
[`config/app-defaults.toml`](config/app-defaults.toml).

---

## I want to work on the Miracast receiver

| Area | Where |
|---|---|
| The WFD control plane and the information elements | `engines/miracast/src/` |
| The baseline codecs and the advertisement | `engines/miracast/src/` |
| The H.264 parsing | `formats/h264/src/` |

Miracast needs the WFD permission of the platform. A device without it does
not advertise the service — the receiver does not work around a missing
system permission.

---

## I want to work on the DLNA media renderer

| Area | Where |
|---|---|
| The SSDP answer and the notification | `engines/dlna/src/` |
| The device and the service descriptions | `engines/dlna/src/` |
| The SOAP dispatcher | `engines/dlna/src/` |
| The GENA events and the callback guard | `engines/dlna/src/` |

The SOAP parser follows the XML rules of
[`config/security-policy.toml`](config/security-policy.toml). The callback URL
is checked against the guard in the same file.

---

## I want to work on the Samsung Smart View receiver

| Area | Where |
|---|---|
| The answer to the search of the MultiScreen Framework | `engines/smartview/src/ssdp.rs` |
| The description document that the `LOCATION` header points at | `engines/smartview/src/description.rs` |
| The HTTP API below `/api/v2/` and the handshake of the sender | `engines/smartview/src/api.rs` |
| The WebSocket upgrade of the channel | `engines/smartview/src/api/channel.rs` |
| The single-session rule | `engines/smartview/src/session.rs` |
| The state machine of the engine | `engines/smartview/src/engine.rs` |
| The service type and the port of the API | `core/phairplay-core/src/config/` |

**The honest limit:** the message format of the Smart View channel is
proprietary and **not implemented** (`EXC-011`). The crate implements the
discovery and the handshake — the part that makes a device a Smart View
target. Nothing in this crate may claim that a picture flows.

---

## I want to change the discovery

| Area | Where |
|---|---|
| The DNS-SD advertisement | `engines/discovery/src/` |
| The service types of the protocols | `core/phairplay-core/src/protocol.rs` |
| The sanitising of the instance name | `core/phairplay-core/src/identity.rs` |

The service type of a protocol is defined in exactly one place. The AirPlay
engine needs **two** service types: `_airplay._tcp` and `_raop._tcp`.

---

## I want to change the shared core

| Area | Where |
|---|---|
| The protocol trait and the engine context | `core/phairplay-core/src/protocol.rs` |
| The session model and the registry | `core/phairplay-core/src/session.rs` |
| The media sinks and the frame types | `core/phairplay-core/src/media.rs` |
| The capability model | `core/phairplay-core/src/capabilities.rs` |
| The configuration model | `core/phairplay-core/src/config.rs` |
| The identity | `core/phairplay-core/src/identity.rs` |
| The counters | `core/phairplay-core/src/metrics.rs` |
| The error hierarchy and the numeric codes | `core/phairplay-core/src/error.rs` |
| The job executor | `core/phairplay-core/src/jobs/` |

A signature of the core is part of the contract of the workspace. Changing it
requires a change request and a rebuild of everything that uses it.

---

## I want to change the bridge or the dashboard backend

| Area | Where |
|---|---|
| The exported surface and the data transfer objects | `platform/phairplay-ffi/src/` |
| The error mapping onto the numeric codes | `platform/phairplay-ffi/src/` |
| The generation of the bindings | `scripts/generate-bindings.sh` (delivered with M3) |
| The HTTP and the WebSocket backend | `apps/web/src/` (M2) |
| The assets of the dashboard | `apps/web/assets/` (M2) |
| The command line of the receiver | `apps/web/src/cli.rs` |
| The composition root: the engines, the start, the report | `apps/web/src/receiver.rs` |
| The lookup of the device profiles | `apps/web/src/profiles.rs` |

Rules:

- the exported surface is platform-neutral; a platform difference is a
  different implementation, never a new exported function
- the generated bindings are never edited by hand
- the web backend binds on the loopback interface only
- **the web application talks to the supervisor, never to an engine.** The
  engines are registered once, in the composition root; a second
  registration would be a second truth about what runs

---

## I want to run the receiver on a device without a screen

| Area | Where |
|---|---|
| The profiles of the classes of devices | `config/profiles/` |
| The names of the profiles | `core/phairplay-core/src/config/profiles.rs` |
| The installation as a service | `deploy/install.sh`, `deploy/systemd/` |
| The container | `deploy/docker/` |
| The guide for the devices | `docs/guides/OTHER-DEVICES.md` |

The values of a profile live in the file, not in the code: a decision that a
user can read, change and compare belongs in a file. The name of a profile
is a word of the product and lives in the core.

---

## I want to change a native shell

| Area | Where |
|---|---|
| The Android application | `apps/android/` (M3) |
| The macOS application | `apps/macos/` (M3) |

Product logic normally does **not** belong in a shell. The shell shows the
state and it switches the protocols; the logic lives in the core.

---

## I want to change the remote control interface

| Area | Where |
|---|---|
| The roving tabindex and the movement of the focus in the dashboard | `apps/web/assets/focus-group.js` |
| The cards and the switch of a protocol | `apps/web/assets/protocol-cards.js` |
| The frame of the focus and the ten-foot scale | `apps/web/assets/cards.css`, `apps/web/assets/theme.css` |
| The order of the focus of the Android shell | `apps/android/app/src/main/java/org/phairplay/receiver/ui/TvFocus.kt` |
| The frame of the focus of the Android shell | `apps/android/app/src/main/java/org/phairplay/receiver/ui/TvFocusFrame.kt` |
| The Leanback entry of the Android manifest | `apps/android/app/src/main/AndroidManifest.xml` |
| The keys and the focus of the macOS shell | `apps/macos/Sources/PhairPlayMacKit/ShellKeyboard.swift`, `ShellFocus.swift` |
| The checks that prove the operation | `apps/web/tools/check-keyboard.mjs`, `check-layout.mjs`, `check-contrast.mjs` |

The rules of this surface: the card of a protocol is the switch, the focus is
drawn by the project and measured, the state is carried by a word and a shape
and not by a colour alone, and the group of the cards leaves itself at its
edge — a remote control has no tab key.

The evidence of a change is in [`docs/audit/`](docs/audit/tv-web-dashboard.md).

---

## I want to change how the background work is scheduled

| Area | Where |
|---|---|
| The executor and the registry of the jobs | `core/phairplay-core/src/jobs/` |
| The worker count and the queue length | `config/performance-budgets.toml`, section `[jobs]` |

A new job kind needs a real operation behind it. A job that does not exist
reports a failure; it never reports a success without doing the work.

---

## I want to add a protocol

1. A new crate in `engines/`.
2. A new entry in `ProtocolKind` in
   `core/phairplay-core/src/protocol.rs`.
3. A new service type in the discovery engine and in
   `config/app-defaults.toml`.
4. The registration of the engine in the composition root,
   `apps/web/src/receiver.rs`.
5. **The protocol in every shell** — the shell that does not know the protocol
   leaves its card out of the grid:
   - the enumeration of the protocols of the Android shell,
     `apps/android/app/src/main/java/org/phairplay/receiver/model/ProtocolId.kt`,
     the mapping of the kind of the bridge in `ReceiverMapping.kt`, the label
     in `ui/StateLabels.kt` and the string in `res/values/strings.xml` (and in
     every translation of it),
   - the enumeration of the protocols of the macOS shell,
     `apps/macos/Sources/PhairPlayMacKit/ReceiverModel.swift`, the mapping in
     `ReceiverMapping.swift` and the string in the catalogues of
     `Resources/*.lproj/Localizable.strings`,
   - the list of the protocols in the defaults of the shell,
     `receiver-defaults.properties` of both shells, and the entry in the
     profiles of the device, `config/profiles/*.toml`,
   - the dashboard, `apps/web/assets/api.js` and the catalogues of
     `apps/web/assets/lang/`.
6. An ADR, because a protocol is an architectural addition.
7. A requirement, a milestone slot and a device test plan entry.
8. A change request in `docs/cr/` — the fifth protocol is the proof of the
   shape of such a request:
   [`docs/cr/CR-0001-samsung-smart-view.md`](docs/cr/CR-0001-samsung-smart-view.md).

**The continuous integration builds the workspace of Rust and not the shells.**
A change in a shell is therefore proved by the build of the shell itself —
`./gradlew :app:testDebugUnitTest` in `apps/android` and `swift test` in
`apps/macos` — and not by the gate of the workspace.

The core does not change. If a new protocol forces a change in the core, the
protocol abstraction is wrong — that is a finding for the architect, not a
reason to bend the core.

---

## I want to add a codec

1. A new crate in `formats/` or an addition to an existing one.
2. A new entry in `VideoCodec` or `AudioCodec` in
   `core/phairplay-core/src/capabilities.rs`.
3. The platform capability that reports the decoder.

A format crate stays pure: no I/O, no socket, no platform call.

---

## I want to add network access

Stop.

The receiver talks to the local network and to nothing else. A new outbound
connection needs:

- a requirement that says why
- a review by the security and by the privacy role
- an entry in the outbound policy of
  [`config/security-policy.toml`](config/security-policy.toml)
- a negative test for the address that must not be reachable

---

## I want to add a dependency

Before adding:

1. explain why the existing code cannot solve it
2. check the license compatibility with the AGPL
3. check the maintenance and the security posture
4. check the cost in binary size and on the platform
5. record the third-party information in the licence inventory of the project

---

## I want to add a setting

Ask:

- Is this really a user-facing setting, or should the automatic behaviour
  just work?
- Which section of the configuration owns it?
- What is the default?
- Is it sensitive? A secret never belongs into the repository configuration.

Then update the configuration file and the typed model together.

---

## I want to change a performance budget

1. Write a change request with the measurement that shows why the old budget
   was wrong.
2. Change the value in
   [`config/performance-budgets.toml`](config/performance-budgets.toml) and in
   the typed model.
3. Update the requirement that refers to the key, if the semantics change.

A budget is never lowered to make a test green.

---

## I want to change the documentation

The current documents live at the top level and under `docs/`. The historical
baselines live under `archive/` and are never edited.

Rules:

- a number exists in exactly one place; the other documents link to it
- every internal link must resolve:
  `bash scripts/check-docs-links.sh`
- a change of the behaviour changes the documentation in the same change
