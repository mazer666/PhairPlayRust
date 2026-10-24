# PhairPlayRust Architecture

**Document version:** 0.1
**Status:** Accepted architecture baseline
**Change rule:** Material changes require an ADR (`docs/adr/`) and, if
requirements are affected, a Change Request (`docs/cr/`).

This document describes the **accepted architecture**. The implementation
status of each part is tracked in [`ROADMAP.md`](ROADMAP.md) and in
[`project/project.yaml`](project/project.yaml). The **interfaces** between the
crates are fixed separately in
[`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md), which is
binding for every crate.

---

## 1. Architectural goals

The architecture must support all of these at the same time:

1. four receiver protocols on one shared core
2. a protocol engine that is testable without a network and without a decoder
3. native macOS and Android shells that share the logic
4. a local dashboard that reads the same state as the native shells
5. untrusted network input that cannot exhaust the memory of the receiver
6. a receiver that never talks to anything but the local network
7. a port that is bounded by an item count **and** by a byte budget
8. errors that reach the native side as a stable numeric code
9. a design that can take a fifth protocol without touching the core
10. no hidden lock-in on a single decoder or on a single platform

---

## 2. The layer model

```text
┌──────────────────────────────────────────────────────────────────────┐
│  apps/android   apps/macos   apps/web                               │
│  Kotlin/Compose SwiftUI       Browser (static assets)                │
│  ────────────   ────────      ──────────────                         │
│  native shell   native shell   local control panel                    │
└───────────┬───────────┬──────────────────────┬───────────────────────┘
            │           │                      │
            │  UniFFI   │  UniFFI              │  HTTP / WebSocket
            │           │                      │  (loopback only)
┌───────────▼───────────▼──────────────────────▼───────────────────────┐
│  platform/phairplay-ffi            apps/web (Rust backend)          │
│  the only door from the native side into the core                    │
└───────────┬──────────────────────────────────────────────────────────┘
            │
┌───────────▼──────────────────────────────────────────────────────────┐
│  core/phairplay-core                                                  │
│  protocol trait · session registry · media sinks · capabilities      │
│  configuration · identity · metrics · error hierarchy · job executor │
└───────────┬──────────────────────────────────────────────────────────┘
            │
┌───────────▼──────────────────────────────────────────────────────────┐
│  engines/          airplay · miracast · cast · dlna · discovery     │
│  the protocol state machines                                          │
└───────────┬──────────────────────────────────────────────────────────┘
            │
┌───────────▼──────────────────────────────────────────────────────────┐
│  formats/          plist · h264 · aac · alac                         │
│  bytes in, bytes out. No I/O, no sockets, no platform calls.        │
└──────────────────────────────────────────────────────────────────────┘
```

Four rules are never broken:

1. **The shell never talks to an engine.** Every call goes through the bridge
   in `platform/phairplay-ffi` or through the local web backend.
2. **The core never knows that a protocol exists.** It knows the trait
   `ProtocolEngine`, the events and the sinks — nothing else.
3. **An engine never decodes.** It delivers *encoded* frames in a
   platform-neutral description; the platform decodes.
4. **A format crate never knows that a protocol exists.** It transforms bytes
   into bytes.

The dependency direction is therefore strict:

```text
apps/*        → platform/*, core/*
platform/*    → core/*
engines/*     → core/*, formats/*
formats/*     → (nothing inside the workspace)
core/*        → (nothing inside the workspace)
```

An engine may use the core and the format crates. The core never knows an
engine exists. A format crate never knows a protocol exists. A `formats/*`
crate must not depend on an `engines/*` crate, an `engines/*` crate must not
depend on a `platform/*` or an `apps/*` crate, and `core/*` must not depend on
any other crate of the workspace.

---

## 3. The crates

| Crate | Package name | Depends on (workspace crates) | Owns |
|---|---|---|---|
| `core/phairplay-core` | `phairplay-core` | — | The shared vocabulary |
| `platform/phairplay-ffi` | `phairplay-ffi` | `core` | The bridge and the data transfer objects |
| `engines/discovery` | `phairplay-discovery` | `core` | DNS-SD advertisement, SSDP browsing |
| `engines/airplay` | `phairplay-airplay` | `core`, `formats/{plist,h264,aac,alac}` | The AirPlay 2 receiver |
| `engines/miracast` | `phairplay-miracast` | `core`, `formats/h264` | The Miracast sink |
| `engines/cast` | `phairplay-cast` | `core`, `formats/plist` | The Google Cast receiver |
| `engines/dlna` | `phairplay-dlna` | `core` | The UPnP / DLNA media renderer |
| `formats/plist` | `phairplay-plist` | — | Apple property lists, XML and binary |
| `formats/h264` | `phairplay-h264` | — | SPS/PPS, NAL units, RTP FU-A |
| `formats/aac` | `phairplay-aac` | — | ADTS, AAC-ELD framing |
| `formats/alac` | `phairplay-alac` | — | The Apple Lossless decoder |

The dependency graph of the workspace:

```text
                         ┌──────────────────────┐
                         │  platform/phairplay-ffi│
                         └───────────┬──────────┘
                                     │
                         ┌───────────▼──────────┐
                         │  core/phairplay-core │
                         └───────────▲──────────┘
                                     │
        ┌──────────────┬─────────────┼─────────────┬──────────────┐
        │              │             │             │              │
 ┌──────▼─────┐ ┌──────▼──────┐ ┌────▼─────┐ ┌─────▼─────┐ ┌──────▼──────┐
 │ engines/    │ │ engines/    │ │ engines/ │ │ engines/  │ │ engines/    │
 │ discovery  │ │ airplay     │ │ miracast│ │ cast      │ │ dlna        │
 └──────┬─────┘ └──────┬──────┘ └────┬─────┘ └─────┬─────┘ └──────┬──────┘
        │              │             │             │              │
        │      ┌───────┴──────┬──────┴─────┬───────┘              │
        │      │              │            │                      │
        │  ┌───▼───┐ ┌────────▼───┐ ┌──────▼──┐                   │
        │  │plist  │ │h264        │ │aac      │                   │
        │  └───────┘ │            │ │alac     │                   │
        │            └────────────┘ └─────────┘                   │
        │                                                            │
        └──────────────► formats/* (no workspace dependency) ◄───────┘
```

The four format crates have **no** dependency on a crate of the workspace. That
is what makes them reusable and trivially testable.

The table names the crates of the workspace. The external crates — the
serialisation, the async runtime, the cryptography, the discovery and the
bridge — come from the dependency table of the root `Cargo.toml` and do not
change the direction rule.

### 3.1 The shared vocabulary

`core/phairplay-core` owns exactly ten modules. Every other crate of the
workspace speaks in their types.

| Module | Responsibility |
|---|---|
| `protocol` | The `ProtocolEngine` trait, the protocol kinds, the state machine, the engine context |
| `session` | The session identifier, the session events, the session registry |
| `media` | The encoded frame types, the sink traits, the pixel formats |
| `capabilities` | The video, audio and passthrough codecs the device can decode |
| `config` | The typed configuration model and its validation |
| `identity` | The advertised identity and the sanitising of the service name |
| `metrics` | The atomic counters of a session |
| `error` | The closed error hierarchy and the stable numeric codes |
| `jobs` | The bounded job executor and the periodic job helper |

`build_info` compiles the version, the target triple and the compiler version
into the binary, so that a bug report can name the exact build.

### 3.2 The engine contract

Every receiver is the same shape: advertise a service, accept a peer,
negotiate, stream, tear down. That shape exists exactly once, as a trait:

```rust
#[async_trait::async_trait]
pub trait ProtocolEngine: Send + Sync + std::fmt::Debug {
    fn kind(&self) -> ProtocolKind;
    fn status(&self) -> ProtocolStatus;
    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError>;
    async fn stop(&self) -> Result<(), EngineError>;
}
```

`EngineContext` hands the engine everything it is allowed to touch:

```rust
pub struct EngineContext {
    pub identity: Arc<DeviceIdentity>,
    pub capabilities: Arc<MediaCapabilities>,
    pub video_sink: Arc<dyn VideoSink>,
    pub audio_sink: Arc<dyn AudioSink>,
    pub events: tokio::sync::mpsc::Sender<SessionEvent>,
}
```

An engine therefore never queries the platform for a capability, never opens a
global singleton and never calls another engine. It receives what it needs,
which makes it testable without a network and without a decoder.

### 3.3 The media path

```text
sender ──RTP/HTTP──► engine ──encoded frame──► VideoSink / AudioSink ──► decoder
                        │                              │
                        │                              └─ bounded: item count + byte budget
                        └─ never decodes
```

`EncodedVideoFrame` and `EncodedAudioFrame` carry a sequence number, a
presentation timestamp in microseconds and the payload in an `Arc<Vec<u8>>`.
The payload is shared, not copied, between the engine and the sink.

A sink that cannot keep up returns `MediaError::QueueFull`. It never grows.
The age of a frame is bounded by `media::MAX_FRAME_AGE`; a frame that is
older than that is stale and is dropped rather than presented.

The decoder itself is a **platform** concern: MediaCodec on Android,
VideoToolbox and AudioToolbox on Apple platforms, a software decoder in the
browser. The core knows nothing about it.

---

## 4. The data flow of a session

The following flow is common to all four protocols; the differences are in
the handshake and in the transport.

```text
  1. discovery     the receiver registers its service (DNS-SD) or answers
                   (SSDP) and waits
  2. accept        a peer connects; the engine checks the limits, the state
                   and the access control
  3. negotiate     the handshake of the protocol: RTSP, WFD, CastV2 or SOAP
  4. configure     the engine announces the negotiated format to the sink
                   (VideoSink::set_format, AudioSink::set_format)
  5. stream        the engine delivers encoded frames into the sink
  6. report        the engine emits the session events; the registry folds
                   them; the shells and the dashboard display them
  7. tear down     the peer leaves or the receiver stops; the engine emits
                   `Ended`, releases the sockets and returns to `Advertising`
```

### 4.1 AirPlay 2

```text
  iPhone / Mac
       │
       │  DNS-SD: _airplay._tcp  +  _raop._tcp
       ▼
  ┌─────────────┐   OPTIONS / ANNOUNCE / SETUP / RECORD / TEARDOWN
  │  RTSP       │◄──────────────────────────────────────────┐
  │  control    │                                           │
  └──────┬──────┘                                           │
         │ SDP: video (H.264 SPS/PPS), audio (ALAC/AAC),     │
         │      AES key material                            │
         ▼                                                  │
  ┌─────────────┐   RTP (interleaved)   ┌──────────────┐    │
  │  frames     │──────────────────────►│ media sinks  │    │
  └──────┬──────┘                       └──────────────┘    │
         │                                                   │
         │  DACP (reverse remote), /photo, /playback-info    │
         ▼                                                   │
  ┌─────────────┐                                           │
  │  metadata   │──── SessionEvent ───────────────────────────┘
  └─────────────┘
```

Details:

- the service is registered under **two** service types, because a receiver
  that omits `_raop._tcp` is invisible to some senders;
- the session key of the FairPlay handshake decrypts the stream; this is the
  session encryption, **not** the FairPlay DRM of a licensed catalogue (see
  the exclusion `EXC-001` of [`REQUIREMENTS.md`](REQUIREMENTS.md));
- the timing of the sender is used to align audio and video;
- a second peer is refused with the status code of the protocol, not queued.

### 4.2 Miracast / Wi-Fi Display

```text
  Windows / Android sender
       │
       │  Wi-Fi Display: information elements, sink discovery
       ▼
  ┌─────────────┐   RTSP (WFD control plane)
  │  WFD        │◄──────────────────────────────────────────┐
  │  control    │                                           │
  └──────┬──────┘                                           │
         │  MPEG-TS / RTP: H.264 video, LPCM audio          │
         ▼                                                  │
  ┌─────────────┐                       ┌──────────────┐    │
  │  frames     │──────────────────────►│ media sinks  │    │
  └─────────────┘                       └──────────────┘    │
                                                            │
  The platform must provide the WFD permission. ────────────┘
```

Details:

- the advertisement contains the baseline codecs: H.264 for the video, linear
  PCM for the audio;
- the platform must offer the WFD feature. A device without it reports the
  missing capability and does not advertise the service — the receiver cannot
  work around a missing system permission (see `FR-035`);
- a session that requires HDCP and that the platform cannot protect is
  refused cleanly (`EXC-003`).

### 4.3 Google Cast

```text
  Chrome / Chromecast / Cast SDK
       │
       │  DNS-SD: _googlecast._tcp
       ▼
  ┌─────────────┐   TLS + CastV2 framing
  │  CastV2     │◄──────────────────────────────────────────┐
  │  control    │   CONNECT, PING/PONG, application id         │
  └──────┬──────┘                                           │
         │  media channels: audio, video, multiplexed         │
         ▼                                                  │
  ┌─────────────┐                       ┌──────────────┐    │
  │  frames     │──────────────────────►│ media sinks  │    │
  └─────────────┘                       └──────────────┘    │
                                                            │
  The application id is configured, never invented. ─────────┘
```

Details:

- the control plane of Cast is TLS; the AirPlay control plane is plaintext
  RTSP. That difference is a property of the protocols, not a policy of the
  project (see `config/security-policy.toml`, section `[transport]`);
- the Cast protocol allows several senders at the same time. That exception
  lives in exactly one place: the `session.multi_peer_protocols` list in
  [`config/app-defaults.toml`](config/app-defaults.toml);
- the end-to-end test of this path needs an application identifier that is
  registered with Google. The project does not ship one and does not fake one
  (`EXC-010`).

### 4.4 DLNA / UPnP MediaRenderer

```text
  Windows "Cast to Device" / VLC / UPnP controller
       │
       │  SSDP: M-SEARCH  →  NOTIFY + ST
       │  HTTP: /description.xml, SCPD documents
       ▼
  ┌─────────────┐   SOAP: ConnectionManager, AVTransport,
  │  SOAP       │          RenderingControl
  │  dispatcher │◄──────────────────────────────────────────┐
  └──────┬──────┘                                           │
         │  GENA: event notification over the callback URL   │
         ▼                                                  │
  ┌─────────────┐                       ┌──────────────┐    │
  │  media      │──────────────────────►│ media sinks  │    │
  └─────────────┘                       └──────────────┘    │
                                                            │
  The callback URL is checked against the SSRF guard. ───────┘
```

Details:

- the device description and the service descriptions are served over HTTP;
  the control is SOAP over XML, the events are GENA notifications;
- the callback URL of a GENA subscription is an address the receiver
  afterwards connects to. That is a server-side request forgery vector, and
  the guard of [`config/security-policy.toml`](config/security-policy.toml)
  accepts only private addresses of the local subnet;
- the XML rules forbid a `DOCTYPE` and external entities;
- the renderer serves audio and pictures. It does not render a live video
  stream (`FR-050`).

---

## 5. The state machine of an engine

```text
  Idle ──start()──► Starting ──► Advertising ──peer──► Streaming
                       │                ▲                  │
                       │                │                  │
                       ▼                │                  ▼
                    Failed          stop()            Stopping
                                        ▲                  │
                                        └──────────────────┘
```

- `start()` moves the engine from `Idle` through `Starting` to `Advertising`.
- `stop()` moves it through `Stopping` to `Stopped`.
- A failure during the start moves it to `Failed`.
- Only `Advertising` and `Streaming` accept a peer
  (`ProtocolState::accepts_peers`).
- `status()` is callable at any moment, also from another thread while a
  transition is in flight.

A protocol engine serves one peer at a time — AirPlay, Miracast and DLNA.
Google Cast is the documented exception. The rule is data, not code: the
`session.max_concurrent_peers` and `session.multi_peer_protocols` entries in
[`config/app-defaults.toml`](config/app-defaults.toml).

---

## 6. The threading and concurrency model

### 6.1 The runtime

The receiver runs on a Tokio multi-threaded runtime. The workspace enables the
features `rt-multi-thread`, `net`, `io-util`, `sync`, `time` and `macros`.

There is exactly one runtime per process. The native shells start it once;
the web backend starts it once. An engine never creates a runtime of its own.

### 6.2 Who owns what

| Object | Owner | Access |
|---|---|---|
| The engine state | The engine | `status()` reads it under an internal lock; `start`/`stop` write it |
| The session registry | The event pump | One owner folds the events; the shells read the snapshot |
| The counters of a session | The streaming task | Atomics, `Relaxed` ordering — they count, they do not synchronise |
| The media sink | The platform | `Arc<dyn VideoSink>`; object-safe, `&self`, `Send + Sync` |
| The event channel | The engine | `tokio::sync::mpsc::Sender<SessionEvent>` |
| The job queue | The job executor | A bounded channel; a saturated pool rejects, it does not queue without limit |

### 6.3 The rules

1. **An engine is a state machine, not a set of threads.** The socket
   handling runs in the tasks that the engine spawns; the state lives in the
   engine.
2. **The registry has exactly one writer.** The events arrive on a channel;
   one task folds them into the registry. Two writers would make the
   "per protocol" view of the registry ambiguous.
3. **The counters are atomic, not locked.** A counter that is read on the
   interface thread and written on the streaming task must not block the
   streaming task. The counters count events; they do not carry a state.
4. **A sink is shared, not copied.** The sink trait takes `&self`, so a sink
   can live in a dedicated decoder task and be reached from several engines
   at the same time.
5. **No unbounded queue exists anywhere.** The frames of a sink, the packets
   of an audio sink, the jobs of the executor and the messages of an engine
   are bounded. A queue that only knows an item count can be filled with
   hundreds of megabytes by a peer that sends maximum-size frames.
6. **No blocking work happens on the runtime.** A parser is pure, a socket
   read is asynchronous, the file system work of the job executor happens in
   the job, not in the engine.
7. **The timeout is not optional.** Every read that waits for a peer has a
   deadline (`latency.idle_timeout_ms`). A peer that connects and says
   nothing must not hold a connection slot.

### 6.4 The job executor

The core owns a bounded pool of workers (`core/phairplay-core/src/jobs/`).
The work that must not run inside the event loop of a protocol — pruning the
pairing store, rotating the log, re-advertising after a network change,
reaping the expired sessions — is registered as a job.

The executor never blocks the caller: submitting to a saturated pool is a
rejection with a typed error, not a wait. The number of workers and the
length of the queue are the `jobs` keys of
[`config/performance-budgets.toml`](config/performance-budgets.toml).

A job that fails is counted in `ExecutorStats::failed`. A job that does not
exist is not executed. There is no job kind without a real operation behind
it.

---

## 7. The error flow

### 7.1 The hierarchy

```text
CoreError
├── Config(ConfigError)        NotFound · Parse · Invalid{field,reason} · Io
├── Engine(EngineError)        Start · NotRunning · AlreadyRunning
│                              Protocol{reason} · LimitExceeded{reason} · Stopped
├── Media(MediaError)          QueueFull{capacity} · Closed · Rejected · Unsupported
├── Session(SessionError)      NoActiveSession · Duplicate · AlreadyActive
│                              NotFound · Ended
├── Network(NetworkError)      Bind{address,reason} · Io · NoInterface
│                              PeerOutsideSubnet
├── Capability(CapabilityError) Query · Inconsistent
└── Aborted(&'static str)
```

One hierarchy for the whole workspace. An engine does not introduce a parallel
error enum — it uses `EngineError::Protocol { reason }` for a peer that
misbehaves, `EngineError::LimitExceeded { reason }` for a limit that was
reached, and `NetworkError::PeerOutsideSubnet` for a peer from the outside.

### 7.2 The classification

`ErrorKind::classify` maps every error onto one of nine kinds:

| Kind | Raised for |
|---|---|
| `InvalidInput` | A configuration that is wrong, a payload the decoder refused |
| `LimitExceeded` | A configured limit that was reached, a queue that is full, a bind that failed |
| `ProtocolViolation` | A peer that violated the protocol |
| `ResourceExhausted` | A queue that is full, a socket that could not be bound |
| `Timeout` | An operation that was aborted because the receiver is shutting down |
| `Unsupported` | A format that the sink cannot decode |
| `InvalidState` | A call in a state in which it is not allowed |
| `Io` | A failed I/O operation |
| `Internal` | Everything that has no more specific classification |

### 7.3 The bridge

```text
engine error  →  CoreError  →  ErrorKind::classify  →  ErrorCode  →  native shell
```

The numeric code of an `ErrorCode` is part of the public contract of the
bridge. It is defined in `core/phairplay-core/src/error.rs` and it is not
renumbered without a major version bump. The shells branch on the code, never
on the text of a message.

`describe` renders an error and its whole cause chain into a single line, so
that a log stays readable and the cause does not get lost.

**A panic never crosses the bridge.** Every failure of the bridge is a typed
error with a code and a message.

---

## 8. The event flow

```text
  engine                    event pump                 shells / dashboard
  ──────                    ──────────                 ──────────────────
  SessionEvent::Started  ──►  registry.apply()  ──►  SessionSnapshot
  SessionEvent::Ended    ──►  registry.apply()  ──►  the session disappears
  VideoFormatChanged     ──►  registry.apply()  ──►  the format of the card
  AudioFormatChanged     ──►  registry.apply()  ──►  the format of the card
  PlaybackStateChanged   ──►  (metadata only)   ──►  the transport state
  MetadataChanged        ──►  (metadata only)   ──►  title, artist, album
  ArtworkChanged         ──►  (metadata only)   ──►  the cover
```

The engine is the only place that knows the truth about the wire, so the
engine reports. The core interprets, the shells display.

The registry keeps one session per protocol. A late `Ended` event that belongs
to an older session does not clear a newer one — the registry identifies the
session by its identifier and not by its position.

The metadata events do not change the snapshot of the registry. They are
forwarded to the display layer, which owns the presentation state.

---

## 9. The configuration architecture

```text
  config/app-defaults.toml  ─┐
  config/performance-*.toml ─┼─►  AppConfig  ─►  the engines read their limits
  config/security-policy.toml ┘        │
                                      └──►  the bridge and the shells
```

The configuration is loaded once into one typed model. The engines read their
limits from that model; they never hard-code a limit. The documentation of
the values lives in [`config/`](config/README.md); the typed mirror lives in
`core/phairplay-core/src/config.rs`. A change to a value touches both in the
same change.

`AppConfig::validate()` rejects a value that is syntactically valid but
semantically wrong. A missing key is never an error, because every field has
a default; an unknown key is always an error, because a typo in a
configuration must not be silently ignored.

---

## 10. The security architecture

The threat model is short: **the attacker is normally a device in the same
local network.** The policy lives in
[`config/security-policy.toml`](config/security-policy.toml).

| Surface | Guard |
|---|---|
| Every socket read | A length limit and a deadline |
| Every message | A maximum message size, a maximum header count, a maximum header length |
| The SDP parser | A rejected exponential-Golomb reader, a bounded parameter set |
| The property list parser | A depth limit, a cycle detection, a validated length before an allocation |
| The SOAP parser | No `DOCTYPE`, no external entity, no network entity |
| The GENA callback | Only private addresses of the local subnet; the link-local address of the cloud metadata service, the loopback address and the unspecified address are refused |
| The queues | An item count **and** a byte budget |
| The logs | The sender name and the IP address are masked; the content of a stream never reaches the log |
| The outbound connections | Only to the local subnet; the dashboard binds on the loopback interface |

The parsers reject; they never repair. A malformed message is an error, not
something to guess at.

---

## 11. The platform bridge

The bridge in `platform/phairplay-ffi` is the only door from the native side
into the core. Its rules:

1. the exported surface is platform-neutral. A platform difference is
   expressed by a different implementation, never by a new exported function
   per platform;
2. the data transfer objects mirror the types of the core; they do not invent
   a second model;
3. every error arrives as a code and a message;
4. the generated bindings are never edited by hand. They are regenerated from
   the Rust sources;
5. the round trip is proven by a test: a value that goes in comes out
   unchanged.

---

## 12. The testing architecture

| Level | What it proves | Where |
|---|---|---|
| Unit | A function behaves as its contract says | next to the function, in the crate |
| Negative | A hardening measure actually rejects the attack | next to the function, in the crate |
| Integration | Two crates work together | `tests/` of the crate |
| Boundary | The value survives the bridge unchanged | `platform/phairplay-ffi` |
| Contract | The dependency direction and the limits are respected | `scripts/` |
| Device | The protocol really works with a real sender | the device test plan in `docs/guides/` |

Three rules of the build contract bind every level:

1. **Every public function has a test.**
2. **Every hardening measure has a negative test** that really exercises the
   attack: a property list with a cycle, the SPS overflow vector of
   `FR-056`, a SOAP body with a `DOCTYPE`, a GENA callback that points at
   `169.254.169.254`.
3. **No stub satisfies an acceptance.** A function that returns a fixed value
   is a lie that the acceptance process will catch.

---

## 13. The extension points

| To add … | Touch … | Never … |
|---|---|---|
| A fifth protocol | A new crate in `engines/`, an entry in `ProtocolKind`, a service type in the discovery engine | the core |
| A new codec | A new crate in `formats/`, an entry in `VideoCodec` or `AudioCodec` | the engine |
| A new platform | A shell in `apps/`, the platform traits in the core | the engines |
| A new setting | `config/*.toml` and the typed model in the core | a literal in a shell |
| A new error kind | The hierarchy in `core/phairplay-core/src/error.rs` | a parallel enum in an engine |
| A new background job | A job in the executor of the core | a new thread pool |

---

## 14. The documentation architecture

| Document | Owns |
|---|---|
| [`README.md`](README.md) | The entry point and the orientation |
| [`REQUIREMENTS.md`](REQUIREMENTS.md) | The behaviour, the identifiers, the verification |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | The structure, the boundaries, the flows |
| [`ROADMAP.md`](ROADMAP.md) | The milestones, the gates, the state |
| [`QUALITY.md`](QUALITY.md) | The acceptance model and the severity levels |
| [`AGENTS.md`](AGENTS.md) | The rules for the agents that work on the project |
| [`PROJECT_MANAGEMENT.md`](PROJECT_MANAGEMENT.md) | The governance, the change requests, the archive |
| [`WHERE_TO_CHANGE.md`](WHERE_TO_CHANGE.md) | The navigation for a newcomer |
| [`GLOSSARY.md`](GLOSSARY.md) | The terms of the protocols |
| [`config/`](config/README.md) | The numbers, the defaults, the policy |
| [`project/project.yaml`](project/project.yaml) | The machine-readable state |
| [`docs/`](docs/README.md) | The detailed living documentation |
