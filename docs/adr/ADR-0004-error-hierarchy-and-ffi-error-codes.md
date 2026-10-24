# ADR-0004 — The Error Hierarchy and the FFI Error Codes

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-002, FR-003, FR-007, FR-015, FR-047, FR-048, FR-054, NFR-SEC-003, NFR-SEC-004, NFR-INT-003, NFR-PERF-005

## Context

The receiver stack crosses several boundaries: a socket on the local
network, a bridge into Kotlin and Swift, a platform decoder. Each boundary
needs to say *what* went wrong without knowing the layer above it, and each
boundary needs to say it in a way the other side can act on.

Three constraints make the decision non-obvious:

1. **The foreign side must not parse English.** The shell has to distinguish
   "the user typed a wrong protocol name" from "the socket could not be
   bound", in two languages, and it must survive a reworded message. A
   string is not a contract; a number is.
2. **The threat model makes the distinction between "invalid" and "resource
   exhausted" a security distinction.** A peer that sends a message that is
   too large is a *protocol violation*; a sink whose queue is full is a
   *resource problem* of the device. A receiver that conflates the two logs
   the wrong thing and reacts with the wrong measure.
3. **Parallel error enums multiply.** Every engine that defines its own
   error type has to be translated into the vocabulary of the bridge, and
   the translation is where a classification is lost. The AirPlay engine
   already has `AirPlayError`, the Miracast engine `MiracastError`; they are
   *internal* errors of the crate and they must not become a second public
   vocabulary.

The privacy promise adds a fourth: an error message crosses into a log file
and into a foreign shell, so it may not carry a device name, an address or
the content of a stream (FR-003, NFR-PRIV-001).

## Decision

The core crate owns **one** error hierarchy, and every crate of the
workspace either returns it or converts into it.

1. **The root is `CoreError`.** It has exactly one variant per concern:

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

2. **An engine error carries the protocol as a `&'static str`.** The
   variants `Start`, `NotRunning`, `AlreadyRunning`, `Protocol`,
   `LimitExceeded` and `Stopped` all name the protocol they belong to, so
   that the message names the receiver without the core having to know the
   concrete engines.

3. **A crate may have an internal error type, but it must not become a
   second public vocabulary.** `AirPlayError` and `MiracastError` exist
   because a parser wants to say "the SPS overflowed" and not "somebody
   broke". They are converted into `EngineError` at the edge of the crate.

4. **The classification is derived from the kind of the error, never from
   the wording of the message.** `ErrorKind::classify(&CoreError) -> ErrorKind`
   maps every error onto one of nine kinds: `InvalidInput`,
   `LimitExceeded`, `ProtocolViolation`, `ResourceExhausted`, `Timeout`,
   `Unsupported`, `InvalidState`, `Io`, `Internal`.

5. **The numeric code is owned by the core and is stable.**

   ```text
   1 InvalidInput        5 Timeout              9 Internal
   2 LimitExceeded       6 Unsupported
   3 ProtocolViolation   7 InvalidState
   4 ResourceExhausted   8 Io
   ```

   The numbers are part of the public contract of the bridge and are not
   renumbered without a major version bump. `ErrorCode::from(ErrorKind)` is
   the only conversion, and it is total.

6. **The bridge mirrors the classification, it does not own it.**
   `FfiErrorCode` is the readable mirror of `ErrorCode`; `FfiErrorCode::as_i32`
   is identical to `ErrorCode::as_i32`, and a unit test compares the two for
   every code. An unknown number is *not* mapped onto a code: `from_i32`
   returns `None`, because a number the core does not define is a bug in the
   caller and must not be silently reported as an internal error.

7. **The message of an error is a single line with the whole cause chain.**
   `phairplay_core::error::describe(&dyn Error)` renders the error and every
   source into one line, so that a log stays readable and the cause does not
   get lost.

8. **An error message carries no secret.** The variants that could carry a
   device name or an address (`NetworkError::Bind`, `PeerOutsideSubnet`)
   carry the *address* only, and the masking of the names happens in the
   privacy layer of the core before the message is written.

## Consequences

### Positive

- The foreign side branches on a number. A reworded message, a translation
  or a new error variant in the core cannot change the behaviour of a shell.
- The security-relevant distinction between "the peer misbehaved" and "the
  device ran out of resources" survives every hop, because it is carried by
  the kind and not by the text.
- A new error variant in the core is classified by one `match` arm in
  `ErrorKind::classify`, and the compiler points at every place that has to
  learn about it.
- The `describe` helper makes the log of a failure one line instead of a
  paragraph, which is what a log aggregator can read.

### Negative / Trade-offs

- The hierarchy is closed. A failure that does not fit one of the variants
  has to be forced into `Internal`, which loses the distinction. A new
  variant is a change of the public API of the core.
- `classify` is a `match` over the whole hierarchy. Every new variant makes
  it non-exhaustive, which is the intended effect, but it means the core and
  the bridge have to be changed together.
- The numbers are frozen. A hole in the numbering cannot be filled later
  without breaking a shell.
- The `&'static str` of the protocol inside `EngineError` is a small leak of
  the protocol names into the error type. It is accepted because the
  alternative — a `ProtocolKind` field — would make the error type of the
  core depend on the vocabulary, and the vocabulary is already in the core
  but the error should be printable without it.

### What the decision makes impossible

- A crate may not define a *public* error enum that crosses the bridge. The
  bridge speaks `FfiError` and nothing else.
- A shell may not branch on the text of an error message.
- An error code may not be renumbered. A new kind gets a new number at the
  end of the list.
- An error message may not carry the name of a sending device, an IP address
  or any content of a stream.

## Alternatives Considered

- **One string per error, no codes.** Rejected: it is what the bridge
  explicitly does not do, and it makes the shell depend on the wording of
  the core in two languages. A typo in a message would become a silent
  behaviour change on a phone.
- **A code per error *variant* (fifty numbers).** Rejected: the shell would
  have to know the internals of the core hierarchy, and every new variant
  would be a change of the contract of the bridge. Nine kinds are enough for
  a host to react: show a message, retry, disable the protocol, or give up.
- **An error code per *engine*.** Rejected: the host would need to know
  which protocol failed to interpret the code, and a new engine would be a
  new numbering scheme.
- **`anyhow::Error` with a `downcast` on the foreign side.** Rejected: the
  downcast does not exist on the other side of a language boundary, and the
  `anyhow` chain is not serialisable.
- **Keeping the numbers in the bridge crate.** Rejected: the core is the
  owner of the vocabulary, and the bridge is a mirror. A number that lives
  in the bridge is a number that the core cannot test.
- **Letting the engine return a free-form `String` as the error.** Rejected:
  that is the definition of an error that cannot be classified, and it is
  the reason `EngineError` carries a `reason` *inside* a closed set of
  variants.

## Verification

- `core/phairplay-core/src/error.rs` carries the unit tests of the
  classification (`error_kind_of_invalid_config_is_invalid_input`,
  `error_kind_of_queue_full_is_resource_exhausted`) and of the stability of
  the numbering (`error_codes_are_stable`).
- `core/phairplay-core/src/error.rs` carries the test of the cause chain
  (`describe_includes_the_cause_chain`).
- `platform/phairplay-ffi/src/error.rs` carries the test that the codes of
  the bridge and the codes of the core agree
  (`the_numeric_codes_match_the_core`) and the test that an unknown number
  is not mapped onto a code (`an_unknown_number_is_not_mapped_onto_a_code`).
- `cargo clippy --workspace --all-targets -- -D warnings` fails if a new
  variant of the hierarchy is added without a classification, because
  `classify` matches exhaustively.
