# ADR-0001 — Cargo Workspace and Module Boundaries

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-006, FR-008, FR-009, FR-011, FR-052, FR-055, NFR-INT-001, NFR-INT-002, NFR-RES-001, NFR-SEC-006

## Context

PhairPlayRust is one product with four receiver protocols (FR-006) that
share a configuration, a session model, a capability model and an error
model. The protocols themselves are unrelated: AirPlay speaks RTSP and SDP,
Miracast speaks RTSP and WFD, Google Cast speaks CastV2 over TLS, DLNA
speaks SSDP, SOAP and GENA. The threat model is the same for all of them —
*any device on the local network* (NFR-SEC-006) — and so are the limits, the
timeouts and the rules of the session lifecycle.

Two forces pull the code apart. The first is the desire to share: a limit
that is written down twice is a limit that is different in two places, and a
sender that is answered differently by two receivers is a bug report that
cannot be reproduced. The second is the desire to separate: a wire format
must not know that a protocol exists, and a protocol must not know that a
platform decoder exists. A format that knows about RTSP cannot be reused by
the Cast engine; a protocol that queries the platform cannot be tested
without a device.

The workspace is therefore split into five layers, and the *direction* of the
dependencies between them is the decision this ADR records. The direction
cannot be left to a code review: a review sees a diff, not the whole graph,
and a dependency that nobody notices is added in a hurry.

## Decision

The workspace consists of five layers. A crate may only depend on the layers
listed next to it:

```text
apps/*        → platform/*, core/*
platform/*    → core/*
engines/*     → core/*, formats/*
formats/*     → (nothing inside the workspace)
core/*        → (nothing inside the workspace)
```

Three consequences of this table are decisions of their own and are named
here so that they are checkable:

1. **`core/*` depends on no other crate of the workspace.** The core is the
   shared vocabulary. A vocabulary that needs a vocabulary is not a
   vocabulary any more. The core is the only crate that may be depended upon
   by every other layer.
2. **`formats/*` depends on no other crate of the workspace.** A format crate
   is pure: it transforms bytes into bytes, it opens no socket, it calls no
   platform API (FR-052, FR-055).
3. **An `engines/*` crate may use a `formats/*` crate, never the other way
   round.** The AirPlay engine needs an H.264 parser; the H.264 parser must
   not know that AirPlay exists.

The concrete crates of the workspace are:

| Layer | Crate | Package | May depend on (workspace) |
|---|---|---|---|
| apps | `apps/web` | `phairplay-web` | `core`, `platform` |
| platform | `platform/phairplay-ffi` | `phairplay-ffi` | `core` |
| core | `core/phairplay-core` | `phairplay-core` | — |
| engines | `engines/discovery` | `phairplay-discovery` | `core` |
| engines | `engines/airplay` | `phairplay-airplay` | `core`, `formats/*` |
| engines | `engines/miracast` | `phairplay-miracast` | `core`, `formats/*` |
| engines | `engines/cast` | `phairplay-cast` | `core`, `formats/*` |
| engines | `engines/dlna` | `phairplay-dlna` | `core`, `formats/*` |
| formats | `formats/plist` | `phairplay-plist` | — |
| formats | `formats/h264` | `phairplay-h264` | — |
| formats | `formats/aac` | `phairplay-aac` | — |
| formats | `formats/alac` | `phairplay-alac` | — |

The native shells `apps/android` and `apps/macos` are not Rust crates. They
are UniFFI *clients* of `platform/phairplay-ffi` and therefore sit in the
`apps` layer without a manifest of their own.

**The decision is machine-checked.** `scripts/check-crate-boundaries.sh` reads
the manifests of the workspace, resolves every `path = "..."` dependency
against the directory of its crate, determines the layer of both ends and
fails with exit code 1 when the pair is not in the table above. The check
covers `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` and
their `[target....]` variants, and it refuses a `path` entry in
`[workspace.dependencies]` — such an entry hands a workspace-internal crate
to every member and would make the table unenforceable.

A crate that lives in one of the five layer directories but is not a member
of the workspace is a finding as well: it would escape the gate.

## Consequences

### Positive

- The dependency direction is a fact that can be checked in a second, not an
  intention that survives until the next hurried change.
- Every engine can be tested without a device, a socket or a decoder, because
  the format crates it needs are pure.
- The core stays free of every third-party dependency except the four it
  needs for its own model (`serde`, `toml`, `thiserror`, `tokio`), so the
  native shells link a small library.
- A protocol can be removed without touching the other three: the only
  shared things are the core types and the discovery service types.

### Negative / Trade-offs

- The format crates cannot use the error type of the core, so each of them
  carries its own small error enum. That is the price of purity, and it is
  paid four times.
- A change of the shared vocabulary — a new field in `ProtocolStatus`, for
  example — touches the core, the bridge, the DTOs of the bridge and the
  dashboard. The layers do not make a change cheap; they make it *local*.
- The gate has to be run in CI. A gate that is not run is a comment.

### What the decision makes impossible

- A format crate may not report an error of the core, may not read a
  configuration file and may not open a socket. A format crate that needs
  one of the three is not a format crate.
- The core may not know the name of a protocol. `ProtocolKind` is the
  closed set of the four protocols, and it lives in the core because the
  session registry and the dashboard have to speak about all of them.
- The shell may not talk to an engine directly. Every call goes through
  `platform/phairplay-ffi` or through the local web backend.

## Alternatives Considered

- **A single crate with modules.** Rejected: the module tree of a single
  crate expresses an *import* discipline, not an enforced one. A `use` that
  points upwards compiles, and the first reviewer that does not notice the
  import leaves a format crate that knows about RTSP.
- **`core` depending on `engines` (a "plugin" direction).** Rejected: the
  core would then need to know the concrete engines to build the receiver,
  and the platform bridge would link every protocol into the native shell
  even when the user switched three of them off.
- **Letting `formats/*` use the error type of the core.** Rejected for now:
  it would be convenient, but it would make the error hierarchy of the core
  part of the public API of four crates that are meant to be reusable on
  their own. The decision is reversible; the cost of reversing it later is
  higher than the cost of four small error enums now.
- **Checking the direction with a linter instead of a script.** Rejected:
  the lints of Cargo cannot express "the layer of a crate" without a custom
  lint, and a custom lint is a toolchain dependency for a rule that is
  fifteen lines of manifest reading.

## Verification

- `bash scripts/check-crate-boundaries.sh` — green on the current tree,
  twelve crates, nine internal edges.
- `bash scripts/check-crate-boundaries.sh --list` — prints the crates and
  every allowed internal edge.
- `bash scripts/check-crate-boundaries.sh --self-test` — copies the manifests
  into a temporary directory, breaks four dependencies
  (`formats → engines`, `core → formats`, `engines → platform`,
  `platform → engines`) and proves that the gate answers with exit code 1 in
  every case and with exit code 0 on the unmodified copy. The repository
  itself is not modified by the self test.
- The negative test on the real tree is recorded verbatim in
  `docs/acceptance/` of the milestone that introduces the gate.
