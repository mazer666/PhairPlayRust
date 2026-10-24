# PhairPlayRust — Build Contract

> **Status:** Binding. Every crate in this workspace is built against this document.
> **Owner:** Lead engineer. Changes require a change request (`docs/cr/`).

This document defines the *interfaces* between crates. It deliberately does
**not** define how a protocol works internally — that is the job of the crate
that implements it. If a worker finds that a rule here prevents a correct
implementation, the rule is wrong and must be changed here first.

---

## 1. Crate topology

```text
apps/android      Kotlin + Jetpack Compose shell        (UniFFI client)
apps/macos        SwiftUI + AppKit shell                (UniFFI client)
apps/web          Dashboard: Rust HTTP/WS server + web UI

platform/phairplay-ffi    UniFFI bridge + composition root
                                                (depends on core, engines/*)

core/phairplay-core       Shared vocabulary               (depends on nothing)

engines/discovery         mDNS + SSDP                    (depends on core)
engines/airplay           AirPlay 2 receiver             (depends on core, formats/*)
engines/miracast         Miracast / WFD sink            (depends on core, formats/h264)
engines/cast              Google Cast receiver           (depends on core, formats/plist)
engines/dlna              UPnP / DLNA MediaRenderer      (depends on core)
engines/smartview         Samsung Smart View sink         (depends on core)

formats/plist             Apple property list codec      (depends on nothing)
formats/h264              H.264 bitstream + RTP FU-A     (depends on nothing)
formats/aac               ADTS / AAC-ELD framing         (depends on nothing)
formats/alac              Apple Lossless decoder         (depends on nothing)
```

**Dependency rule (hard):** arrows point strictly downwards, with one
exception that has its own name.

| From | May depend on |
|---|---|
| `apps/web` | `platform/*`, `core/*` |
| `apps/android`, `apps/macos` | `platform/*`, `core/*` |
| `platform/*` | `core/*`, `engines/*` |
| `core/*` | nothing |
| `engines/*` | `core/*`, `formats/*` |
| `formats/*` | nothing |

**The exception: the bridge may depend on the engines.** The bridge holds the
**composition root** — the one place that knows which engines exist and hands
them to the supervisor. Without it the supervisor is empty, and every
`start_protocol` is answered with *"no engine implementation is registered
for the protocol"*. That would leave the native shells able to show a
dashboard and unable to receive anything. The cost is that the bridge links
every protocol; the price is paid once, in the binary of the product, and it
buys a receiver that actually receives. See
[`ADR-0006`](../adr/ADR-0006-the-composition-root-lives-in-the-bridge.md).

A `formats/*` crate must not depend on an `engines/*` crate. An `engines/*`
crate must not depend on `platform/*` or `apps/*`. `core/*` must not depend
on any other crate in the workspace. A native shell must not depend on an
engine: it reaches the engines through the generated bindings, so that the
same engine serves the shell and the dashboard.

---

## 2. The shared vocabulary (`phairplay-core`)

Everything below already exists and compiles. **Do not change these signatures
without a change request.** If a crate needs something that is not here, add it
to `phairplay-core` first, in its own commit, and extend its test suite.

### 2.1 Protocol abstraction

```rust
pub enum ProtocolKind { AirPlay, Miracast, Cast, Dlna }

pub enum ProtocolState { Idle, Starting, Advertising, Streaming, Stopping, Stopped, Failed }

pub struct ProtocolStatus {
    pub protocol: ProtocolKind,
    pub state: ProtocolState,
    pub enabled: bool,
    pub tcp_port: Option<u16>,
    pub udp_port: Option<u16>,
    pub peer_count: u32,
    pub last_error: Option<String>,
}

#[async_trait::async_trait]
pub trait ProtocolEngine: Send + Sync + std::fmt::Debug {
    fn kind(&self) -> ProtocolKind;
    fn status(&self) -> ProtocolStatus;
    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError>;
    async fn stop(&self) -> Result<(), EngineError>;
}
```

`EngineContext` hands the engine everything it may touch:

```rust
pub struct EngineContext {
    pub identity: Arc<DeviceIdentity>,
    pub capabilities: Arc<MediaCapabilities>,
    pub video_sink: Arc<dyn VideoSink>,
    pub audio_sink: Arc<dyn AudioSink>,
    pub events: tokio::sync::mpsc::Sender<SessionEvent>,
}
```

**An engine must never** query the platform for capabilities, open a global
singleton, or call into another engine. It receives what it needs.

### 2.2 Media

```rust
pub trait VideoSink: Send + Sync + Debug {
    fn submit(&self, frame: EncodedVideoFrame) -> Result<(), MediaError>;
    fn set_format(&self, format: &VideoFormat) -> Result<(), MediaError>;
    fn reset(&self);
}
pub trait AudioSink: Send + Sync + Debug {
    fn submit(&self, frame: EncodedAudioFrame) -> Result<(), MediaError>;
    fn set_format(&self, format: &AudioFormat) -> Result<(), MediaError>;
    fn set_volume(&self, volume: f32) -> Result<(), MediaError>;
    fn reset(&self);
}
```

An engine pushes **encoded** frames. It never decodes.

### 2.3 Sessions

```rust
pub enum SessionEvent {
    Started { session: SessionSnapshot },
    VideoFormatChanged { session: SessionId, format: VideoFormat },
    AudioFormatChanged { session: SessionId, format: AudioFormat },
    PlaybackStateChanged { session: SessionId, state: String },
    MetadataChanged { session: SessionId, title: Option<String>, artist: Option<String>, album: Option<String> },
    ArtworkChanged { session: SessionId, artwork: Option<Vec<u8>> },
    Ended { session: SessionId, reason: SessionEndReason },
}
```

An engine **must** emit `Started` when a peer is accepted and `Ended` when the
session terminates. The other events are optional but expected where the
protocol supports them.

---

## 3. Engine implementation rules

### 3.1 Every engine is a state machine

```
Idle → Starting → Advertising → Streaming → Stopping → Stopped
                     ↘ Failed
```

`start()` transitions `Idle → Starting → Advertising`. `stop()` transitions
to `Stopping → Stopped`. `status()` must be callable at any time, including
from another thread while a transition is in flight.

### 3.2 One peer at a time — unless the protocol says otherwise

A second concurrent connection is **rejected**, not queued. AirPlay, Miracast
and DLNA are single-session protocols; Google Cast is multi-session (a Cast
receiver may serve several senders at once) and is the documented exception.

### 3.3 Every network input is untrusted

The threat model: *any device on the local network*. Therefore:

- **Length limits are mandatory** on every read: message bytes, header count,
  header line length, plist size, SDP size, artwork, photo, media payload.
  The limits live in `AppConfig::limits`; engines must read them from there,
  not hard-code them.
- **Bounded queues only.** Both a frame-count *and* a byte budget. A queue
  that is bounded by item count alone can be made to hold hundreds of
  megabytes by a peer sending maximum-size frames — that is a denial of
  service on the local network, which the privacy model explicitly forbids.
- **No unbounded recursion, no unbounded allocations from a length field.**
- **Parsers must reject, not repair.** A malformed SDP is an error, not
  something to guess at.

### 3.4 Timeouts are mandatory

Every blocking read has a deadline. A peer that connects and then says
nothing must not hold a connection slot forever. The default idle timeout is
`AppConfig::performance.idle_timeout_ms`.

### 3.5 Errors carry the reason

Use the core error types. `EngineError::Protocol { reason }` for a peer that
misbehaves, `EngineError::LimitExceeded { reason }` for a limit, and so on. Do
not invent parallel error enums in an engine crate.

---

## 4. Format crate rules

`formats/*` crates are pure: no I/O, no sockets, no platform calls. They
transform bytes into bytes. That makes them trivially testable and reusable.

| Crate | Responsibility |
|---|---|
| `plist` | Apple property list: XML *and* binary (`bplist00`) encode/decode, both directions. |
| `h264` | SPS/PPS parsing, NAL unit splitting, Annex-B, RTP FU-A reassembly. |
| `aac` | ADTS header parsing, AAC-ELD framing, sample counting. |
| `alac` | Apple Lossless decoder — a Rust port of the reference C decoder. |

**Licence note.** The ALAC reference decoder is published by Apple under the
Apache License 2.0. A Rust port may be derived from it. The port must carry
an explicit attribution header in the files derived from it, and the
third-party inventory (`THIRD_PARTY.md`) must list it. Copying the C sources
verbatim into the tree is **not** acceptable — the point of the port is a
native Rust implementation.

**Safety.** `unsafe_code` is denied workspace-wide. Where a decoder benefits
from `unsafe` (it does not, for ALAC), the crate must justify it in an ADR
and the workspace lint must be overridden locally, never globally.

---

## 5. Testing contract

1. **Every public function has a unit test.** Not "most" — every one.
2. **Every hardening measure has a negative test** that actually exercises the
   attack: a 31-zero-bit SPS that overflows, a plist with a 1 GB declared
   length, a SOAP body with a DOCTYPE, a GENA callback pointing at
   `169.254.169.254`.
3. **A test that cannot fail is not a test.** No `assert!(true)`, no test
   that only checks the absence of a panic.
4. **No test is skipped, ignored or commented out** to make a build green.
5. **No stub returns a fake success.** `Ok(())` in place of real work is a
   lie that the acceptance process will catch. (Vocument rule CR-0002.)
6. **No invented numbers.** A measured value is quoted with the command that
   produced it.

---

## 6. Code style

| Rule | Value |
|---|---|
| Edition | Rust 2024 |
| Minimum toolchain | 1.85 |
| `unsafe_code` | denied workspace-wide |
| `unwrap()` / `expect()` | denied workspace-wide |
| `missing_docs` | warn — **every public item is documented** |
| File length | `code_limits.target_max_lines_per_file` (soft) and `code_limits.hard_max_lines_per_file` (hard) in `config/performance-budgets.toml` |
| Formatting | `cargo fmt` |
| Lints | `cargo clippy -- -D warnings` |

Every module starts with a doc comment that states **what** it is, **why** it
exists, and (where it helps) a **short example**. The Vocument house style is
the reference; the PhairPlay Kotlin sources use the same "WHAT / WHY /
EXAMPLE" convention and are a good model.

---

## 7. Documentation contract

The Vocument document set is the binding format. Every crate ships with:

- a `README.md` in its directory explaining the crate's role in one paragraph
  and linking to the crate root documentation;
- an ADR in `docs/adr/` for every decision that is not obvious from the code;
- acceptance evidence in `docs/acceptance/` for every milestone, in the
  Vocument format.

**A code change is not finished if the documentation became wrong.**

---

## 8. Licensing

- The whole workspace is **AGPL-3.0-or-later**.
- Every `Cargo.toml` inherits `license.workspace = true` — never a literal.
- Every source file that carries a licence header carries the AGPL header.
- Third-party code keeps its own licence and is listed in `THIRD_PARTY.md`.
- The dependency policy is enforced by `deny.toml` and a `cargo-deny` gate in
  CI. The **pinned** cargo-deny version is part of the configuration: a
  different version reads the same `deny.toml` differently. Use the pinned
  version or the gate is not a gate.
