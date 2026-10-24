# ADR-0003 — UniFFI Bindings and the Verification of the Bridge

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-003, FR-005, FR-006, FR-011, FR-014, FR-022, FR-072, FR-073, NFR-INT-001, NFR-INT-002, NFR-SEC-001

## Context

PhairPlayRust has two native shells — an Android app in Kotlin and a macOS
app in Swift — and a web dashboard. All three talk to the same Rust core.
The shells must not contain a second copy of the protocol knowledge, and
they must not be able to crash the receiver: a panic in the Rust library
takes the application of the user with it.

The transport across the language boundary has to satisfy three properties
that pull in different directions:

1. **The foreign side must be able to react to a failure.** The shell has to
   distinguish "the user typed a wrong protocol name" from "the socket could
   not be bound" without parsing an English sentence, in two languages.
2. **The foreign side must be able to push.** The engines hand encoded
   frames to a sink and report events. A sink that lives in the host —
   MediaCodec on Android, VideoToolbox on Apple platforms — has to be
   callable from the Rust side.
3. **The boundary must not become a second architecture.** Every type that
   crosses it needs a foreign mirror, and every mirror is a place where the
   two sides can drift apart.

The product promise adds a constraint: the bridge is a *transport*, not a
service. It opens no socket of its own, sends nothing to a cloud and
collects nothing (FR-003, FR-005).

## Decision

The bridge is `platform/phairplay-ffi`. It is the only door from the native
side into the core, and it is a UniFFI crate.

1. **The exported surface is platform-neutral.** There is no exported
   function per platform. A platform difference is expressed by a different
   implementation of a callback interface, never by a new function.
2. **The data transfer objects mirror the types of the core.** `FfiAppConfig`
   mirrors `AppConfig`, `FfiProtocolStatus` mirrors `ProtocolStatus`,
   `FfiSessionSnapshot` mirrors `SessionSnapshot`. The DTOs do not introduce
   a second model; they carry the same fields under the same names.
3. **Every error crosses the bridge as a code and a message.** The numeric
   code is the `ErrorCode` of the core (see ADR-0004). The field is called
   `details` and not `message`, because `message` is the name of
   `Throwable.message` in Kotlin and would collide there.
4. **The host implements three callback interfaces.** `ReceiverEventSink` for
   the events, `PlatformVideoSink` and `PlatformAudioSink` for the media.
   The bridge adapts them to the sinks of the core. The call happens on a
   thread of the bridge, and the documentation of every callback says so.
5. **The bindings are generated, never written by hand.** The crate ships a
   `uniffi-bindgen` binary target; `scripts/generate-bindings.sh` produces
   the Swift bindings and the Kotlin bindings from the Rust sources. A
   hand-edited binding drifts from the source, and the drift is noticed when
   a phone crashes.
6. **A panic never crosses the bridge.** Every failure that leaves the
   bridge is an `FfiError` with a code and an explanation. A panic that
   slips through is caught and reported as an internal failure.
7. **The round trip is proven by a test, not by an inspection.** The
   integration tests of the crate push a value through the bridge and read it
   back:
   - `platform/phairplay-ffi/tests/bridge_roundtrip.rs` — the DTOs and the
     configuration,
   - `platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs` — the foreign
     ABI, i.e. the bytes that really cross the boundary,
   - `platform/phairplay-ffi/tests/supervisor_lifecycle.rs` — the lifecycle
     of the receiver through the bridge.
   The shared fixtures live in `platform/phairplay-ffi/tests/common/`.

The library is built as `lib`, `cdylib` and `staticlib` at once: the native
shells load the shared library, the dashboard links the static library into
its binary, and the Rust tests use the plain library.

## Consequences

### Positive

- One implementation of the receiver, three consumers. A fix in the core
  reaches the phone, the Mac and the dashboard at the same time.
- The shell can branch on a number. A change of the wording of an error
  message cannot change the behaviour of the host.
- The generated bindings cannot silently drift: they are produced by one
  command from the Rust sources.
- The foreign ABI is covered by a test, so a mistake in the layout of a DTO
  is caught on the machine that builds, not on the device that crashes.

### Negative / Trade-offs

- Every type that crosses the boundary needs a foreign mirror. A new field in
  a core type means a new field in the DTO, in the Swift struct and in the
  Kotlin data class.
- The callback interfaces are invoked from a thread of the bridge. A host
  that blocks in a callback blocks the receiver; the documentation has to
  say that, and every host has to obey it.
- The `unsafe_code = "deny"` of the workspace is lifted *locally* for this
  one crate, because the scaffolding that UniFFI generates contains `unsafe`
  blocks. The exception is stated in the manifest of the crate, covers only
  the generated code and is never global. The code of the crate itself
  contains no `unsafe`.
- A panic inside a callback of the host is a panic in the host, not in the
  bridge. The bridge cannot protect the host from the host.

### What the decision makes impossible

- A shell may not reach a protocol engine directly. Every call goes through
  the bridge or through the local web backend.
- The bridge may not open a socket, bind a port or send anything over the
  network. It is a transport.
- The bridge may not introduce a type that the core does not have. A DTO
  that exists only on the foreign side is a second model.
- A hand-edited binding may not be committed. The generated directories are
  output, not source.

## Alternatives Considered

- **A hand-written C ABI with a Swift/Kotlin wrapper of its own.** Rejected:
  two wrappers means two implementations of the marshalling, and the
  ownership rules of a C ABI (who frees the buffer, who owns the callback)
  are exactly the bugs that a crash on a phone is made of. UniFFI generates
  that part.
- **A local HTTP/WebSocket API for the shells as well.** Rejected: it makes
  the shells network clients of a service that has to be reachable, and the
  privacy promise of the product is *local network only, no service*. The
  dashboard may have a loopback server because it is a local control panel;
  a phone app that pairs with a desktop over the LAN is a different product.
- **Exposing the core types directly with `serde` and a JSON codec.**
  Rejected: the error of a JSON envelope is a string, and the shell would be
  back to parsing English sentences. It also loses the type safety of the
  generated bindings.
- **One bridge crate per platform.** Rejected: the exported surface would
  fork. A feature that exists on iOS and not on Android would need two
  crates, two versions of the DTOs and a rule for the difference.
- **Catching panics with `catch_unwind` at every boundary.** Accepted as
  part of the decision, but only as a last line of defence: the real
  guarantee is that the bridge returns a typed error instead of panicking in
  the first place.

## Verification

- `cargo test -p phairplay-ffi` runs the three integration tests named
  above plus the unit tests of the DTOs, the error mapping and the sinks.
- `bash scripts/generate-bindings.sh` regenerates the Swift and the Kotlin
  bindings from the Rust sources; a regeneration in a clean tree produces no
  diff.
- The drift gate of the CI re-runs the generator and fails when the
  generated directories differ from the committed ones.
- The exception of the `unsafe_code` lint is visible in the manifest of the
  crate and is the only one of its kind in the workspace.
