# ADR-0005 — The Boundary Between the Format Level and the Protocol Level

**Status:** proposed
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-017, FR-019, FR-020, FR-029, FR-052, FR-053, FR-054, FR-055, FR-056, FR-057, FR-058, NFR-SEC-003, NFR-SEC-005, NFR-RES-002

## Context

A *format* is a transformation of bytes into bytes: a property list, an
H.264 bitstream, an ADTS frame, an ALAC frame. A *protocol* is a
conversation: RTSP, SDP, CastV2, SSDP, SOAP, GENA. The two are easy to
confuse because a protocol is mostly made of formats, and the confusion has
a price:

- A format that knows its protocol cannot be reused. The Cast engine needs
  a property list; the AirPlay engine needs a property list; DLNA needs an
  XML reader. Three copies of the same decoder means three places where the
  next overflow bug has to be fixed.
- A format that knows its protocol cannot be tested honestly. The test of a
  decoder says "a crafted document is rejected"; the test of a protocol
  parser says "a crafted *request* is rejected". The second hides the first.
- The limits of a format are the limits of the process. The threat model is
  *any device on the local network* (NFR-SEC-006): a plist that declares a
  length of one gigabyte must be rejected by the decoder, and that
  rejection must not depend on which protocol delivered the bytes.

The build contract already states the rule — "`formats/*` crates are pure:
no I/O, no sockets, no platform calls" and "`engines/*` crates must not
depend on `platform/*` or `apps/*`" — and `ARCHITECTURE.md` §3 already names
the intended dependencies of the AirPlay engine as `core` **and**
`formats/{plist,h264,aac,alac}`.

**The tree does not yet match that description.** Two deviations exist
today, and this ADR decides what to do with them:

| Deviation | Size | Where |
|---|---|---|
| The AirPlay engine carries a *second* property-list codec | nine files, each below the code limit of `config/performance-budgets.toml` | `engines/airplay/src/plist/**` next to `formats/plist` |
| The AirPlay engine carries a *second* H.264 parameter-set parser and a second FU-A reassembler | two files, each below the code limit of `config/performance-budgets.toml` | `engines/airplay/src/h264.rs`, `engines/airplay/src/rtp.rs` next to `formats/h264` |

Both are *duplicates of a format inside a protocol crate*. The rule that is
being decided here is therefore binding; what is **proposed** is the
migration that removes the two deviations, because it changes the public API
of `phairplay-airplay` and therefore needs a change request
(`docs/cr/`) per the build contract.

A third, smaller deviation belongs to the same family: the Miracast engine
hard-codes the number of headers and the length of a header line that the
global limit set of the core already defines — the keys
`limits.max_header_count` and `limits.max_header_bytes` in
`config/performance-budgets.toml` — while the build contract says that "the
limits live in `AppConfig::limits`; engines must read them from there, not
hard-code them".

## Decision

1. **The format level is `formats/*`, and it is pure.** A format crate
   transforms bytes into bytes. It opens no socket, reads no configuration
   file, calls no platform API and knows no protocol. It has its own error
   type, because it must be usable without the core.
2. **A protocol crate uses the format crates; it never re-implements them.**
   The AirPlay engine depends on `formats/plist` and `formats/h264` and
   uses their decoders. The Cast engine depends on `formats/plist` and uses
   the same decoder, which is the point: one decoder, one set of limits, one
   place where an overflow is fixed.
3. **The limits of a format are the limits of the format crate, not of the
   engine that happens to use it.** `formats/plist` carries `PlistLimits`,
   `formats/h264` carries its own bound on the size of a reassembled NAL
   unit. An engine that wants a smaller limit passes a smaller one; it does
   not write a second decoder with a different constant.
4. **The migration of the two deviations is done in this order:**
   a. `engines/airplay` gains a dependency on `formats/plist` and on
      `formats/h264` (both are allowed by ADR-0001: `engines/* → core/*,
      formats/*`);
   b. the AirPlay property-list codec is replaced by `formats/plist`. The
      public name `phairplay_airplay::plist::PlistValue` becomes a
      re-export of `phairplay_plist::Value`, so that a caller of the
      published API keeps compiling;
   c. the AirPlay SPS parser and its FU-A reassembler are replaced by
      `formats/h264`. `phairplay_airplay::h264::parse_sps` becomes a thin
      adapter over `phairplay_h264::SequenceParameterSet`, and the
      reassembler is taken from `formats/h264::FuAReassembler`;
   d. the modules `engines/airplay/src/plist/**` and the SPS parser of
      `engines/airplay/src/h264.rs` are removed.
5. **The Miracast limits are read, not written.** The RTSP reader of the
   Miracast engine takes its limits from the limit set of the core instead
   of from constants of its own. The public function `rtsp::read_request`
   keeps its signature; the limits are handed to it by the caller, which
   gets them from the configuration.
6. **The result is checked, not reviewed.** The dependency direction is
   checked by `scripts/check-crate-boundaries.sh` (ADR-0001). A format crate
   that grows an `std::net`, an `std::fs` or a `phairplay_core` import is a
   finding; a rule that is only written down is a rule that is broken
   quietly.

## Consequences

### Positive

- One property-list decoder and one H.264 parser for the whole workspace.
  A fix for a crafted document lands in every engine at once.
- The hardened decoders are the ones that are actually used. The
  `formats/plist` crate carries the negative tests of the build contract —
  a cyclic object graph, a length field that is not an allocation size — and
  today the AirPlay engine does not use them.
- The format crates become reusable outside the receiver, which is what
  makes them worth their own crate.
- The engine crates shrink by roughly 1900 lines, which is a measurable
  reduction of the attack surface of the receiver.

### Negative / Trade-offs

- The migration touches the AirPlay engine in almost every module that
  reads a property list or a parameter set, and it touches the public API of
  the crate. A re-export keeps the source compatible for a caller, but a
  caller that names the *type* of a variant of `PlistValue` has to follow
  the rename of the variants.
- The two codecs are not identical. `formats/plist` has a `Value` with the
  types of the specification and a `PlistLimits`; the codec of the AirPlay
  engine has a `PlistValue` with a slightly different shape and a
  `max_bytes` parameter. The adapter has to bridge the two, and every
  difference is a place where a test has to prove that the behaviour did not
  change.
- The format crates cannot use the error type of the core, so a conversion
  sits on the edge of every engine. That is the price of purity and it is
  paid four times.
- The Miracast change makes the RTSP reader take a parameter. A pure
  function becomes a function with a configuration, and its tests have to
  build one.

### What the decision makes impossible

- A format crate may not import `std::net`, `std::fs` or a platform API.
  A format crate that needs the network is not a format crate.
- An engine crate may not contain a decoder of a format that exists in
  `formats/*`. "It is only a few lines" is not an exception; the whole
  point of the level is that the decoder is written once.
- A protocol crate may not depend on another protocol crate. The Cast engine
  does not use the AirPlay SDP parser, and the DLNA renderer does not use
  the Cast framing.

## Alternatives Considered

- **Leave the two codecs in the AirPlay engine and declare the format level
  optional.** Rejected: the level would then exist on paper only, and the
  next engine would copy the AirPlay engine instead of the format crate.
  A boundary that is sometimes crossed is not a boundary.
- **Merge the two property-list codecs into one crate and keep it inside
  `engines/airplay`.** Rejected: the Cast engine would then have to depend
  on the AirPlay engine, which breaks the direction of ADR-0001 and makes a
  protocol depend on a protocol. The codec belongs to the level, and the
  level is `formats/*`.
- **Move the codec of the AirPlay engine into `formats/plist` and delete the
  codec of `formats/plist`.** Rejected: the codec of `formats/plist` is the
  one with the negative tests of the build contract (a cyclic graph, an
  unchecked length field) and the one with the limits. The AirPlay codec is
  the one that was written second.
- **Let the engines keep their own limits instead of reading the
  configuration.** Rejected: the build contract says the opposite, and the
  reason is the threat model. A limit that the user cannot see is a limit
  that nobody audits.
- **Let `formats/*` depend on the core for the error type.** Rejected for
  now, see ADR-0001. It is the most likely future change of this ADR.

## Verification

- `bash scripts/check-crate-boundaries.sh` proves that no `formats/*` crate
  depends on an `engines/*` crate and that the AirPlay engine may depend on
  the format crates.
- `formats/plist` carries the negative tests of the build contract: a
  document with a cyclic object graph, a length field that is used as an
  allocation size without a check, a document that is too large.
- `formats/h264` carries the overflow test of the Exp-Golomb reader
  (FR-056) and the tests of the FU-A reassembly.
- The migration is complete when `engines/airplay` has no module `plist`
  and no SPS parser of its own, when the dependency list of
  `engines/airplay/Cargo.toml` names `formats/plist` and `formats/h264`,
  and when the test suite of the workspace is green.
- A crate-level check that a `formats/*` crate does not import `std::net` or
  `std::fs` is a proposed follow-up; it is not part of the decision yet.
