# ADR-0002 — The Engine Trait: One Protocol Abstraction

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-006, FR-007, FR-011, FR-015, FR-021, FR-032, FR-036, FR-037, FR-040, FR-051, NFR-INT-001, NFR-PERF-001

## Context

The four receiver protocols of PhairPlayRust share a *shape* even though
they share no bytes: they advertise a service, accept a peer, negotiate,
stream, and tear down. Everything around the protocol is identical: the
session registry has to know which protocol a session belongs to, the
dashboard has to show one card per protocol, the native shell has to offer
one switch per protocol, and the metrics have to be collected the same way.

If each engine answers those questions in its own way, the shell grows four
special cases and the dashboard grows four data models. The alternative is
one abstraction that the four engines implement, so that the shell, the
dashboard and the session registry work the same way regardless of which
protocol is running.

The abstraction must also survive two hard constraints of the product:

1. **The engine never reaches for global state.** The threat model is *any
   device on the local network* (NFR-SEC-006) and the privacy promise is
   *no telemetry, no cloud* (FR-003, FR-005). An engine that queries the
   platform for its capabilities, or that opens a global singleton, cannot
   be tested and cannot be reasoned about.
2. **The engine never decodes.** Decoding is a platform concern: MediaCodec
   on Android, VideoToolbox and AudioToolbox on Apple platforms, a software
   decoder in the browser. The engine delivers *encoded* frames in a
   platform-neutral description (FR-007 — the receiver advertises only what
   it can decode).

## Decision

The core crate owns exactly one protocol abstraction:

```rust
// core/phairplay-core/src/protocol.rs

pub enum ProtocolKind { AirPlay, Miracast, Cast, Dlna }

pub enum ProtocolState {
    Idle, Starting, Advertising, Streaming, Stopping, Stopped, Failed,
}

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

and exactly one environment for it:

```rust
pub struct EngineContext {
    pub identity: Arc<DeviceIdentity>,
    pub capabilities: Arc<MediaCapabilities>,
    pub video_sink: Arc<dyn VideoSink>,
    pub audio_sink: Arc<dyn AudioSink>,
    pub events: tokio::sync::mpsc::Sender<SessionEvent>,
}
```

The rules that follow from this shape are part of the decision:

1. **An engine receives everything it is allowed to touch.** The identity it
   advertises, the capabilities it may claim, the sinks that receive its
   frames and the channel on which it reports are handed to it in
   `EngineContext`. An engine never reads global state, never calls into
   another engine and never queries the platform.
2. **An engine is a state machine.** `Idle → Starting → Advertising →
   Streaming → Stopping → Stopped`, with `Failed` as the dead end.
   `status()` must be callable at any time, from any thread, including
   while a transition is in flight.
3. **An engine pushes encoded frames.** `VideoSink::submit` takes an
   `EncodedVideoFrame`, `AudioSink::submit` an `EncodedAudioFrame`. The
   engine never decodes and never talks to a platform decoder.
4. **An engine reports what happened, not what it thinks.** The channel in
   `EngineContext` carries `SessionEvent`s. `Started` when a peer is
   accepted, `Ended` when the session terminates; the metadata events are
   optional and only where the protocol supports them.
5. **One peer at a time — except for Cast.** AirPlay, Miracast and DLNA
   reject a second concurrent connection instead of queueing it (FR-021,
   FR-036, FR-051). Google Cast is the documented exception (FR-040).
6. **A protocol that is not a receiver is not an engine.** `engines/discovery`
   is the facility the receiver protocols share, not a protocol of its own, and
   therefore does not implement `ProtocolEngine`. Adding
   `ProtocolKind::Discovery` would change the vocabulary of the core, which
   this ADR forbids.
7. **The engine owns its configuration, the core owns the limits.** The
   per-protocol settings (`enabled`, `port`, `require_pin`) are the
   configuration of the engine; the resource limits are the budget of the
   process and live in `AppConfig` (see ADR-0001 and the configuration
   architecture in `ARCHITECTURE.md` §9).

## Consequences

### Positive

- The session registry, the metrics, the capability advertisement, the
  native bridge and the dashboard are written once and work for all four
  protocols.
- An engine is testable without a network, a decoder or a device: the
  context is a value, the sinks are traits, the event channel is a channel.
- A fifth protocol is a new crate in `engines/` and an entry in
  `ProtocolKind`. Nothing else in the workspace changes.
- The privacy promise is enforceable: an engine that only sees what the
  context hands it cannot leak a device name or an address that it never
  received.

### Negative / Trade-offs

- `EngineContext` is a fat parameter. Every engine that needs one more
  input — the configuration, for example — cannot get it without a change
  of the core, and that change touches every engine and every test that
  builds a context.
- `ProtocolStatus` is a snapshot, not a subscription. The dashboard polls
  or receives the event channel; a push model for the status would need a
  second channel and a second version of the trait.
- `async_trait` boxes every future of the trait. The engines are I/O bound
  and never notice; a CPU bound engine would.
- The trait is object safe on purpose, because the bridge holds the four
  engines as `Arc<dyn ProtocolEngine>`. That rules out associated types and
  generic methods.

### What the decision makes impossible

- An engine may not read a configuration file, may not query the platform
  and may not open a socket that belongs to another engine.
- The core may not grow a variant of `ProtocolKind` for a facility that is
  not a receiver protocol.
- The platform bridge may not expose an engine-specific function to a shell.
  Everything a shell can do, it does through `ProtocolStatus`,
  `SessionEvent` and the configuration.

## Alternatives Considered

- **One trait per protocol, with a `Box<dyn …>` per shell.** Rejected: the
  shell would need four fields, four switches and four status paths, and the
  dashboard four models. The protocol differences are already visible in the
  status and in the events.
- **An enum-dispatch engine (`match kind { … }` in the core).** Rejected:
  the core would have to know the concrete engines to dispatch, which
  breaks the direction of ADR-0001 and turns the core into a switchboard
  that every new protocol has to edit.
- **A message-based engine (the core sends commands, the engine replies).**
  Rejected for now: it makes `start()` asynchronous in a way that no caller
  needs, and it hides the lifetime of the engine behind a channel. The
  trait with `start`/`stop`/`status` is the smallest interface that carries
  the whole lifecycle.
- **A `ProtocolKind::Discovery` variant.** Rejected: discovery is not a
  receiver. A sender never streams to a discovery service, and a status card
  for it would be a fifth card that says nothing.
- **Letting the engine query the platform for its capabilities.** Rejected:
  it would make every engine untestable on a laptop and would make the
  capability advertisement a per-platform affair instead of a value that the
  platform layer fills in once.

## Verification

- The trait lives in `core/phairplay-core/src/protocol.rs`; every engine
  implements it in `engines/*/src/engine.rs` and the implementations are
  exercised by the state-machine tests of the respective crate.
- `core/phairplay-core/src/protocol.rs` carries the unit tests for
  `ProtocolKind` (round trip of the string form, the aliases, the service
  types) and `ProtocolState::accepts_peers`.
- `core/phairplay-core/src/session.rs` carries the tests of the registry
  that folds the events of the engines.
- The direction that an engine may not break is checked by
  `scripts/check-crate-boundaries.sh` (ADR-0001).
- `scripts/check-docs-links.sh` keeps the documented budgets and the typed
  model of the configuration in agreement.
