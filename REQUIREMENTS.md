# PhairPlayRust Product Requirements

**Requirements baseline:** 0.2
**Status:** Product baseline accepted
**Rule:** Requirements are the source of truth for behaviour. The code does not silently redefine them.

Each requirement carries an identifier, a *How to Verify* and a *Pass
Condition*. A requirement without a verification is a wish; a requirement
without a pass condition cannot be failed.

Implementation status is **not** tracked here. It lives in
[`ROADMAP.md`](ROADMAP.md) (milestone gates) and in
[`project/project.yaml`](project/project.yaml) (machine-readable state).

## How to read a requirement

Every requirement has the same four parts:

```text
### FR-0NN — The title of the requirement
The statement. What the receiver does or does not do, and under which
condition.

**How to Verify:** The command, the test or the inspection that produces the
evidence.
**Pass Condition:** The result that makes the requirement pass — and, by
implication, the result that makes it fail.
```

The identifier series:

| Series | Meaning |
|---|---|
| `FR-NNN` | A functional requirement. The number is consecutive and never reused. |
| `NFR-<AREA>-NNN` | A non-functional requirement: `PERF`, `RES`, `SEC`, `PRIV`, `INT`, `REL`, `DOC`. |
| `EXC-NNN` | An explicit exclusion. A product boundary, not an open defect. |

A requirement that cannot be verified is a wish. A requirement without a
pass condition cannot be failed.

## How the numbers are owned

The numeric budgets of the non-functional requirements are **not** repeated
here. They are owned by
[`config/performance-budgets.toml`](config/performance-budgets.toml), and every
NFR below names the key that carries its budget. A budget that is written in
two places is a budget that drifts. The same rule applies to the default
behaviour ([`config/app-defaults.toml`](config/app-defaults.toml)) and to the
security policy ([`config/security-policy.toml`](config/security-policy.toml)).

---

# 1. Product principles and the privacy promise

### FR-001 — The receiver is free and open source
PhairPlayRust is distributed under the AGPL-3.0-or-later licence. Every crate
of the workspace inherits `license.workspace = true`; no crate carries a
literal licence string of its own.

**How to Verify:** `cargo metadata --no-deps --format-version 1` lists the
licence of every workspace member, and `grep -rn "license" --include=Cargo.toml
core engines formats platform` finds no literal licence in a member manifest.
**Pass Condition:** Every workspace member reports `AGPL-3.0-or-later`, and no
member manifest contains a literal licence identifier.

### FR-002 — No account, ever
The receiver has no account, no login, no sign-up and no "first run" gate. The
receiver starts advertising as soon as it is started.

**How to Verify:** Search the source directories for account, sign-in, token
and credential flows (`grep -rniE "sign[ -]?in|sign[ -]?up|log[ -]?in|account"
core engines formats platform apps`). Inspect the start-up path of the shells
in `apps/android` and `apps/macos`.
**Pass Condition:** No authentication flow exists in the codebase, and the
receiver reaches the advertising state without any user input beyond the
start command.

### FR-003 — No telemetry, no analytics, no crash upload
The receiver sends nothing to any host except the peers on the local network
that it is receiving from. There is no heartbeat, no usage report, no crash
upload and no update check.

**How to Verify:** `grep -rniE "telemetry|analytics|sentry|crashlytics|mixpanel|amplitude|segment\.io|google-analytics" core engines formats platform apps`
returns no hit. The documentation names the promise; it is not a production
hit. The outbound-socket policy in
[`config/security-policy.toml`](config/security-policy.toml) is
`allow_outbound_connections_except_local_subnet = false`.
**Pass Condition:** No telemetry client, endpoint or identifier exists in the
source, and the only outbound connections the receiver opens are to peers of
the local subnet.

### FR-004 — No advertising, no upsell, no nagging
The interface shows the state of the receiver and the state of the sessions.
It never shows an advertisement, a promotional banner, a tip of the day, a
badge, or a request to rate the app.

**How to Verify:** Inspect the status surfaces in `apps/android`,
`apps/macos` and the dashboard assets in `apps/web`. Compare the strings
against the exclusion list of the acceptance template.
**Pass Condition:** No user-visible surface contains promotional text, and
there is no mechanism that reports a rating prompt or a "tip" state.

### FR-005 — Local network only
The receiver listens on the local network and accepts peers from the local
subnet. It never connects to a host outside the local subnet, and it is never
published to the internet by the product.

**How to Verify:** Inspect the bind and connect paths of the engines and of
the web backend. The `NetworkError::PeerOutsideSubnet` variant in
`core/phairplay-core/src/error.rs` is the rejection path.
**Pass Condition:** Every accepted peer address is inside the permitted subnet,
and the web backend binds to the loopback interface only
(`config/app-defaults.toml`, section `[web]`).

### FR-006 — One product, five protocols, one shared core
AirPlay 2, Miracast, Google Cast, DLNA/UPnP and Samsung Smart View are
implemented as five engines that share one core. The protocol-specific code
lives in the engine crates; the vocabulary lives in `core/phairplay-core`.

**How to Verify:** `cargo tree --workspace` shows that every engine depends on
`phairplay-core` and that no engine depends on another engine.
**Pass Condition:** The five engines exist as separate crates, none of them
depends on another, and all five implement
`phairplay_core::protocol::ProtocolEngine`.

### FR-007 — Honest capability advertisement
The receiver advertises only the codecs the device can actually decode. A
codec that is not decoded is not advertised, even if the protocol allows it.

**How to Verify:** The `MediaCapabilities` value that is handed to an engine
comes from the platform layer. Compare the advertised MIME types
(`MediaCapabilities::advertised_video_mime_types`) with the set of decoders
the platform reports.
**Pass Condition:** Every advertised codec is backed by a decoder that the
platform reports as available, and the Cast and Miracast baseline checks
(`supports_cast_baseline`, `supports_miracast_baseline`) hold before the
corresponding engine advertises itself.

---

# 2. Service discovery and advertising

### FR-008 — The AirPlay receiver advertises two service types
The AirPlay engine advertises `_airplay._tcp` **and** `_raop._tcp` over
DNS-SD. A receiver that advertises only `_airplay._tcp` is invisible to some
senders.

**How to Verify:** `cargo test -p phairplay-discovery` must contain a test
that asserts both service types for `ProtocolKind::AirPlay`
(`ProtocolKind::mdns_service_types`).
**Pass Condition:** Both service types appear in the DNS-SD advertisement of a
running AirPlay engine.

### FR-009 — Every protocol advertises itself under a stable service type
The discovery layer advertises `_miracast._tcp` for Miracast,
`_googlecast._tcp` for Google Cast and answers SSDP for DLNA/UPnP. The
Samsung Smart View receiver answers the search of the MultiScreen Framework
and belongs to the family of the service types as well —
`_samsungmsf._tcp`.

**How to Verify:** Inspect the service type table in
`core/phairplay-core/src/protocol.rs` and the registration path in
`engines/discovery`.
**Pass Condition:** Each protocol is reachable under exactly the service type
that its sender expects, and the service type of a protocol is defined in one
place only.

### FR-010 — The DNS-SD instance name is sanitised
The advertised instance name is stripped of control and non-printable
characters and capped at the DNS-SD limit. An unhandled rejection inside the
registration path would take the whole receiver down.

**How to Verify:** `cargo test -p phairplay-core` must contain the tests of
`phairplay_core::identity::sanitize_service_name` (control characters, the
length cap, the fallback).
**Pass Condition:** Every instance name that reaches the DNS-SD registrar
contains no control character and is at most
`identity::MAX_SERVICE_NAME_LENGTH` characters long.

### FR-011 — Every protocol can be switched on and off independently
A disabled protocol is not advertised and binds no socket. Switching a
protocol off releases its ports and stops its advertisement.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of
`AppConfig::is_enabled`; the engine start-up path must read the flag before
it binds a socket.
**Pass Condition:** With a protocol disabled, its service does not appear in
the DNS-SD or SSDP response and its port is not bound.

### FR-012 — Advertising is restored after a network change
When the network interface changes — a Wi-Fi reconnect, a resume from standby
— the receiver re-registers its services instead of waiting for the sender to
retry.

**How to Verify:** The re-advertisement path in `engines/discovery` and the
`discovery.re_advertise_on_network_change` default in
[`config/app-defaults.toml`](config/app-defaults.toml).
**Pass Condition:** After an interface change, the receiver's services are
registered again and a sender finds the device without a manual restart.

### FR-013 — The receiver reports its own liveness
The receiver checks that its advertisement is still alive and re-registers it
when it is not.

**How to Verify:** The check interval is the
`discovery.advertisement_check_interval_seconds` key in
[`config/app-defaults.toml`](config/app-defaults.toml); the check itself lives
in `engines/discovery`.
**Pass Condition:** A receiver whose advertisement was withdrawn by a peer or
by a daemon restart is visible again within the configured interval.

### FR-014 — The advertised name is the configured name
The name in the sender's device picker is the configured device name, not a
randomly generated one.

**How to Verify:** `DeviceIdentity::with_name` and the `device.name` default in
[`config/app-defaults.toml`](config/app-defaults.toml).
**Pass Condition:** The instance name of every service equals the sanitised
configured device name.

### FR-015 — The receiver survives a sender that vanishes
A peer that disappears without a clean teardown is reaped, and the receiver
returns to the advertising state.

**How to Verify:** The `idle_timeout_ms` key in
[`config/performance-budgets.toml`](config/performance-budgets.toml) is the
deadline of every blocking read; the session registry is the place where the
stale session is removed.
**Pass Condition:** After a sender disappears, the receiver reports no active
session on that protocol within the idle timeout and accepts a new peer.

### FR-016 — Discovery answers only what it must
An SSDP `M-SEARCH` and a DNS-SD query are answered with the service
description and nothing else. The receiver does not disclose the identity of
other devices in the network.

**How to Verify:** Inspect the response builder of the discovery engine and
the DLNA description document.
**Pass Condition:** A discovery response contains only the description of the
receiving device.

---

# 3. AirPlay 2 receiver

### FR-017 — The RTSP control plane is complete
The AirPlay engine answers the RTSP methods of the mirroring and the audio
session: `OPTIONS`, `ANNOUNCE`, `SETUP`, `RECORD`, `TEARDOWN`,
`GET_PARAMETER`, `SET_PARAMETER`, `FLUSH` and `PAUSE`.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test per
method that asserts the status code and the required headers of the answer.
**Pass Condition:** Every listed method returns a well-formed RTSP response and
no other method is answered with a success.

### FR-018 — The HTTP-like routes of the AirPlay receiver exist
The engine serves the routes that Apple senders use outside the RTSP session:
`/info`, `/play`, `/rate`, `/scrub`, `/stop`, `/playback-info`, `/photo` and
`/feedback`.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test per
route; a route that is not implemented must answer with a documented status
code and not with a connection reset.
**Pass Condition:** Each listed route answers with its documented status code
and body, and an unknown route answers `404`.

### FR-019 — The SDP of a session is parsed and rejected, never repaired
The engine parses the SDP of an `ANNOUNCE` into media sections, extracts the
H.264 SPS/PPS from `sprop-parameter-sets`, the ALAC format parameters and
the AES key material. A malformed SDP is an error.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a negative
test with a malformed SDP and one with the **SPS overflow vector** — the
negative test vector of `FR-056`, an SPS that carries 31 leading zero bits
(the overflow pattern of the H.264 `ue(v)` reader).
**Pass Condition:** A malformed SDP is rejected with a typed error, and the
SPS reader never overflows on the 31-zero-bit pattern.

### FR-020 — RTP packets are reassembled into frames
The engine reassembles the fragmented RTP payloads of the video and the audio
stream into complete frames and hands them to the media sinks.

**How to Verify:** `cargo test -p phairplay-airplay` and
`cargo test -p phairplay-h264` must contain the reassembly tests, including a
fragment overflow case.
**Pass Condition:** A complete frame is delivered to the sink, and an
overlong fragment sequence is rejected instead of being buffered without limit.

### FR-021 — One AirPlay peer at a time
A second concurrent AirPlay connection is rejected with the status code the
protocol defines for that case. It is not queued.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test that
opens a second connection while the first one is streaming.
**Pass Condition:** The second connection is refused, the first session
continues undisturbed, and the receiver does not open a second socket for it.

### FR-022 — The receiver answers with the configured identity
The `SET_PARAMETER` answer of the receiver carries the configured device
name, the model, the manufacturer and the protocol version.

**How to Verify:** `DeviceIdentity` in `core/phairplay-core/src/identity.rs`
and the `device` section of [`config/app-defaults.toml`](config/app-defaults.toml).
**Pass Condition:** The identity in the RTSP answer equals the configured
identity.

### FR-023 — Pairing with the HomeKit-style procedure
The engine supports the Ed25519/X25519 pairing of modern Apple senders and
the legacy SRP PIN pairing, and it stores the pairing material with
owner-only permissions.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a pairing
test per procedure; the attempt limit is the key `limits.max_pairing_attempts`
of [`config/performance-budgets.toml`](config/performance-budgets.toml) and
the file permissions are the key `pairing.store_permissions` of
[`config/security-policy.toml`](config/security-policy.toml).
**Pass Condition:** A sender that has completed a pairing can connect again
without a new pairing, and a sender that fails repeatedly is locked out after
the configured number of attempts.

### FR-024 — The FairPlay session key is decrypted
The engine implements the `fp-setup` handshake (version 2 and version 3) and
the legacy `rsaaeskey` exchange, and decrypts the session key that protects
the stream.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test that
runs the handshake against a known key and checks the derived session key.
**Pass Condition:** The session key of a known handshake is derived correctly
and the encrypted stream is decrypted with it.

### FR-025 — FairPlay DRM content is not decrypted — and the receiver says so
The receiver does not decrypt FairPlay DRM content. This is a product
boundary, not a defect, and it is documented in the exclusion list of this
document.

**How to Verify:** The `supports_fairplay_drm = false` entry in
[`config/security-policy.toml`](config/security-policy.toml) and the exclusion
`EXC-001` of this document.
**Pass Condition:** No code path claims to decrypt DRM content, and the
public documentation states the limitation in the same words.

### FR-026 — Now-playing metadata and artwork are delivered
The DMAP metadata of the sender (title, artist, album) and the artwork are
reported as session events and are shown in the shell.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of the
`SessionEvent::MetadataChanged` and `SessionEvent::ArtworkChanged` folding;
`cargo test -p phairplay-airplay` must contain the test that produces them.
**Pass Condition:** The metadata of a session reaches the shell unchanged, and
an artwork payload that exceeds the configured limit is rejected.

### FR-027 — The reverse remote is implemented
The DACP reverse remote lets the receiver control the playback of the sender.

**How to Verify:** `cargo test -p phairplay-airplay` must contain the test of
the DACP request builder.
**Pass Condition:** A play, pause or skip command from the shell reaches the
sender and the answer of the sender is reported back.

### FR-028 — Audio and video are synchronised
The NTP timing of the sender is used to align the audio and the video
stream.

**How to Verify:** `cargo test -p phairplay-airplay` must contain the test of
the timing synchronisation; the budget is the `latency.audio_video_sync_ms`
key in [`config/performance-budgets.toml`](config/performance-budgets.toml).
**Pass Condition:** The deviation between the presentation timestamps of the
audio and the video stays inside the configured budget for a session of at
least one minute.

### FR-029 — Photos are received and displayed
The engine accepts the photo transfer of the AirPlay protocol, supports JPEG
and PNG and hands the picture to the shell.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test per
image format; the size limit is the `limits.max_photo_bytes` key in
[`config/performance-budgets.toml`](config/performance-budgets.toml).
**Pass Condition:** A valid photo is delivered to the shell, and a photo that
exceeds the configured size limit is rejected without being buffered.

### FR-030 — The audio-only session works without a video stream
A sender that streams only audio (music, a podcast) is served without a
video track.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test that
starts a session without a video media section.
**Pass Condition:** The audio session starts, is reported in the session
snapshot and reaches the audio sink.

### FR-031 — The AirPlay session is torn down cleanly
`TEARDOWN` and `FLUSH` release the peer connection, close the stream and
return the engine to the advertising state.

**How to Verify:** `cargo test -p phairplay-airplay` must contain a test that
tears a live session down; the state machine is described in
[`ARCHITECTURE.md`](ARCHITECTURE.md) §5.
**Pass Condition:** After the teardown the engine reports the `Stopped` or
`Advertising` state, no peer is counted, and the ports are released.

---

# 4. Miracast / Wi-Fi Display receiver

### FR-032 — The Miracast engine is a protocol engine
The Miracast engine implements `phairplay_core::protocol::ProtocolEngine` and
passes through the same state machine as every other engine.

**How to Verify:** `cargo test -p phairplay-miracast` must contain a test of
the state transitions.
**Pass Condition:** `Idle → Starting → Advertising → Streaming → Stopping →
Stopped` is reachable and observable through `ProtocolEngine::status`.

### FR-033 — The WFD control plane is answered
The engine answers the RTSP control plane of Wi-Fi Display, parses the WFD
information elements and answers the sink discovery request.

**How to Verify:** `cargo test -p phairplay-miracast` must contain a test for
the information-element parser, including a negative test with a broken
length field.
**Pass Condition:** A well-formed WFD information element is parsed, and an
element with an impossible length is rejected instead of being allocated.

### FR-034 — The Miracast baseline codecs are the advertised ones
The engine advertises H.264 for video and linear PCM for audio, which is the
baseline of the WFD protocol. The advertisement is derived from
`MediaCapabilities::supports_miracast_baseline`.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of
`supports_miracast_baseline`; `cargo test -p phairplay-miracast` must assert
that the advertised codec set is the baseline set.
**Pass Condition:** A receiver without the baseline codecs does not advertise
the Miracast service at all.

### FR-035 — Miracast needs the WFD permission of the system
Miracast uses a system-level Wi-Fi Display permission on the platforms that
provide one (Android: the Wi-Fi Display / Wi-Fi Direct system feature; macOS:
no system WFD sink exists). The receiver cannot work around a missing
permission.

**How to Verify:** The permission handling of the Android shell in
`apps/android` and the platform capability query that feeds
`MediaCapabilities`.
**Pass Condition:** A platform without the WFD permission reports the missing
capability instead of advertising a service that cannot work, and the user
sees the reason.

### FR-036 — A second Miracast peer is rejected
Miracast serves one peer at a time. A second connection is refused.

**How to Verify:** `cargo test -p phairplay-miracast` must contain a test with
two concurrent connections.
**Pass Condition:** The second connection is refused and the first session
continues.

---

# 5. Google Cast receiver

### FR-037 — The Cast engine is a protocol engine
The Cast engine implements `phairplay_core::protocol::ProtocolEngine`.

**How to Verify:** `cargo test -p phairplay-cast` must contain a test of the
state transitions.
**Pass Condition:** The engine reaches the advertising state and reports it
through `ProtocolEngine::status`.

### FR-038 — The Cast control plane speaks CastV2 over TLS
The engine accepts the TLS connection of the Cast protocol, answers the
`PING`/`PONG` heartbeat and speaks the CastV2 framing.

**How to Verify:** `cargo test -p phairplay-cast` must contain a test of the
framing, including a negative test with an oversized length field.
**Pass Condition:** A frame with an impossible length is rejected, and the
heartbeat is answered.

### FR-039 — The Cast application identifier is handled
The receiver answers with the configured Cast application identifier and reacts
to an unknown identifier in a defined way.

**How to Verify:** The `device` section of
[`config/app-defaults.toml`](config/app-defaults.toml) and the Cast engine
tests.
**Pass Condition:** A sender that presents the configured identifier
establishes a session; a sender that presents an unknown one is refused with
a documented error.

### FR-040 — Cast is the documented exception to the one-peer rule
Google Cast may serve several senders at the same time. The rule lives in one
place: the `session.multi_peer_protocols` list in
[`config/app-defaults.toml`](config/app-defaults.toml).

**How to Verify:** `cargo test -p phairplay-cast` must contain a test with two
concurrent senders; the other engines must contain the opposite test.
**Pass Condition:** Two Cast senders stream at the same time, and no other
protocol accepts a second peer.

### FR-041 — The Cast baseline codecs are the advertised ones
The engine advertises the baseline of the Cast protocol — H.264 and VP8 for
video, AAC-LC, MP3 and linear PCM for audio — and derives the advertisement
from `MediaCapabilities::supports_cast_baseline`.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of
`supports_cast_baseline`.
**Pass Condition:** A receiver without the baseline codecs does not advertise
the Cast service.

### FR-042 — The media status of a Cast session is reported
The receiver reports the state of the media playback — the transport state, the
position and the metadata — as session events.

**How to Verify:** `cargo test -p phairplay-cast` must contain the test of the
media receiver status, and `cargo test -p phairplay-core` the folding of
`SessionEvent::PlaybackStateChanged`.
**Pass Condition:** The state of a session is visible in the shell and in the
dashboard.

---

# 6. DLNA / UPnP MediaRenderer

### FR-043 — The DLNA engine answers SSDP
The engine answers an SSDP `M-SEARCH` with a `NOTIFY` and registers itself
with the `alive` notification of UPnP.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a test of the
SSDP answer, including a negative test with an oversized search target.
**Pass Condition:** An SSDP search finds the device, and an oversized search
is rejected.

### FR-044 — The device description and the service descriptions are served
The engine serves the device description document and the service description
documents of the ConnectionManager, the AVTransport and the RenderingControl
services.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a test per
document.
**Pass Condition:** Every advertised service has a description document that
parses as well-formed XML.

### FR-045 — The SOAP dispatcher answers the three services
The dispatcher implements `ConnectionManager`, `AVTransport` and
`RenderingControl` and answers a `SOAPAction` it does not know with a
documented error.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a test per
action.
**Pass Condition:** Every implemented action answers with a valid SOAP
response, and an unknown action is refused.

### FR-046 — GENA events are delivered
The engine delivers the GENA event notifications of the state variables it
implements.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a test of the
event delivery.
**Pass Condition:** A subscribed client receives the state change it
subscribed to.

### FR-047 — The SOAP body is parsed with the XML rules of the policy
A SOAP body with a `DOCTYPE` or with an external entity is rejected. The
rules live in [`config/security-policy.toml`](config/security-policy.toml),
section `[xml]`.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a negative test
with a `DOCTYPE` and one with an external entity.
**Pass Condition:** Both bodies are rejected with a typed error, and no
external resource is ever contacted while parsing a body.

### FR-048 — The GENA callback is checked against the SSRF guard
A callback URL is accepted only when it points at a private address of the
local subnet. The link-local address of the cloud metadata service, the
loopback address and the unspecified address are refused.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a negative test
with a callback that points at `169.254.169.254` and one with `::1`; the
allowed ranges are the `callbacks.allowed_cidr` list in
[`config/security-policy.toml`](config/security-policy.toml).
**Pass Condition:** Both refused callbacks are rejected and the receiver does
not open a connection to them.

### FR-049 — The payload limits of DLNA are enforced
The size of a media payload, the number of headers and the length of a header
line are limited by the configuration, not by the protocol implementation.

**How to Verify:** The `limits` section of
[`config/performance-budgets.toml`](config/performance-budgets.toml); the
negative tests of `cargo test -p phairplay-dlna`.
**Pass Condition:** A request that exceeds a limit is refused before the
payload is allocated.

### FR-050 — DLNA serves audio and pictures, not a live video stream
The DLNA engine is a media renderer for audio and pictures. It does not render
a live video stream.

**How to Verify:** `ProtocolKind::Dlna::carries_video` in
`core/phairplay-core/src/protocol.rs` and the DLNA service descriptions.
**Pass Condition:** The DLNA service description advertises no video
rendering, and the engine rejects a video rendering request.

### FR-051 — One DLNA peer at a time
A second concurrent control connection is refused.

**How to Verify:** `cargo test -p phairplay-dlna` must contain a test with two
concurrent connections.
**Pass Condition:** The second connection is refused and the first session
continues.

---

# 7. The format and codec layer

### FR-052 — The property list codec reads and writes both formats
The property list crate encodes and decodes the XML format and the binary
format `bplist00`, in both directions.

**How to Verify:** `cargo test -p phairplay-plist` must contain a round-trip
test per format.
**Pass Condition:** A value that is encoded in the XML format is decoded
again to the same value, and the same holds for the binary format.

### FR-053 — The binary property list decoder rejects a cyclic object graph
A property list whose object references form a cycle is rejected instead of
being followed.

**How to Verify:** `cargo test -p phairplay-plist` must contain a negative
test with a cyclic object graph.
**Pass Condition:** The decoder returns a typed error and terminates.

### FR-054 — A length field is never used unchecked as an allocation size
A declared length that exceeds the message limit is rejected before a buffer
is allocated.

**How to Verify:** `cargo test -p phairplay-plist` must contain a negative
test with an oversized declared length.
**Pass Condition:** The decoder allocates nothing for the oversized object and
returns a typed error.

### FR-055 — The H.264 reader parses SPS and PPS and splits NAL units
The H.264 crate parses the sequence and the picture parameter set, splits a
Annex-B byte stream into NAL units and reads the exponential-Golomb fields.

**How to Verify:** `cargo test -p phairplay-h264` must contain the SPS, the
PPS and the NAL splitting tests.
**Pass Condition:** A known SPS yields the expected width, height and profile,
and a byte stream is split into the expected NAL units.

### FR-056 — The exponential-Golomb reader does not overflow
The reader computes in 64 bits and clamps. A sequence parameter set with 31
leading zero bits is rejected, not silently wrapped.

This requirement owns the **SPS overflow vector**: the negative test input
that every document of the project refers to as "the SPS overflow vector".

**How to Verify:** `cargo test -p phairplay-h264` must contain the negative
test with the 31-zero-bit pattern.
**Pass Condition:** The parser returns an error and no value wraps around.

### FR-057 — The AAC crate reads the ADTS header and frames AAC-ELD
The AAC crate parses the ADTS header, frames the AAC-ELD payload and counts
the samples.

**How to Verify:** `cargo test -p phairplay-aac` must contain a test per
function.
**Pass Condition:** The sample rate, the channel count and the sample count
of a known ADTS frame are read correctly.

### FR-058 — The ALAC decoder is a native Rust port
The ALAC crate is a Rust port of the Apple reference decoder, not a copy of
the C sources. Every file derived from the reference carries an attribution
header.

**How to Verify:** Inspect the header of every file in `formats/alac` and the
third-party inventory of the workspace.
**Pass Condition:** Every derived file names Apple and the Apache-2.0
licence, and no C source of the reference decoder is present in the tree.

### FR-059 — An unfinished port is marked as unfinished
If the ALAC port is not complete, the crate says so in its module
documentation and the remaining work is listed. A partial port is never
presented as a complete one.

**How to Verify:** Read the module documentation of `formats/alac` and the
acceptance record of the milestone that covers it.
**Pass Condition:** The documentation names the parts that are done and the
parts that are open, and the two lists agree with the tests.

### FR-060 — The format crates are pure
A format crate performs no I/O, opens no socket and calls no platform
function. It transforms bytes into bytes.

**How to Verify:** `cargo tree --workspace` shows that no format crate
depends on a crate that provides I/O or a platform API, and the sources
contain no `std::net` or `std::fs` usage.
**Pass Condition:** A format crate is usable in a test that has no network
and no file system.

---

# 8. Session lifecycle and control

### FR-061 — Every engine is a state machine
The states are `Idle`, `Starting`, `Advertising`, `Streaming`, `Stopping`,
`Stopped` and `Failed`. `status()` is callable at any moment, also from
another thread while a transition is in flight.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of
`ProtocolState::accepts_peers`; every engine crate must contain a test of its
own transitions.
**Pass Condition:** Only the states `Advertising` and `Streaming` accept a
peer, and a concurrent `status()` call never observes an invalid state.

### FR-062 — A session is reported with `Started` and `Ended`
An engine emits `SessionEvent::Started` when it accepts a peer and
`SessionEvent::Ended` when the session terminates.

**How to Verify:** `cargo test -p phairplay-core` must contain the tests of
the session registry; every engine crate must contain a test that observes
both events.
**Pass Condition:** The registry knows about a live session from the first
event to the last, and the session disappears from it when the session ends.

### FR-063 — A late `Ended` event does not clear a newer session
The registry ignores an `Ended` event that belongs to a session that is not
the current one on that protocol.

**How to Verify:** `cargo test -p phairplay-core` must contain the test
`an_end_for_an_unknown_session_does_not_clear_a_newer_one`.
**Pass Condition:** After a late `Ended` event, the newer session is still
reported as live.

### FR-064 — The reason of an end is always recorded
An ended session carries a reason: the sender disconnected, the receiver
stopped it, the peer violated the protocol, a limit was exceeded, or the
session failed.

**How to Verify:** `SessionEndReason` in `core/phairplay-core/src/session.rs`.
**Pass Condition:** Every end event carries one of the documented reasons; the
reason reaches the shell and the dashboard.

### FR-065 — The metrics of a session are objective
The receiver counts the delivered and the dropped video frames, the delivered
and the dropped audio packets, the received and the sent bytes, the packets
the sender reported as lost and the frames the sender reported as skipped.

**How to Verify:** `cargo test -p phairplay-core` must contain the tests of
`StreamCounters`; the counters are filled by the engines.
**Pass Condition:** The counters of a session are non-zero for a session that
streamed, and the drop ratio is `0.0` when nothing was delivered.

### FR-066 — A stalled decoder never becomes an out-of-memory kill
A sink that cannot keep up reports `MediaError::QueueFull` instead of growing
without limit. The queues are bounded by an item count **and** by a byte
budget.

**How to Verify:** The `queues` section of
[`config/performance-budgets.toml`](config/performance-budgets.toml) and the
negative tests of the engines; the test of `StreamCounters::drop_ratio` in
the core.
**Pass Condition:** A session that is fed faster than the sink can consume
reports a rising drop counter and a stable resident memory.

### FR-067 — The background work does not block a protocol
The pruning of the pairing store, the rotation of the log, the re-advertising
after a network change and the reaping of expired sessions run in the job
executor and not inside a protocol event loop.

**How to Verify:** `cargo test -p phairplay-core` must contain the tests of
the job executor; the executor is documented in
[`ARCHITECTURE.md`](ARCHITECTURE.md) §8.
**Pass Condition:** A running job does not delay the event loop of an engine
that is streaming.

### FR-068 — A job that fails is counted, not hidden
The executor counts the executed and the failed jobs and exposes them through
`JobExecutor::stats`.

**How to Verify:** `cargo test -p phairplay-core` must contain the test
`a_failing_job_is_counted`.
**Pass Condition:** A failing job increases the failure counter and the
counter is visible to the shell.

### FR-069 — The receiver survives a broken peer
A peer that violates the protocol ends its session with the reason
`ProtocolViolation` and does not take the receiver with it.

**How to Verify:** The negative tests of every engine crate.
**Pass Condition:** After the violation the engine is still in the
`Advertising` state and accepts a new peer.

---

# 9. Platform bridge and native shells

### FR-070 — The bridge is the only way into the core
The native shells reach the receiver only through the UniFFI bridge in
`platform/phairplay-ffi`. They never talk to an engine directly.

**How to Verify:** The boundary check of the project
(`scripts/check-crate-boundaries.sh`, delivered with the architecture gate)
and the import list of the shells in `apps/android` and `apps/macos`.
**Pass Condition:** The shells import the generated bindings and nothing else
from the workspace.

### FR-071 — No panic ever crosses the bridge
Every error that crosses the bridge arrives as a structured error with a
stable numeric code and a message. A panic in the Rust code must not reach
the native side.

**How to Verify:** `cargo test -p phairplay-ffi` must contain the round-trip
test of the error mapping; `ErrorCode` in
`core/phairplay-core/src/error.rs` carries the stable numbers.
**Pass Condition:** Every documented failure of the bridge is reachable as a
typed error, and the numeric codes do not change without a major version
bump.

### FR-072 — The value round trip is proven
A value that goes into the bridge comes out of it unchanged. A test proves
the binding, not only that it compiles.

**How to Verify:** `cargo test -p phairplay-ffi` must contain a round-trip
test per exported data type.
**Pass Condition:** Every exported data type has a round-trip test that
compares the value before and after the crossing.

### FR-073 — The generated bindings are never edited by hand
The Kotlin and the Swift bindings are generated from the Rust sources. A
hand-edited generated file is a defect.

**How to Verify:** The binding generation script in `scripts/` and the header
of the generated files.
**Pass Condition:** A regeneration produces no diff in a clean tree.

### FR-074 — The Android shell runs as a foreground service
The Android receiver keeps running while the interface is in the background,
and it shows a persistent notification with the state of the receiver.

**How to Verify:** The manifest of the Android shell in `apps/android` and the
service declaration.
**Pass Condition:** A mirroring session survives the backgrounding of the
interface, and the notification is present while the service runs.

### FR-075 — The Android permission list is minimal
The shell asks only for the permissions that the receiver really needs.

**How to Verify:** The manifest of the Android shell in `apps/android`.
**Pass Condition:** The manifest contains no location, no microphone and no
contact permission, and the list is limited to the network permissions that
the protocols require.

### FR-076 — The macOS shell shows the same information architecture
The macOS shell presents the same information as the Android shell: one card
per protocol with the state, the port, the number of peers and the switch.

**How to Verify:** The sources of the macOS shell in `apps/macos` and the
acceptance record of the milestone that covers it.
**Pass Condition:** Both shells distinguish the states "disabled", "advertising",
"streaming" and "error", and neither confuses "stopped" with "disabled".

### FR-077 — The build information is available at runtime
The version, the target triple, the compiler version and the licence of the
build are compiled into the binary and are shown in the settings of the app.

**How to Verify:** `BuildInfo::current` in
`core/phairplay-core/src/build_info.rs` and the build script of the core.
**Pass Condition:** A bug report can name the exact build that produced it.

### FR-078 — The build contract is obeyed
Every crate of the workspace obeys `docs/design/BUILD-CONTRACT.md`: the
dependency direction is strict, an engine never queries the platform for
capabilities, and an engine never decodes.

**How to Verify:** The boundary check of the project
(`scripts/check-crate-boundaries.sh`, delivered with the architecture gate)
and the contract itself.
**Pass Condition:** The boundary check is green and the contract describes the
signatures that the core really has.

---

# 10. The local web dashboard

### FR-079 — The dashboard binds on the loopback interface only
The web backend of the dashboard is a local control panel, not a network
service. It never accepts a connection from a foreign address.

**How to Verify:** The bind address in the web backend in `apps/web` and the
`web.bind_address` default in
[`config/app-defaults.toml`](config/app-defaults.toml).
**Pass Condition:** The server binds on the loopback interface, and a request
from a foreign address is refused.

### FR-080 — The dashboard shows the state of every protocol
The dashboard shows one card per protocol with the state, the port, the
number of peers, the switch and the live counters.

**How to Verify:** The API of the web backend in `apps/web` and the assets in
`apps/web/assets`.
**Pass Condition:** The card of every protocol reflects the state of the
engine, and the four states are distinguishable.

### FR-081 — The dashboard reads the core and invents nothing
Every value that the dashboard shows comes from the session registry, from
the metrics or from the build information of the core.

**How to Verify:** The data source of every endpoint of the web backend.
**Pass Condition:** No endpoint computes a value that the core does not
provide.

### FR-082 — The dashboard can switch a protocol on and off
A protocol can be enabled and disabled from the dashboard.

**How to Verify:** The enable and the disable endpoint of the web backend.
**Pass Condition:** The switch changes the state of the engine, and the
change is visible in the status of the next request.

### FR-083 — The dashboard delivers the events live
The dashboard receives the session events over the event channel.

**How to Verify:** The event endpoint of the web backend and the
`SessionEvent` type of the core.
**Pass Condition:** An event that the core emits appears in the dashboard
without a reload.

### FR-084 — The dashboard is accessible
The dashboard is operable with the keyboard, has a visible focus, meets the
contrast requirement and respects the reduced-motion preference. The keyboard
operation is the operation of a remote control: the arrow keys move the focus
inside the group of the cards, and the group is one tab stop (`FR-096`).

**How to Verify:** The assets of the dashboard in `apps/web/assets`, the three
checks in `apps/web/tools` and the accessibility section of the acceptance
record.
**Pass Condition:** Every control is reachable with the keyboard, the text
contrast is at least the required ratio and the animation is suppressed when
the user asked for it.

### FR-085 — The dashboard loads no external resource
The dashboard is self-contained: no CDN, no web font, no icon pack of a third
party.

**How to Verify:** Search the assets of the dashboard for external URLs.
**Pass Condition:** No external resource is loaded; the dashboard works with
the network disconnected.

---

# 11. Configuration, limits and observability

### FR-086 — The configuration has exactly one model
The configuration is loaded from TOML into one typed model, and every value
has a default. A missing key is never an error, a wrong value always is.

**How to Verify:** `cargo test -p phairplay-core` must contain the test of
the TOML round trip and the test of the validation.
**Pass Condition:** A missing key yields the default, an invalid value yields
a `ConfigError::Invalid` with the dotted path of the offending key.

### FR-087 — An unknown key is an error
A configuration file with a key that the model does not know is rejected
instead of being ignored silently.

**How to Verify:** `cargo test -p phairplay-core` must contain the test
`an_unknown_key_is_rejected`; `AppConfig` is derived with
`deny_unknown_fields`.
**Pass Condition:** A file with an unknown key is rejected with a message that
names the key.

### FR-088 — The limits are read from the configuration
The engines read their limits from the configuration model. They never
hard-code a limit of their own.

**How to Verify:** Search the engine crates for numeric literals that
correspond to a limit in
[`config/performance-budgets.toml`](config/performance-budgets.toml).
**Pass Condition:** No engine contains a hard-coded message size, header
count, header length or queue size.

### FR-089 — The defaults of the documentation and the code agree
The typed model in `core/phairplay-core/src/config.rs` carries the same
values as the TOML files in [`config/`](config/README.md).

The budgets of [`config/performance-budgets.toml`](config/performance-budgets.toml)
fall into two groups, and both are checked:

- **Mirrored budgets** — every value that has a typed counterpart in the Rust
  model. The script compares the two and fails on a difference.
- **Budgets owned by the file alone** — the measurement targets, the bounds
  of the job executor and the code limits. They have no typed counterpart;
  the script checks that they exist and that their value is not written down
  a second time in the documentation.

**How to Verify:** `bash scripts/check-docs-links.sh` compares the mirrored
budgets with the Rust defaults, checks the existence and the uniqueness of
the budgets that the file owns alone, and fails on a difference.
**Pass Condition:** The check is green and reports the count of both groups;
a difference or a second copy makes it red.

### FR-090 — The receiver writes no log about a peer that identifies it
Sender names and IP addresses are masked in every log line, and the content
of a stream never reaches the log.

**How to Verify:** The masking in the core and the
`privacy.mask_device_names` and `privacy.mask_ip_addresses` defaults in
[`config/security-policy.toml`](config/security-policy.toml).
**Pass Condition:** A log line of a session contains no complete sender name
and no complete IP address.

### FR-091 — The identity of the receiver is stable
The device identifier of the installation is persisted and does not change
between two starts, and it is not derived from the display name.

**How to Verify:** `DeviceIdentity` in
`core/phairplay-core/src/identity.rs`.
**Pass Condition:** Two starts of the receiver produce the same device
identifier, and a renamed display name does not change it.

### FR-092 — The build information is complete
The version, the name, the licence, the repository, the compiler version and
the target triple are compiled into the binary.

**How to Verify:** `cargo test -p phairplay-core` must contain the tests of
`BuildInfo`.
**Pass Condition:** No field of the build information is empty.

### FR-093 — The error taxonomy is closed
The core owns one error hierarchy. An engine does not introduce a parallel
error enum.

**How to Verify:** The `CoreError` hierarchy in
`core/phairplay-core/src/error.rs` and the error types of the engines.
**Pass Condition:** Every error of an engine is a variant of the core
hierarchy.

### FR-094 — The numeric error codes are stable
The numeric codes that the bridge hands to the shells are part of the public
contract of the bridge and are not renumbered without a major version bump.

**How to Verify:** `cargo test -p phairplay-core` must contain the test
`error_codes_are_stable`.
**Pass Condition:** The documented code of every `ErrorKind` equals the
implemented value.

### FR-095 — The error message carries the cause chain
A failure is reported with its cause, on one line, so that a log stays
readable.

**How to Verify:** `describe` in `core/phairplay-core/src/error.rs` and its
test.
**Pass Condition:** The rendered chain contains the cause of the error and
contains no line break.

---

# 12. The interface as a remote control

The dashboard and the native shells are built for a television and for a
remote control, not for a mouse. A remote control has an arrow pad, an *OK*
button and — usually — no tab key at all.

### FR-096 — The group of the cards is one tab stop, and the arrow keys move inside it
The cards of the receivers form one group in the tab order: exactly one card
carries the focus, and the arrow keys move it to the neighbouring card. At the
edge of the group an arrow key leaves the group and reaches the next control
of the page — a remote control without a tab key would otherwise come to a
dead end at the last card.

**How to Verify:** `node apps/web/tools/check-keyboard.mjs` drives a real
Chromium with the keyboard only — the pointer is never used. The check
asserts that exactly one card is in the tab order, that every arrow key
reaches the card the eye expects, and that the arrow key at the edge reaches
the control behind the group. The raw output is in
[`docs/audit/tv-web-dashboard-keyboard.log`](docs/audit/tv-web-dashboard-keyboard.log),
the report in [`docs/audit/tv-web-dashboard.md`](docs/audit/tv-web-dashboard.md).
**Pass Condition:** The check reports no failure, exactly one card of the
group is in the tab order, and every arrow key at the edge of the group
leaves the group instead of staying where it is.

### FR-097 — The focus is visible and measured
Every focusable surface of the dashboard and of the native shells carries a
**drawn** focus frame. The frame of the system is not enough: it is thin, it
is dark, and it is nearly invisible on a dark surface. The frame of the
project is measured against the surface it lies on and against the page, and
it appears at once — a focus that fades in over a fraction of a second is a
focus the user does not see.

**How to Verify:** `node apps/web/tools/check-contrast.mjs` computes the
contrast of the ring of the focus with the formula of WCAG in both themes,
and `node apps/web/tools/check-layout.mjs` measures the width of the ring
and the reduced motion. The measured pairs are in
[`docs/audit/tv-web-dashboard.md`](docs/audit/tv-web-dashboard.md) §4. The
Android shell measures its frame against the card and against the screen in
§8.1 of
[`docs/audit/tv-android-shell.md`](docs/audit/tv-android-shell.md).
**Pass Condition:** The ring of the focus is at least 3 px wide, reaches the
contrast of 3:1 that WCAG asks for against the card and against the page in
both themes, is not animated into existence, and stays visible under
`prefers-reduced-motion`.

### FR-098 — The card is the switch
The card of a protocol **is** the control that switches the protocol. A
switch of a few pixels in the corner of a card is a target that nobody hits
with a remote control, so there is no second, smaller control to reach
first: *OK*, *Enter* and *Space* act on the card that holds the focus.

**How to Verify:** The keyboard check of the dashboard asserts that *Enter*
and *Space* switch the receiver and that the switch calls the real endpoint
of the receiver (`POST /api/protocols/<id>/enable`). The Android shell
proves the order of the focus with `TvFocusGraphTest`, the macOS shell with
`ShellKeyboardTests.testTheSpaceKeySwitchesTheFocusedProtocol` — the
evidence is in [`docs/audit/tv-android-shell.md`](docs/audit/tv-android-shell.md)
and in [`docs/audit/tv-macos-shell.md`](docs/audit/tv-macos-shell.md).
**Pass Condition:** The protocol is switched by the card alone, the state of
the card changes after the key press, and the state survives the next
reading of the API.

### FR-099 — The Android shell is an application of the television
The Android shell is a real television application: its manifest carries the
`LEANBACK_LAUNCHER` category, the banner that the launcher of the television
scales, and the `uses-feature` of the television. The shell stays one
application for the telephone and for the television.

**How to Verify:** The merged manifest of the build carries the category, the
banner and the `uses-feature` — the excerpt is in
[`docs/audit/tv-android-shell.md`](docs/audit/tv-android-shell.md) §8.5.
**Pass Condition:** The merged manifest carries `LEANBACK_LAUNCHER`, a
banner and `android.software.leanback`, and the application appears in the
list of the television. The appearance in the list needs a **device test**
and is **NOT RUN** in this repository.

---

# 13. The headless build

A receiver that runs on a device without a screen — a Raspberry Pi, a NAS, a
container on a server — must not wait for a click in a browser. The program
is the product there.

### FR-100 — The receiver starts the protocols itself
`phairplay-web` starts every protocol that the configuration enables at the
start of the process. The start is a **total judgement**: every protocol of
the core gets an outcome — started, failed or switched off — and a protocol
that cannot start (a container without multicast, a device without a
Wi-Fi Direct radio) is reported with its reason and does **not** stop the
other protocols.

**How to Verify:** The tests `every_protocol_is_answered_for` and
`a_protocol_that_is_switched_off_is_not_started` in
`apps/web/src/receiver.rs`, and the report that the program prints at the
start (`phairplay-web --profile generic` names every protocol with its
state).
**Pass Condition:** The report of the start names every protocol of
`ProtocolKind::ALL`; a protocol that is switched off is not started and says
why; a protocol that fails does not stop the others.

### FR-101 — The configuration of the receiver is read from a file
The receiver reads its configuration from a TOML file. The file is laid over
the base of the configuration, so that what the file names wins and what it
does not mention keeps the value of the base. A file that cannot be read,
that is not valid TOML or that does not fit the model is refused with a
message that names the file and the reason.

**How to Verify:** The tests `a_file_lays_over_the_profile`,
`a_broken_configuration_file_names_the_file_and_the_reason` and
`a_missing_configuration_file_names_the_file` in
`apps/web/src/receiver.rs`; the test of the core that refuses an unknown
key.
**Pass Condition:** A file that names a value overrides the base, a file
that does not mention a value leaves it alone, and a broken file is refused
with a message that names the file and the reason.

### FR-102 — The receiver knows the profiles of the classes of devices
The receiver knows the profiles of the classes of devices it runs on: a
single-board computer, a NAS, a container, a desktop, and the neutral
fallback. The name of a profile is the name of its file, a profile is a
complete configuration that can be handed to `--config` unchanged, and a
profile that nobody knows is refused with the list of the known ones.

**How to Verify:** `phairplay-web --help` names every profile and its
sentence; the test `every_profile_of_the_repository_is_found_and_loaded` in
`apps/web/src/profiles.rs` loads every profile of
[`config/profiles/`](config/profiles/README.md) and validates it; the test
`an_unknown_profile_is_refused` in `apps/web/src/cli.rs`.
**Pass Condition:** Every profile of the directory is loadable, valid and
named in the help of the program; an unknown name is refused with the list of
the known names.

### FR-103 — A control panel in the network is announced
The dashboard binds to the loopback interface by default. A `--bind` to
another address is possible — a user may want to reach the panel from
another machine — but it is a different security model, so the program warns
about it on stderr **before** it starts anything.

**How to Verify:** The test `the_default_interface_is_the_loopback_interface`
in `apps/web/src/cli.rs` and the warning that `phairplay-web --bind
0.0.0.0` prints on stderr.
**Pass Condition:** Without an option the dashboard binds to the loopback
interface; with a foreign address the warning names the address and appears
before the first protocol is started.

---

# 14. The Samsung Smart View receiver

Samsung Smart View is the screen mirroring of the Galaxy phones and tablets.
Its framework is the MultiScreen Framework (MSF). The project implements the
discovery and the handshake; the limit of the channel is written down as
`EXC-011`.

### FR-104 — The receiver answers the search of the MultiScreen Framework
A Galaxy device finds a receiver with an SSDP search for
`urn:samsung.com:service:MultiScreenService:1`. The receiver answers it with
an HTTP answer that carries the `LOCATION` header of the description
document, and it announces itself — and takes itself away again — with the
`NOTIFY` messages of the framework.

**How to Verify:** The tests of the crate `engines/smartview`:
`a_search_for_the_multi_screen_service_is_answered`,
`the_search_target_is_the_one_samsung_documents`,
`the_answer_carries_the_location_and_the_target` and
`the_announcement_announces_the_receiver_and_its_leaving` — and, over a real
socket, `a_search_for_the_multi_screen_service_is_answered_over_the_network`
and `the_announcement_reaches_the_multicast_group`. The output of the run is
in [`docs/audit/smartview-engine-test.log`](docs/audit/smartview-engine-test.log).
The engine is registered with the supervisor in the composition root
(`apps/web/src/receiver.rs`), so the receiver starts it with the other
protocols.
**Pass Condition:** The search of the framework is answered with the target
of the framework and a `LOCATION` that points at the description document; a
search for another service is not answered; the announcement reaches the
multicast group of the network; the protocol is among the protocols that the
receiver starts and reports in its status. The SSDP port of the workspace is
shared with the DLNA renderer: the conflict is named in
[`ROADMAP.md`](ROADMAP.md) (M5), and a protocol that cannot open its socket
is reported with the reason instead of disappearing from the status.

### FR-105 — The receiver serves the description document and the API of the framework
The description document that the `LOCATION` header points at is served, and
so is the API below `/api/v2/`: the root of the API, the description of the
resources, the base URL, the handshake of the sender and the WebSocket
upgrade of the channel.

**How to Verify:** The tests of the API of the crate `engines/smartview`:
`the_api_root_names_the_service_and_the_base_url`,
`the_description_document_is_served_as_xml`,
`the_handshake_records_the_announced_peer` and
`the_websocket_upgrade_is_accepted`.
**Pass Condition:** The description document names the display name, the
model and the base URL of the receiver; the handshake records the peer that
announced itself; the upgrade is answered with the accept key of RFC 6455.

### FR-106 — The Smart View receiver serves one sender at a time
The channel of the Smart View receiver accepts one sender. A second channel
is refused, and the first one keeps its slot.

**How to Verify:** The test
`a_second_channel_is_refused_and_the_first_one_keeps_the_slot` and the test
`a_peer_that_opened_a_channel_is_counted` of the crate `engines/smartview`.
**Pass Condition:** The second channel is refused, the session of the first
sender is not disturbed, and the status of the protocol reports one peer.

### FR-107 — The receiver never claims a picture that it cannot deliver
The state of the Smart View protocol reports the session of the sender, but
the receiver never claims to deliver a picture over the channel: the
transport of the picture is proprietary and not implemented. The limit is
written down — in the requirements, in the glossary, in the architecture and
in the user-facing documentation — with the same words.

**How to Verify:** The exclusion `EXC-011` of this document, the section
"The honest limit" of the module documentation of `engines/smartview` and
the entry of the glossary.
**Pass Condition:** No document, no status and no log claims that a picture
of a Galaxy device flows; the limit is stated with the same words in every
place that names the protocol.

### FR-108 — The channel of a sender is served under the name that it gives
The client library of Samsung builds the URL of the channel from the base URL
of the service and the name of the channel — the remote control of a
television is `samsung.remote.control`, and the URL is
`ws://<host>/api/v2/channels/<name>`. The receiver therefore serves **every**
name under that prefix and not one fixed path, and it greets the sender with
the event `ms.channel.connect` — the event that the library waits for before
it says anything on the channel.

**How to Verify:** The test `the_channel_under_its_own_name_opens_and_greets_the_sender`
of the crate `engines/smartview` — over a real connection, with the greeting
read out of the socket.
**Pass Condition:** A sender that names its channel finds the channel, and the
first frame that it reads is the event `ms.channel.connect` with the token of
the session.

### FR-109 — The receiver understands the keys of the remote control
The keys of the remote control of a television travel as the method
`ms.remote.control` on the channel, with the key in the parameter
`DataOfCmd` — for example `KEY_POWER`. The receiver reads the method and
hands the key to the platform, which decides what the key does. The messages
of the channel are JSON objects, and a message that the receiver does not
know is ignored — a sender that sends one unknown message has not broken the
channel.

**How to Verify:** The test `the_key_of_the_remote_control_is_found` and the
test `a_method_without_a_key_is_no_key_press` of the crate
`engines/smartview`, and the test
`a_key_press_of_the_remote_control_reaches_the_sender` over a real
connection.
**Pass Condition:** The key of a method `ms.remote.control` is found, a
method without that parameter is not a key press, and the channel stays open
when the receiver reads a key.

---

# 15. Explicitly excluded scope

These are product boundaries, not open defects. Each one is stated in the
user-facing documentation with the same words.

### EXC-001 — FairPlay DRM content
FairPlay Streaming is Apple's DRM for HLS content. It is not implementable
without a licence from Apple and no open-source receiver can decrypt it. The
receiver therefore does not play Apple TV+, Netflix, Disney+ or any other
FairPlay-protected catalogue. The session encryption of the AirPlay mirroring
path (AES-128-CTR) is a different mechanism and is not affected by this
exclusion.

### EXC-002 — Apple Music in-app audio
The in-app audio of the Apple Music app is protected on every AirPlay path.
The workaround is the system audio output of the sender, which the receiver
receives as an ordinary audio stream.

### EXC-003 — HDCP 2.x link protection
HDCP is negotiated inside the WFD handshake and enforced by the hardware of
the sender and of the display. The receiver does not implement the key
exchange. A session that requires HDCP and that the platform cannot protect is
refused cleanly instead of being accepted without protection.

### EXC-004 — Widevine and PlayReady
The DRM systems of the Cast ecosystem belong to the platform DRM framework of
the device, not to the receiver. The receiver does not implement them.

### EXC-005 — Cloud and remote streaming
The receiver is a device on the local network. It does not relay a stream
over the internet and does not offer a remote access.

### EXC-006 — Publishing the receiver to the internet
The receiver is not designed for a port forwarding and is not hardened
against an exposure to the internet.

### EXC-007 — Buffered audio playback
The AirPlay audio mode 103 (buffered) is accepted by the handshake but is not
played back.

### EXC-008 — Several peers on a single-session protocol
AirPlay, Miracast and DLNA serve one peer at a time. Google Cast is the
documented exception.

### EXC-009 — Transcoding
The receiver decodes what it advertised. It does not transcode a format into
another one.

### EXC-010 — A Cast application identifier is not invented
The end-to-end test of the Google Cast path needs an application identifier
that is registered with Google. The project does not ship one and does not
fake one.

### EXC-011 — The picture of the screen of a Galaxy device

**WHAT is excluded:** the transport of the **picture of the screen** of a
Galaxy device to the receiver. The format of that transport is proprietary
and Samsung does not document it; the Smart View SDK realises it as closed
`.jar` libraries. A receiver therefore **finds** the sender, **connects** to
it, **holds** the channel open and **understands** what travels on it — but
no picture of the screen flows over it.

**WHAT is NOT excluded — and is implemented:**

- the **discovery** of the receiver and the **announcement** of it,
- the **description document** and the **API** below `/api/v2/`,
- the **handshake** of the sender,
- the **channel** itself under the name that the sender gives it —
  `ws://<host>/api/v2/channels/<name>`, which is the URL that the client
  library of Samsung builds,
- the **events** of the channel — `ms.channel.connect`, `ms.channel.ready`,
  `ms.channel.unauthorized` and the heartbeat `__ping` — as the JSON
  objects that the client library expects,
- the **keys of the remote control** — the method `ms.remote.control` with
  its parameter `DataOfCmd`, for example `KEY_POWER`.

**WHY the exclusion is narrower than it was.** The first version of this
document excluded "the message format of the channel". That was too broad:
the messages of the channel are JSON objects with an `event` or a `method`
and a `data` or a `params`, and the names of the events and the methods are
observable on the network and are the ones the client library of Samsung
uses. A proprietary format is not an unimplementable one — the Miracast
receiver, the Google Cast receiver and the AirPlay receiver of this project
are all implementations of formats that nobody documents either. The
exclusion therefore names **the one thing** that is not implemented: the
picture of the screen.

This is a limit of the format, not a DRM: Smart View is an open feature of
the Galaxy devices, and nothing in this project invents a restriction that
the protocol does not have. The limit stays until the format is documented
by Samsung, until another implementation exists under a licence that allows
the port, or until the observations of the transport are complete enough to
implement it as a clean-room implementation from the documentation of the
observations.

---

# 16. Non-functional requirements

## 16.1 Performance and resources

### NFR-PERF-001 — Time to the first picture
The receiver presents the first decoded frame to the output surface within the
budget `latency.first_frame_ms` of
[`config/performance-budgets.toml`](config/performance-budgets.toml).

**How to Verify:** Measure the time between the acceptance of the peer and the
first frame at the output surface, with a sender and a receiver on the same
wired network, over at least ten sessions. The measurement belongs into the
acceptance record of the milestone that covers the protocol.
**Pass Condition:** Every measured session is at most the configured budget.

### NFR-PERF-002 — Sustained frame rate
The receiver sustains at least the frame rate `throughput.minimum_fps` of
[`config/performance-budgets.toml`](config/performance-budgets.toml) during a
mirroring session of at least the configured measurement window.

**How to Verify:** Count the delivered frames over the measurement window and
divide by the length of the window. Use the counters of the core
(`StreamCounters::snapshot`), not a stopwatch on the screen.
**Pass Condition:** The measured rate is at least the configured minimum in
every session, and the drop counter stays below the frame counter.

### NFR-PERF-003 — Resident memory
The resident memory of the receiver process stays at or below the budget
`resources.max_memory_mb` of
[`config/performance-budgets.toml`](config/performance-budgets.toml) during a
session of at least the configured measurement window.

**How to Verify:** Read the resident memory of the process while a session
runs and repeat the measurement with a long session. The result belongs into
the acceptance record together with the command that produced it.
**Pass Condition:** The measured resident memory never exceeds the configured
budget, and it does not grow with the length of the session.

### NFR-PERF-004 — CPU usage
The sustained CPU usage of the receiver process stays at or below the budget
`resources.max_cpu_percent` of
[`config/performance-budgets.toml`](config/performance-budgets.toml) during
a session of at least the configured measurement window.

**How to Verify:** Measure the CPU time of the process over the measurement
window and divide it by the wall-clock time of the window.
**Pass Condition:** The measured usage never exceeds the configured budget
over the whole window.

### NFR-PERF-005 — Audio and video synchronisation
The deviation between the presentation timestamps of the audio and the video
stays at or below the budget `latency.audio_video_sync_ms` of
[`config/performance-budgets.toml`](config/performance-budgets.toml).

**How to Verify:** Record the presentation timestamps of the received audio
and video packets over at least one minute and compute the deviation.
**Pass Condition:** The measured deviation stays inside the configured budget
over the whole recording.

### NFR-RES-001 — Bounded queues
A queue of the receiver is bounded by an item count **and** by a byte budget.
A peer that sends faster than the sink consumes must not be able to grow the
memory of the receiver.

**How to Verify:** Feed a session with frames of the maximum accepted size
until the sink is saturated and watch the resident memory and the drop
counter.
**Pass Condition:** The resident memory stays inside the budget of
`NFR-PERF-003`, the drop counter rises, and the receiver does not stall.

### NFR-RES-002 — Every read has a deadline
No blocking read waits forever. A peer that connects and stays silent must
not hold a connection slot.

**How to Verify:** The idle timeout key `latency.idle_timeout_ms` of
[`config/performance-budgets.toml`](config/performance-budgets.toml) and the
negative tests of the engines.
**Pass Condition:** A silent peer is disconnected within the configured
timeout and the slot is free again.

## 16.2 Security

### NFR-SEC-001 — Every network input is untrusted
Every byte that arrives over a socket is untrusted. A length limit, a
recursion limit and a timeout exist on every path that reads from the
network.

**How to Verify:** The negative test suite of the engines and the format
crates: a 31-zero-bit SPS, a property list with a cyclic object graph, a
SOAP body with a `DOCTYPE`, a GENA callback that points at
`169.254.169.254`, a CastV2 frame with an oversized length, a WFD information
element with a broken length.
**Pass Condition:** Every one of these inputs is rejected with a typed error,
and no allocation follows a length field that was not validated.

### NFR-SEC-002 — The parsers reject, they never repair
A malformed input is an error. A parser never guesses what the sender meant.

**How to Verify:** The negative tests of the format crates and of the SDP
parser.
**Pass Condition:** Every malformed input that the test suite offers is
rejected, and none of them produces a partially corrected result.

### NFR-SEC-003 — The response time of a hostile peer stays bounded
A peer that sends a flood of valid-looking messages must not be able to make
the receiver work without limit.

**How to Verify:** The bounded queues of `NFR-RES-001` and the bounded job
executor.
**Pass Condition:** The resident memory and the CPU usage stay inside the
budgets of `NFR-PERF-003` and `NFR-PERF-004` under the flood.

### NFR-SEC-004 — The access control of the local network is explicit
Whether a PIN is required is a decision of the user and is visible in the
interface. The default is documented in
[`config/app-defaults.toml`](config/app-defaults.toml).

**How to Verify:** The `require_pin` key per protocol and the pairing
settings in [`config/security-policy.toml`](config/security-policy.toml).
**Pass Condition:** The state of the access control is visible in the shell
and in the dashboard, and the default of the configuration is the documented
one.

### NFR-SEC-005 — No unsafe code
The workspace denies `unsafe` for the whole workspace. A crate that needs it
must justify the exception in an ADR and override the lint locally, never
globally.

**How to Verify:** `cargo clippy --workspace --all-targets -- -D warnings`
together with the `unsafe_code = "deny"` entry in the workspace lints.
**Pass Condition:** The lint is green and no crate carries a global override.

### NFR-SEC-006 — No panicking path in production code
`unwrap()` and `expect()` are denied for the whole workspace. A value that is
missing is an error, not a panic.

**How to Verify:** The workspace lints and `cargo clippy --workspace
--all-targets -- -D warnings`.
**Pass Condition:** The lint is green, and every error path of the bridge
returns a typed error instead of panicking.

## 16.3 Privacy

### NFR-PRIV-001 — Zero telemetry
The receiver collects nothing and sends nothing to any host outside the local
subnet.

**How to Verify:** The search for telemetry clients and the
`privacy` section of [`config/security-policy.toml`](config/security-policy.toml).
**Pass Condition:** No telemetry client exists in the source, and the only
outbound connections are to peers of the local subnet.

### NFR-PRIV-002 — The logs are masked
Sender names and IP addresses are masked in every log line, and the content
of a stream never reaches the log.

**How to Verify:** The masking helper of the core and the negative test that
searches a rendered log line for a complete name or address.
**Pass Condition:** No log line contains a complete sender name or a complete
IP address.

### NFR-PRIV-003 — The pairing material is protected
The pairing store is written with owner-only permissions and its content
never leaves the device.

**How to Verify:** The `pairing.store_permissions` key in
[`config/security-policy.toml`](config/security-policy.toml) and the test that
reads the permissions of the written file.
**Pass Condition:** The file mode of the pairing store is the configured one
on the platform.

### NFR-PRIV-004 — No account, no identifier, no profile
The receiver does not ask for a name, a mail address or any other personal
data, and it does not build a profile of the sender.

**How to Verify:** The settings surface of the shells and the storage of the
shells.
**Pass Condition:** The only user input is the display name of the receiver
and the settings of the protocols.

## 16.4 Interoperability

### NFR-INT-001 — The protocols are tested against real senders
Every protocol is tested with the senders that exist in the market: an Apple
device for AirPlay 2, Windows for Miracast and for the DLNA media renderer,
Chromecast and the Cast SDK for Google Cast, VLC for DLNA.

**How to Verify:** The device test plan in `docs/guides/` and the acceptance
record of the milestone that covers the protocol.
**Pass Condition:** Every sender of the matrix is marked as tested, and a
sender that could not be tested is named with the reason.

### NFR-INT-002 — The receiver works without a sender of the same vendor
No protocol path depends on a component of the sender vendor beyond the
protocol itself. The Cast path in particular does not depend on the Google
Cast SDK being present on the device.

**How to Verify:** The dependency list of the workspace and the fallback
behaviour of the Cast engine.
**Pass Condition:** The receiver starts and advertises on a device without the
Cast SDK, and the Cast service is the only one that is missing.

### NFR-INT-003 — The formats are interoperable
A property list, an H.264 bitstream and an AAC frame that the receiver writes
are readable by the corresponding standard implementation.

**How to Verify:** The corpus tests of the format crates against a reference
decoder.
**Pass Condition:** The reference decoder reads the result of the encoder
without a complaint.

### NFR-INT-004 — The protocol behaviour is documented
The behaviour of a protocol is described in the architecture document and in
the guide of the protocol, including the parts that differ from the sender's
expectation.

**How to Verify:** [`ARCHITECTURE.md`](ARCHITECTURE.md) and the guides in
`docs/guides/`.
**Pass Condition:** Every protocol has a data-flow description, and every
documented deviation is also visible in the acceptance record.

## 16.5 Release and quality

### NFR-REL-001 — The gate is green
The continuous integration runs the format check, the lint check, the test
suite, the documentation check and the licence check, and it is red when a
rule is violated.

**How to Verify:** The workflow of the continuous integration and the scripts
in `scripts/`. A rule has to be broken deliberately to see the gate turn red,
and the proof belongs into the acceptance record.
**Pass Condition:** The gate is green on the main branch and turns red when a
rule is violated.

### NFR-REL-002 — The acceptance record carries evidence
Every statement in an acceptance record names the test or the command that
produced it.

**How to Verify:** The acceptance template in
[`docs/acceptance/ACCEPTANCE_TEMPLATE.md`](docs/acceptance/ACCEPTANCE_TEMPLATE.md)
and the acceptance records.
**Pass Condition:** No acceptance statement says "implemented" without a test
name or a command output next to it.

### NFR-REL-003 — A stub never satisfies an acceptance
A function that returns a fixed value, an empty implementation or a mocked
bridge does not satisfy a requirement.

**How to Verify:** The acceptance record and the review of the delivered
code.
**Pass Condition:** Every claimed capability is demonstrated by a test that
exercises the real implementation.

### NFR-DOC-001 — The documentation is part of the product
A change that makes a document wrong is an incomplete change. The
documentation is updated in the same change.

**How to Verify:** The same-change rule of
[`docs/DOCUMENTATION_POLICY.md`](docs/DOCUMENTATION_POLICY.md) and the
review of the pull request.
**Pass Condition:** A change that touches a behaviour, a limit, a default or
a structure carries the matching documentation change.

### NFR-DOC-002 — Every link in the documentation resolves
Every internal link of the documentation points at a file that exists.

**How to Verify:** `bash scripts/check-docs-links.sh`.
**Pass Condition:** The script is green.

### NFR-DOC-003 — The numbers exist only once
Every number and every rule of the documentation tree is written down exactly
once. The other documents link to it.

The owner of a budget is named in the table of
[`config/README.md`](config/README.md): either the typed Rust model (for a
mirrored budget) or the TOML file of the configuration (for a budget that the
file owns alone).

**How to Verify:** `bash scripts/check-docs-links.sh` compares the mirrored
budgets with the typed Rust model, and it looks for a second copy of every
budget that a configuration file owns alone — a copy carries a unit, and that
is what the search looks for.
**Pass Condition:** The check is green, it reports the count of both groups,
and no budget value with a unit appears in two documents.

### NFR-PORT-001 — The supported platforms are named
The supported platforms and their minimum versions are named in one place.

**How to Verify:** The `platforms` section of
[`project/project.yaml`](project/project.yaml).
**Pass Condition:** A platform that is not listed there is not claimed
anywhere else in the documentation.
