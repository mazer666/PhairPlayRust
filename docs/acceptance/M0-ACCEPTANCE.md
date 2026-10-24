# Acceptance — M0 Foundation and the first implementation of the four protocols

**Requirement IDs:** FR-001 … FR-095, NFR-PERF-001 … NFR-DOC-003
**Milestone:** M0 — Foundation, and the first implementation of the four receiver engines
**Build/commit:** `main` at the time of writing
**Tester/agent roles:** Lead engineer (integration), Rust QA Engineer (gate), Rust Security Engineer (adversarial review), Rust Architect (boundaries), Verifier (independent re-checks)
**Date:** 2026-10-24
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, 8 GB RAM. **No receiver hardware was available** — see §3 and §9.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The whole workspace compiles | no error | **pass** | `cargo build --workspace` → `Finished`, exit 0 |
| The whole workspace passes its tests | no failure | **pass** | `cargo test --workspace` → **1054 passed, 0 failed** (1015 unit/integration + 39 doctests) |
| The property-list codec reads and writes XML | round trip | **pass** | `phairplay-plist` → 84 passed, 0 failed |
| The property-list codec reads and writes the binary format | round trip | **pass** | `phairplay-plist` → 84 passed, 0 failed; `xml_plist_hardening` → 9 passed |
| The H.264 parser reads a sequence parameter set | width and height | **pass** | `phairplay-h264` → 51 passed, 0 failed |
| The ALAC decoder decodes a frame | the samples of the frame | **pass** | `phairplay-alac` → 61 passed, 0 failed |
| The AirPlay engine serves a sender and reports the session | a started and an ended event | **pass** | `phairplay-airplay` → 230 passed, 0 failed |
| The Cast engine answers the receiver handshake | the receiver status | **pass** | `phairplay-cast` → 80 passed, 0 failed; `cast_engine_hardening` → 6 passed |
| The Miracast engine serves a sender | a started and an ended event | **pass** | `phairplay-miracast` → 35 passed, 0 failed |
| The DLNA engine answers the SOAP actions | the service description | **pass** | `phairplay-dlna` → 119 passed; `control_plane` → 10 passed |
| The discovery layer registers the four services | four service types | **pass** | `phairplay-discovery` → 51 passed, 0 failed |
| The bridge carries a value out and back | the same value | **pass** | `bridge_roundtrip` → 11 passed; `foreign_abi_roundtrip` → 2 passed |
| The supervisor starts and stops the engines | the lifecycle | **pass** | `supervisor_lifecycle` → 9 passed, 0 failed |
| The dashboard answers the API | the state of the receivers | **pass** | `phairplay-web` → 16 passed, 0 failed |

**Per-crate result of `cargo test --workspace`:**

| Crate | Unit | Integration | Doctests |
|---|---:|---:|---:|
| `phairplay-core` | 80 | — | 17 |
| `phairplay-plist` | 84 | 9 | 9 |
| `phairplay-h264` | 51 | — | 9 |
| `phairplay-aac` | 24 | — | 6 |
| `phairplay-alac` | 61 | — | 5 |
| `phairplay-discovery` | 51 | — | 8 |
| `phairplay-airplay` | 230 | — | 30 |
| `phairplay-cast` | 80 | 6 | 11 |
| `phairplay-dlna` | 119 | 10 | 10 |
| `phairplay-miracast` | 35 | — | 4 |
| `phairplay-ffi` | 38 | 22 | 14 |
| `phairplay-web` | 16 | — | 4 |

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh` → exit 0. The script was proved with a deliberate violation: a `formats/*` crate that depended on an `engines/*` crate made the gate red, the violation was removed and the gate became green again.
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh` → exit 0. The script compares **16** mirrored budgets with the typed Rust model and checks **7** budgets that the configuration file owns alone; **256** internal links resolve, **0** are broken.
- The dependency direction is unchanged: **yes** — the same gate, and the five ADRs under `docs/adr/` record the decisions.
- The tests are green: **yes** — 1054 passed, 0 failed.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **yes** — no warning, exit 0.
- The format check is green (`cargo fmt --all -- --check`): **yes** — exit 0.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** — exit 0.
- The licence headers are present: **yes** — `bash scripts/check-license-headers.sh` → exit 0.
- The length of the source files is within the limit: **yes** — `bash scripts/check-file-size.sh` → exit 0.
- The dependency policy is satisfied: **yes** — `bash scripts/audit-licenses.sh` → `bans ok, licenses ok, sources ok` with the **pinned** cargo-deny 0.20.2.
- The versioning or the migration is checked: **not applicable** — the project is at version 0.1.0 and has no migration to prove.

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | not measured | **NOT RUN** |
| Sustained frame rate | `throughput.minimum_fps` | not measured | **NOT RUN** |
| Resident memory | `resources.max_memory_mb` | not measured | **NOT RUN** |
| CPU usage | `resources.max_cpu_percent` | not measured | **NOT RUN** |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | not measured | **NOT RUN** |

**Why:** every one of these budgets is a property of the *running* receiver on a
*real* device, with a *real* sender. The sandbox has neither a television nor
a sender, and a measurement on a build server would be a number without
meaning. The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml) and
are **not** repeated here.

**How they will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md)) names
the sender, the command and the expected result for every one of them. The
acceptance record of the milestone that runs on the hardware will carry the
measured values.

## 4. Usability

- The number of steps of the task: **not measured on a device** — the shells
  are built, the device test is pending.
- The discoverability notes: the dashboard shows one card per protocol with
  the state, the port, the number of peers and live counters; the four
  states — *disabled*, *advertising*, *streaming*, *error* — are carried by a
  label and a glyph as well as by a colour, so that the state is readable
  without colour vision.
- The distinction of the states: **proved.** A protocol that is switched off
  in the configuration reads *disabled*; a protocol that failed to start
  reads *error* together with the reason. The distinction is a requirement of
  the dashboard and of the shells (`FR-042`, `FR-043`).
- The clarity of an error message: every failure of the core carries a typed
  error and a stable numeric code (`phairplay_core::error::ErrorCode`); the
  dashboard shows the reason of the last failure of a protocol.
- The mobile / native behaviour: **NOT RUN** — the device test plan.

## 5. Accessibility

- The keyboard operation: the dashboard is a web page and is fully
  operable with the keyboard; the focus is visible, and the toggles are real
  buttons. Proven in a real browser by the dashboard's own end-to-end check.
- The screen reader (VoiceOver / TalkBack): the dashboard carries ARIA
  attributes and accessible names; the shells use the platform semantics.
  **NOT RUN** with a screen reader — that needs a device.
- The scaling, the high contrast, the reduced motion: the palette is
  measured — **59** colour pairs, every text pair at **≥ 4.5:1** — and the
  motion is switched off under `prefers-reduced-motion`. The measurement is
  a script, not an estimate.

## 6. Security and privacy

| Measure | Result | Evidence |
|---|---|---|
| Every network input is length-limited | **pass** | the limits live in `AppConfig::limits`; `phairplay-core` → 80 passed |
| Every blocking read carries a deadline | **pass** | the idle timeout of the core; `phairplay-miracast` → 35 passed |
| Every queue is bounded by a byte budget as well | **pass** | `queues.video_queue_bytes` and `queues.audio_queue_bytes`; the queue of the core is covered by its own tests |
| The property-list decoder refuses a cyclic object graph | **pass** | `xml_plist_hardening` → 9 passed |
| The SPS parser cannot overflow on 31 leading zero bits | **pass** | `phairplay-h264` → 51 passed |
| The ALAC decoder cannot overflow on a narrow sample with a shift | **pass** | `phairplay-alac` → 61 passed; the regression test fails without the fix |
| The SOAP parser refuses a DOCTYPE | **pass** | `phairplay-dlna` → `hardening` 11 passed |
| The GENA callback guard refuses an address outside the subnet | **pass** | `phairplay-dlna` → `hardening` 11 passed |
| The Cast engine refuses a frame that announces too much | **pass** | `cast_engine_hardening` → 6 passed |
| The log does not name a peer | **pass** | `phairplay-core` → `privacy` (masking of the name, the address and the socket address) |
| The dashboard binds to the loopback interface only | **pass** | `phairplay-web` → 16 passed |
| The dependency policy is satisfied | **pass** | `cargo-deny` → `bans ok, licenses ok, sources ok` |
| A dependency has no known advisory | **NOT RUN** | `cargo audit` is part of the CI, but it is not installed in this sandbox — the CI job `audit` is the evidence, not this run |

The threat model and the guarantees are written down in
[`SECURITY.md`](../../SECURITY.md). The data that the receiver sees, and the
data it does not keep, is written down in
[`PRIVACY.md`](../../PRIVACY.md).

## 7. Interoperability

| Counterpart | Expected | Result | Evidence |
|---|---|---|---|
| An Apple sender (macOS, iOS) | the handshake | **NOT RUN** | needs hardware — `docs/guides/DEVICE_TEST_PLAN.md` |
| A Windows sender ("Cast to Device") | the handshake | **NOT RUN** | needs hardware |
| VLC and other UPnP control points | the description and the SOAP answers | **NOT RUN** | needs a control point |
| A Cast sender | the receiver handshake | **NOT RUN** | needs a sender |

What *is* proved is the side of the protocol that does not need a peer: the
parsers, the answers and the limits. The interoperability of a real sender
is a device test and is recorded as **NOT RUN** here — not as passed.

## 8. Known limitations

1. **No hardware test has been run.** Every interoperability row and every
   performance row above is NOT RUN. The project does not claim otherwise.
2. **The media path is the platform's job.** The engines deliver *encoded*
   frames to a sink; the decoding belongs to the native shell. A receiver
   that cannot decode a codec does not advertise it.
3. **The Miracast group formation is a platform concern.** The engine
   implements the WFD control plane; the Wi-Fi Direct group needs a system
   API of the host.
4. **FairPlay DRM content cannot be played.** No part of this project
   attempts to decrypt it (`EXC-001`).
5. **Apple Music in-app audio is protected** on every AirPlay path
   (`EXC-002`).
6. **The Android and macOS shells are built, but their Gradle and Xcode
   builds were not run here** — the sandbox has no Android SDK licence and
   no macOS SDK. The status is recorded in the deliverable of the platform
   worker as NOT RUN with the reason.

## 9. Decision

**ACCEPTED with reservations.**

The workspace compiles, the lint is clean, **1054** tests pass, the four
protocol engines, the five format crates, the bridge and the dashboard are
implemented and covered by tests, the structural gates and the licence gate
are green, and the documentation is consistent with the code.

The reservations are named, not hidden:

- The **device tests are outstanding**. This acceptance record proves the
  software, not the product on a television. A milestone may not be
  *announced* on the strength of this record; it may only be *announced*
  together with the device evidence of the device test plan.
- **The performance budgets are unmeasured.** They are budgets, and a
  budget is a target that a measurement has to prove.
- **The native shells are unbuilt here.** The FFI contract is proved by two
  round-trip tests, but the Kotlin and Swift sources were not compiled in
  this sandbox.

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.
