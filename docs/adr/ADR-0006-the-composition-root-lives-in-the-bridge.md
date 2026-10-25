# ADR-0006 — The composition root lives in the bridge

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-003, FR-007, FR-011
**Supersedes:** nothing. **Amends:** ADR-0001 (the dependency matrix).

---

## Context

ADR-0001 fixed the dependency matrix of the workspace. It drew the line
between the layers so that a format crate cannot know that RTSP exists, and
so that the core cannot know a concrete protocol.

That matrix had one hole, and the hole was invisible until a receiver had to
be **started**. The question "who builds the engines?" has exactly one
correct answer, and nobody answered it:

- The **core** must not know the engines. ADR-0001 rejected the "plugin"
  direction for exactly this reason, and the rejection stands.
- The **engines** must not know each other and must not know the platform.
- The **native shells** reach the receiver through the generated UniFFI
  bindings. They cannot build an engine themselves — they do not even have
  the Rust types.
- The **platform bridge** exposed `register_engine` and the supervisor, but
  nothing ever called `register_engine`. The bridge therefore had a
  supervisor with an **empty** engine table.

The consequence was not a compile error. It was a product that starts, shows
the protocols in its dashboard and answers every attempt to start one with
*"no engine implementation is registered for the protocol"*. The native
shells of Android and macOS could display the state of a receiver that could
not receive.

The local web backend grew a composition root of its own while this was
being fixed — in `apps/web`, which the boundary script of ADR-0001 rejected,
because the matrix said an application may not depend on an engine. The
script was right about the shell and wrong about the backend: the script
could not tell a shell from a backend, because it knew only the layer
`apps/*`.

## Decision

1. **The composition root lives in the platform bridge**, in
   `platform/phairplay-ffi/src/composition.rs`. It is the one place that
   knows which engines exist, builds them from the configuration and hands
   them to the supervisor.
2. **The bridge may depend on `engines/*`.** This is the one exception to
   the downwards rule of ADR-0001, and it is named rather than smuggled in.
3. **The local web backend reaches the engines through the bridge**, like
   every other client. `apps/web` depends on `platform/phairplay-ffi` and on
   `core/*` — nothing else.
4. **A native shell may never depend on an engine.** It reaches the engines
   through the generated bindings. The rule is in the matrix even though no
   Cargo crate can violate it today: the native shells are not Cargo crates,
   and the rule keeps it that way.
5. **The boundary script distinguishes the two kinds of application.** The
   layer `apps/*` becomes `app-backend` (`apps/web`) and `app-shell`
   (`apps/android`, `apps/macos`).

## Consequences

- The native shells can start a protocol. This is the point of the change.
- There is **one** truth about what the receiver consists of. The bridge
  registers the engines; the web backend and the shells both call it.
- The bridge links every protocol into every binary of the product. That is
  the price ADR-0001 wanted to avoid. It is paid once, in the binary, and it
  buys a receiver that receives. A binary that is a few megabytes larger and
  works is worth more than a binary that is small and does nothing.
- The self-test of the boundary gate lost the case `platform → engines` —
  that direction is legal now — and gained two cases that are still
  forbidden: `platform → formats` and `app-backend → formats`. The gate
  still fails on five real violations.

## Alternatives Considered

- **The composition root stays in the local web backend.** Rejected: the
  native shells would still find an empty supervisor. The project would
  have two composition roots — a second truth about what runs — and one
  application that works while the other does not.
- **The core depends on the engines (the "plugin" direction).** Rejected in
  ADR-0001, and the rejection still stands: the core is the shared
  vocabulary, and a vocabulary that knows every protocol is no longer a
  vocabulary.
- **The shells get their own composition root.** Rejected: the shells are
  Kotlin and Swift. They cannot build a Rust engine, and a second
  composition root would be a second truth.
- **A new crate `platform/phairplay-receiver` beside the bridge.** Deferred:
  it would have the same effect as putting the composition root into the
  bridge, with one more crate. If the bridge ever grows two surfaces — a
  bridge for the shells and one for the dashboard — the composition root
  moves into a crate of its own.

## Verification

- `bash scripts/check-crate-boundaries.sh` — green, thirteen crates, fifteen
  internal edges, no violation.
- `bash scripts/check-crate-boundaries.sh --self-test` — the gate still
  answers with exit code 1 for `formats → engines`, `core → formats`,
  `engines → platform`, `platform → formats` and `app-backend → formats`,
  and with exit code 0 on the unmodified copy.
- `cargo test --workspace` — the tests of the composition root are in
  `platform/phairplay-ffi/src/composition/tests.rs` and prove that every
  protocol of the configuration is registered with the supervisor.
