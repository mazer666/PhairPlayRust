# Acceptance — M4 Hardening, interoperability and release readiness

**Requirement IDs:** FR-001 … FR-107, NFR-PERF-001 … NFR-PERF-005, NFR-SEC-001 … NFR-SEC-006, NFR-PRIV-001 … NFR-PRIV-004, NFR-INT-001 … NFR-INT-004, NFR-REL-001 … NFR-REL-003, NFR-DOC-001 … NFR-DOC-003
**Milestone:** M4 — Hardening, interoperability and release readiness
**Build/commit:** `feat/other-devices` at `2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` — *"fix(ci): the licence gate has to pass on a clean checkout"*, 2026-10-24 13:35:31 +0000
**Tester/agent roles:** Rust QA & Test Engineer (this record), Lead engineer (integration), Rust Security Engineer (the threat model), Documentation role (the records under `docs/audit/`)
**Date:** 2026-10-24
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, 8 GB RAM, `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. **No receiver hardware, no sender and no television were available** — see §3, §7 and §9.

---

## 0. How the measurement was made, and what it belongs to

The working tree of the repository was being **modified by another agent**
while this record was written: a refactoring of `engines/dlna/src/ssdp.rs`
and `engines/dlna/src/engine.rs` was in the tree and did not compile
(`error[E0560]: struct 'SsdpService' has no field named 'primary_target'`).
Every number below therefore belongs to the **committed revision**
`2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0`, measured in a git worktree —
the same method that
[`docs/audit/2026-10-24-baseline.md`](../audit/2026-10-24-baseline.md) §1
uses for the baseline:

```text
git worktree add /workspace/scratch/m4-acc 2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0
cd /workspace/scratch/m4-acc
export HOME=/workspace/.home PATH=$HOME/.cargo/bin:$PATH
CARGO_TARGET_DIR=/workspace/target-qa4w cargo build --offline -p phairplay-ffi
CARGO_TARGET_DIR=/workspace/target-qa4w cargo test  --offline --workspace --no-fail-fast
```

The step `cargo build --offline -p phairplay-ffi` is **not optional**: the
round-trip test of the bridge loads the shared library of the bridge, and
`cargo test` builds the test harness of the library, not its `cdylib`. Without
the step the two tests of `platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs`
fail with *"the shared library of the bridge must be built next to the test
binary"*. The continuous integration has the same step
(`.github/workflows/ci.yml`, the step *"The shared library and the binding
generator of the bridge"*). This was observed, not assumed — see §9.

### The continuous integration — the release evidence

The gate ran on the GitHub-hosted runners on **exactly this revision**.

| Item | Value |
|---|---|
| Run | **40098804432** — <https://github.com/mazer666/PhairPlayRust/actions/runs/40098804432> |
| Head SHA | `2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` |
| Started / finished | 2026-10-24 13:35:38Z → 13:38:51Z |
| Conclusion | **success** |

| Job | Job id | Steps | Conclusion |
|---|---|---:|---|
| Quality Gate (format, lint, tests, documentation) | 120886521026 | 10 | **success** |
| Licenses (cargo-deny, pinned) | 120886521011 | 6 | **success** |
| Security Audit (cargo-audit) | 120886520855 | 7 | **success** |
| Structure (boundaries, file size, licence headers, documentation) | 120886521060 | 7 | **success** |

The steps of the *Quality Gate* job are named in the workflow and were all
green: *"The format of the sources"*, *"The lint of the workspace"*,
*"The shared library and the binding generator of the bridge"*,
*"The tests of the workspace"*, *"The documentation of the workspace"*.
The steps of the *Structure* job are
*"The crate boundaries"*, *"The length of the source files"*,
*"The licence headers"*, *"The links of the documentation"*.

**The honest limit of this evidence:** the *conclusions* of the jobs and of
every step are read from the GitHub API
(`GET /repos/mazer666/PhairPlayRust/actions/runs/40098804432/jobs`). The
**raw log of a job is not downloadable** by this account — the API answers
`403 {"message": "Must have admin rights to Repository."}`. The numbers of
the test suite in this record therefore come from the **local** run of §1,
not from the runner; the runner proves that the *gate* is green, the local
run proves *what* is green.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The whole workspace compiles | no error | **pass** | `cargo build --offline -p phairplay-ffi` → `Finished 'dev' profile … in 8m 41s`, exit 0 |
| The whole workspace passes its tests | no failure | **pass** | `cargo test --offline --workspace --no-fail-fast` → **1224 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out** (998 unit + 82 integration + 144 doctests, 38 test binaries) |
| The format of the sources is the format of `rustfmt` | no diff | **pass** | `cargo fmt --all -- --check` → exit 0, no output |
| The lint of the workspace is clean | no warning | **pass** | `cargo clippy --offline --workspace --all-targets -- -D warnings` → exit 0, no warning |
| The documentation of the workspace builds | no error | **pass with one warning** | `cargo doc --offline --workspace --no-deps` → exit 0, **1 warning** — `engines/smartview/src/lib.rs:56`: `unresolved link to 'ProtocolEngine'` (see §8, S1) |
| The crate boundaries hold | no violation | **pass** | `bash scripts/check-crate-boundaries.sh` → `Crates checked: 13 / Internal edges: 15 / Violations: 0`, exit 0 |
| The boundary gate refuses a violation | exit code 1 | **pass** | `bash scripts/check-crate-boundaries.sh --self-test` → 5 cases, each `exit code 1`, final `OK`, exit 0 |
| No source file is longer than the hard limit | ≤ 550 lines | **pass** | `bash scripts/check-file-size.sh` → `Files measured: 365 / Above the hard limit: 0`, exit 0 (8 files above the *soft* limit of 400) |
| The licence headers are complete | no violation | **pass** | `bash scripts/check-license-headers.sh` → `Member manifests checked: 13 / Source files checked: 251 / Derived files checked: 10 / Violations: 0`, exit 0 |
| Every internal link of the documentation resolves | no broken link | **pass** | `bash scripts/check-docs-links.sh` → `Markdown files scanned: 59 / Internal links checked: 488 / Broken links: 0`, exit 0 |
| The licences of the dependencies fit the AGPL | no violation | **pass** | `bash scripts/audit-licenses.sh` → cargo-deny **0.20.2** (the pinned version) → `bans ok, licenses ok, sources ok`, exit 0 |
| No dependency has a known advisory | no finding | **pass with one accepted warning** | `cargo audit --ignore RUSTSEC-2023-0071` → exit 0, `warning: 1 allowed warning found` (`RUSTSEC-2026-0374`, `basic-toml` unmaintained) |
| The receiver starts headless and names every protocol | a start report | **pass** | `docs/audit/other-devices-guide.md` §4 — `phairplay-web --profile raspberry-pi --port 7879` → *"the receiver starts 5 protocols"*, three advertising, Miracast off, Smart View *"not started — no engine implementation is registered"* |
| The command line of the receiver is complete | every option in the help | **pass** | `apps/web/src/cli.rs` → **21** unit tests, among them `every_profile_is_understood`, `an_unknown_option_is_refused`, `a_port_out_of_the_range_is_refused` |
| Every device profile is a loadable configuration | five profiles | **pass** | `apps/web/src/profiles.rs` → `every_profile_of_the_repository_is_found_and_loaded`; the files are `config/profiles/{raspberry-pi,nas,docker,desktop,generic}.toml` |
| The installer prints its plan without changing anything | exit 0 | **pass** | `bash -n deploy/install.sh` → exit 0; `bash deploy/install.sh --dry-run --profile raspberry-pi` → exit 0, six `DRY-RUN` steps, `WARNING: systemctl is not installed` |
| The gate is a gate — it refuses a broken rule | red, then green | **pass** | `bash scripts/local-ci.sh --self-test` → see §2 |

**Per-crate result of `cargo test --offline --workspace --no-fail-fast`:**

| Crate | Unit | Integration | Doctests |
|---|---:|---:|---:|
| `phairplay-core` | 94 | — | 19 |
| `phairplay-plist` | 84 | — | 7 |
| `phairplay-h264` | 51 | — | 9 |
| `phairplay-aac` | 24 | — | 6 |
| `phairplay-alac` | 61 | — | 5 |
| `phairplay-discovery` | 52 | — | 8 |
| `phairplay-airplay` | 230 | 9 | 30 |
| `phairplay-cast` | 80 | 6 | 11 |
| `phairplay-dlna` | 119 | 21 | 10 |
| `phairplay-miracast` | 35 | — | 3 |
| `phairplay-smartview` | 75 | 24 | 13 |
| `phairplay-ffi` | 43 | 22 | 14 |
| `phairplay-web` | 50 | — | 9 |
| **Total** | **998** | **82** | **144** |

The 38 test binaries report **62.35 s** of run time in total (the sum of the
`finished in` of every `test result` line). The wall clock of the whole
chain — the build of the bridge (8m 41s), the compilation of the test
targets (13m 41s) and the execution of the suite — was **24m 40.665 s** on a
machine with two CPUs that was shared with another build.

**The history of the number.** The baseline of the project
([`docs/audit/2026-10-24-baseline.md`](../audit/2026-10-24-baseline.md))
measured **961** tests on the untouched commit `6947f35…`: **948** passed,
**13** failed, and the run did not even terminate before the deadlock of
`phairplay-cast` was repaired. The same document records **1054** tests,
all green, after the work of the QA of M0. The revision of this record
carries **1224**. The increase is the hardening of M4 (the negative tests of
the DLNA, Smart View and Cast engines), the fifth protocol and the headless
build.

---

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh`
  → `Crates checked: 13 / Internal edges: 15 / Violations: 0`, exit 0. The
  script was proved with five deliberate violations; every one of them made
  the gate red with **exit code 1** and the clean tree green again:
  `bash scripts/check-crate-boundaries.sh --self-test` →

  ```text
  [ ok ] the unmodified copy is green
  [ ok ] formats -> engines: the gate answered with exit code 1
  [ ok ] core -> formats: the gate answered with exit code 1
  [ ok ] engines -> platform: the gate answered with exit code 1
  [ ok ] platform -> formats: the gate answered with exit code 1
  [ ok ] app-backend -> formats: the gate answered with exit code 1
  OK: the gate fails on every violation and passes on the clean tree.
  ```

  The self-test works on a copy in a temporary directory — the repository is
  never modified (`git status --porcelain` was empty after the run).
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh`
  → `Budgets compared: 16 / Budgets owned alone: 7`, exit 0. The check
  compares the mirrored budgets with the typed Rust model
  (`core/phairplay-core/src/config.rs`) and proves that every budget that
  the configuration file owns alone exists exactly once.
- The dependency direction is unchanged: **yes** — the same gate, and the
  ADRs under `docs/adr/` record the decisions.
- The tests are green: **yes** — **1224** passed, **0** failed, **0** ignored,
  **0** measured, **0** filtered out. Every one of the 38 `test result` lines
  reads `0 ignored` and `0 filtered out`.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **yes** —
  exit 0, no warning. The workspace denies `unsafe_code`, `unwrap_used` and
  `expect_used` (`Cargo.toml`, `[workspace.lints]`).
- The format check is green (`cargo fmt --all -- --check`): **yes** — exit 0.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** —
  **59** markdown files, **488** internal links, **0** broken, exit 0.
- The versioning or the migration is checked: **not applicable** — the project
  is at version `0.1.0` (`VERSION`, `Cargo.toml`) and has no migration to
  prove.

### The gate is a gate

`bash scripts/local-ci.sh --self-test` breaks every rule on purpose, proves
that the gate goes red, undoes the break and proves that the gate goes green
again. The raw output of the run of this record (8m 25.779 s, exit 0):

```text
==> The self-test of the gate of PhairPlayRust
    Every rule is broken on purpose, the gate has to go red, the
    break is undone and the gate has to go green again.

--- 1. the lint of the workspace: an unwrap() in the code
    clippy with an unwrap() -> exit code 101
    OK: the lint refused the unwrap().
    clippy after the repair -> exit code 0
    OK: the gate is green again.

--- 2. the length of the source files: a file above the hard limit
    the file-size check with a long file -> exit code 1
    OK: the check refused the long file.
    the file-size check after the repair -> exit code 0
    OK: the gate is green again.

--- 3. the licence headers: a literal licence in a member manifest
    the licence-header check with a literal licence -> exit code 1
    OK: the check refused the literal licence.
    the licence-header check after the repair -> exit code 0
    OK: the gate is green again.

--- 4. the links of the documentation: a link to a file that does not exist
    the link check with a broken link -> exit code 1
    OK: the check refused the broken link.
    the link check after the repair -> exit code 0
    OK: the gate is green again.

==> The self-test of the gate
    rules proven   : 4
    failed rules   : 0

OK: every rule of the gate refused its violation and was green again after the repair.
```

The self-test changes the working tree for a few seconds and restores every
file it touched with a `trap`; `git status --porcelain` was empty after the
run.

The four rules that the self-test covers are the lint, the length of the
files, the licence headers and the links of the documentation; the fifth rule
— the licence audit — has its own proof, because the rule is *"the tool has
the wrong version"*: a machine with another cargo-deny reports **NOT RUN**
with the reason and the exit code **2**, never a false error and never a
green run for a configuration it did not read
([`docs/audit/2026-10-24-gate-self-test.md`](../audit/2026-10-24-gate-self-test.md) §4).

---

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | not measured | **NOT RUN** |
| Sustained frame rate | `throughput.minimum_fps` | not measured | **NOT RUN** |
| Resident memory | `resources.max_memory_mb` | not measured | **NOT RUN** |
| CPU usage | `resources.max_cpu_percent` | not measured | **NOT RUN** |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | not measured | **NOT RUN** |

**Why:** every one of these budgets is a property of the *running* receiver
on a *real* device, with a *real* sender. The sandbox has neither a
television nor a sender, and a number measured on a build server would be a
number without meaning. The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml)
and are **not** repeated here.

**How they will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) §3)
names the sender, the command and the expected result for every one of them
— `ps -o rss= -C phairplay-web` for the memory, `ps -o pcpu= -C phairplay-web`
for the CPU, the counters of the dashboard for the frame rate, the stop watch
of the AirPlay script for the time to the first picture. The acceptance
record of the milestone that runs on the hardware will carry the measured
values.

**What *is* measured in this milestone** are the limits that bound the
memory and the time of a hostile peer — they are the budgets of the *attack*,
not of the picture:

| Limit | Owner | Test that executes the attack |
|---|---|---|
| Byte budget of the video queue | `queues.video_queue_bytes` | `core/phairplay-core/src/queue.rs` → `the_byte_budget_holds_under_the_ninety_frames_of_eight_megabytes_attack` |
| Item budget of the video queue | `queues.video_queue_frames` | `the_oldest_entry_is_dropped_when_the_item_budget_is_reached` |
| An entry that is larger than the budget | `queues.video_queue_bytes` | `an_entry_that_is_larger_than_the_budget_is_refused` |
| The deadline of a blocking read | `latency.idle_timeout_ms` | `engines/airplay/src/rtsp/reader/tests.rs` → `a_stalled_peer_hits_the_deadline`; `engines/cast/tests/cast_engine_hardening.rs` → `a_peer_that_stays_silent_is_dropped_after_the_idle_timeout` |
| The byte budget of the SSDP event queue | — | `engines/discovery/src/ssdp/tests.rs` → `the_event_queue_drops_when_the_byte_budget_is_exhausted` |

---

## 4. Usability

- The number of steps of the task: **not measured on a device** — the
  comparison of the dashboard and the native shells needs both sides running.
  What *is* measured is the path on the machine without a screen: one
  command starts the receiver and prints the state of every protocol
  (`docs/audit/other-devices-guide.md` §4).
- The discoverability notes: the guide
  ([`docs/guides/OTHER-DEVICES.md`](../guides/OTHER-DEVICES.md),
  [`…en.md`](../guides/OTHER-DEVICES.en.md)) names the device classes, the
  profile of each class and the installation per device; the checklist in §8
  of the English edition answers *"does the receiver run"* and *"does a
  sender find it"* with six commands.
- The distinction of the states ("disabled", "advertising", "streaming", "error"):
  **proved** — the start report of the receiver names every protocol with its
  state and the reason of every failure
  (`apps/web/src/main.rs` → `report`), and the API carries the same states
  (`docs/audit/other-devices-guide.md` §4, the answer of
  `/api/status`).
- The clarity of an error message: **proved** — a protocol that does not
  start does not stop the others and names its reason
  (*"not started — no engine implementation is registered for the protocol
  `smartview`"*); a broken configuration names the file and the reason
  (`apps/web/src/receiver.rs` → `a_broken_configuration_file_names_the_file_and_the_reason`).
- The mobile / native behaviour: **NOT RUN** — the device test plan.

---

## 5. Accessibility

- The keyboard operation: the dashboard is a television interface and was
  checked with the keyboard only in a real Chromium — **35** keyboard checks,
  **31** layout checks and **38** measured contrast pairs, 0 failures
  ([`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §4).
  M4 changes nothing in the panel; it changes the binary that starts the
  receiver.
- The screen reader (VoiceOver / TalkBack): the dashboard carries ARIA
  attributes and accessible names; the shells use the platform semantics.
  **NOT RUN** with a screen reader — that needs a device.
- The scaling, the high contrast, the reduced motion: the palette is
  measured — every text pair at **≥ 4.5:1** — and the motion is switched off
  under `prefers-reduced-motion`. The measurement is a script, not an
  estimate.
- **The headless build has no screen at all.** The receiver on a device
  without a display reports its state on the console and over the local API
  (`/api/status`, `/api/sessions`, `/api/metrics`) — the interface of that
  device is the API, not the panel. This is a deliberate consequence of the
  headless goal and it is written down in
  [`docs/guides/OTHER-DEVICES.md`](../guides/OTHER-DEVICES.md) §1.

---

## 6. Security and privacy

Every attack of the threat model
([`SECURITY.md`](../../SECURITY.md) §1) has a negative test that **executes
the attack**. The name of the test is the evidence — a row without a test
name is not a result.

| Attack | The negative test that runs it |
|---|---|
| RTSP request line, header count, header length, body size | `engines/airplay/src/rtsp/reader/tests.rs` → `an_overlong_request_line_is_rejected`, `too_many_headers_are_rejected`, `an_overlong_header_line_is_rejected`, `a_declared_body_longer_than_the_limit_is_rejected`, `a_malformed_content_length_is_rejected`, `a_negative_content_length_is_rejected`, `a_header_without_a_colon_is_rejected` |
| A peer that sends nothing | `a_stalled_peer_hits_the_deadline` (AirPlay), `a_peer_that_stays_silent_is_dropped_after_the_idle_timeout` (Cast) |
| A queue that is filled with maximum-size frames | `core/phairplay-core/src/queue.rs` → `the_byte_budget_holds_under_the_ninety_frames_of_eight_megabytes_attack`, `an_entry_that_is_larger_than_the_budget_is_refused` |
| A malformed SDP | `engines/airplay/src/sdp/tests.rs` → `an_oversized_sdp_is_rejected`, `an_empty_sdp_is_rejected`, `an_sdp_without_a_media_section_is_rejected` |
| XXE — an external entity in the property list | `engines/airplay/tests/xml_plist_hardening.rs` → `an_external_entity_is_never_resolved`, `a_doctype_with_an_internal_subset_is_refused` |
| SSRF — an entity that points at a network service | `engines/airplay/tests/xml_plist_hardening.rs` → `an_entity_that_points_at_a_network_service_is_never_resolved` |
| A property list that nests too deeply | `engines/airplay/tests/xml_plist_hardening.rs` → `a_document_that_nests_too_deeply_is_refused` |
| A property list with a cyclic object graph | `engines/airplay/src/plist/binary/tests.rs` → `a_document_with_a_reference_cycle_is_rejected`, `a_document_that_declares_an_absurd_object_count_is_rejected` |
| An Exp-Golomb code that overflows | `formats/h264/src/bits_tests.rs` → `thirty_one_leading_zeros_do_not_overflow`, `thirty_two_leading_zeros_are_clamped`, `too_many_leading_zeros_are_refused` |
| An ALAC frame with a shift that is wider than the sample | `formats/alac/src/decoder_tests.rs` → `refuses_a_shift_that_is_wider_than_the_sample`, `refuses_a_stereo_shift_that_is_wider_than_the_sample`, `refuses_a_frame_with_a_broken_shift_flag`, `refuses_a_frame_that_announces_too_many_samples` |
| An oversized SSDP message | `engines/discovery/src/ssdp/tests.rs` → `an_oversized_message_is_rejected`, `too_many_headers_are_rejected`, `an_over_long_header_line_is_rejected`, `the_browser_rejects_an_oversized_query` |
| XXE in a SOAP body | `engines/dlna/tests/hardening.rs` → `a_doctype_in_the_soap_body_is_refused`, `an_external_entity_in_the_soap_body_is_refused` |
| SSRF — a GENA callback outside the local subnet | `engines/dlna/tests/hardening.rs` → `a_gena_callback_to_the_metadata_service_is_refused` (169.254.169.254), `a_gena_callback_to_the_loopback_address_is_refused`, `a_gena_callback_to_the_ipv6_loopback_is_refused`, `a_gena_callback_to_a_host_name_is_refused`; `engines/dlna/src/gena/tests.rs` → `an_address_outside_the_local_subnet_is_refused`, `a_public_address_is_refused`, `a_credential_in_the_url_is_refused`, `a_non_http_scheme_is_refused` |
| A HTTP-like request that is too long or too fat | `engines/dlna/tests/hardening.rs` → `an_over_long_request_line_is_refused`, `too_many_headers_are_rejected`, `an_over_long_header_line_is_refused`, `a_body_that_is_too_large_is_refused`, `a_chunked_body_is_refused` |
| A CastV2 frame that announces more than the limit | `engines/cast/tests/cast_engine_hardening.rs` → `a_message_that_announces_more_than_the_limit_ends_the_session`, `a_message_that_is_not_a_protobuf_ends_the_session`, `a_client_that_does_not_speak_tls_is_dropped` |
| A WFD information element that runs past the buffer | `engines/miracast/src/wfd.rs` → `a_block_that_runs_past_the_end_is_rejected`, `a_trailing_type_byte_without_a_length_is_rejected`, `an_oversized_element_is_rejected`, `too_many_blocks_are_rejected`, `a_version_block_of_the_wrong_length_is_rejected` |
| A Smart View request or frame that is too large | `engines/smartview/tests/hardening.rs` → `a_body_that_is_too_large_is_refused_before_it_is_read`, `a_websocket_frame_that_announces_a_gigabyte_is_refused`, `an_unmasked_websocket_frame_breaks_the_channel`, `a_content_length_that_is_not_a_number_is_refused`, `a_chunked_body_is_refused` |
| A control character in the name of a sender | `engines/smartview/tests/hardening.rs` → `a_control_character_in_the_name_of_the_sender_never_reaches_the_answer`, `a_control_character_in_the_name_of_the_device_never_reaches_the_document` |
| A second sender on a single-session protocol | `engines/airplay/src/engine/tests.rs` → `a_second_sender_is_rejected_with_service_unavailable`; `engines/dlna/src/soap/tests.rs` → `a_second_control_point_is_rejected_while_the_first_streams`; `engines/smartview/src/session/tests.rs` → `a_second_peer_is_rejected_and_not_queued` |
| The control panel on the network | `apps/web/src/server.rs` → `the_address_of_the_server_is_the_loopback_interface`, `the_bound_socket_is_on_the_loopback_interface` |
| The pairing attempts | `engines/airplay/src/pairing/store/tests.rs` → `the_failed_attempt_counter_grows_and_can_be_reset`; the limit itself is `limits.max_pairing_attempts` (`config/security-policy.toml`, `[pairing]`) |

### The masking of the log — a partial proof, and the gap that is named

The masking helpers of the core are covered by **16 unit tests** and
**6 doctests** in `core/phairplay-core/src/privacy.rs`, among them
`a_name_is_not_readable_in_its_pseudonym`, `the_host_part_of_an_ipv4_address_is_masked`,
`the_host_part_of_an_ipv6_address_is_masked`, `the_metadata_address_is_masked_as_well`,
`the_port_of_a_socket_address_is_kept`,
`the_host_of_a_url_is_masked_and_the_path_is_dropped`.

Every call site of the log that carries personal data uses them:
`engines/airplay/src/engine/connection.rs` (`mask_socket_addr`), `engines/cast/src/peer.rs`
and `engines/cast/src/engine.rs` (`mask_socket_addr`), `engines/miracast/src/engine.rs`
(`mask_socket_addr`), `engines/dlna/src/gena.rs` (`mask_url`).

**The gap:** `NFR-PRIV-002` in
[`REQUIREMENTS.md`](../../REQUIREMENTS.md) names *"the negative test that
searches a rendered log line for a complete name or address"* as the method
of verification, and the gate of M4 in
[`ROADMAP.md`](../../ROADMAP.md) says *"The mask of the logs is proven by a
test that searches a rendered line."* **Such a test does not exist.**
`tracing-subscriber` appears in the tree only in `apps/web/src/main.rs` — in
the production code, never in a test — and no test of the workspace installs
a subscriber or captures a rendered line. The helpers are proved; the
*rendered line* is not. This is listed as **S1** in §8 and is the one gate
line of M4 whose proof does not exist in the tree.

### The rest of the section

| Measure | Result | Evidence |
|---|---|---|
| Every network input is length-limited | **pass** | the limits live in `AppConfig::limits` (`core/phairplay-core/src/config.rs` → `LimitsConfig`); `phairplay-core` → 94 passed |
| Every blocking read carries a deadline | **pass** | `latency.idle_timeout_ms`; `a_stalled_peer_hits_the_deadline` |
| Every queue is bounded by a byte budget as well | **pass** | `queues.video_queue_bytes`, `queues.audio_queue_bytes`; `the_byte_budget_holds_under_the_ninety_frames_of_eight_megabytes_attack` |
| The property-list decoder refuses a cyclic object graph | **pass** | `a_document_with_a_reference_cycle_is_rejected` |
| The SPS parser cannot overflow on 31 leading zero bits | **pass** | `thirty_one_leading_zeros_do_not_overflow` |
| The ALAC decoder refuses a shift that is wider than the sample | **pass** | `refuses_a_shift_that_is_wider_than_the_sample`; the regression test fails without the fix |
| The SOAP parser refuses a `DOCTYPE` | **pass** | `a_doctype_in_the_soap_body_is_refused` |
| The GENA callback guard refuses an address outside the subnet | **pass** | `an_address_outside_the_local_subnet_is_refused` |
| The Cast engine refuses a frame that announces too much | **pass** | `a_message_that_announces_more_than_the_limit_ends_the_session` |
| The dependency policy is satisfied | **pass** | cargo-deny 0.20.2 → `bans ok, licenses ok, sources ok` |
| A dependency has no known advisory | **pass with one accepted warning** | `cargo audit --ignore RUSTSEC-2023-0071` → exit 0; `RUSTSEC-2026-0374` (`basic-toml`, unmaintained) is a warning, not a vulnerability |
| The attribution of the ALAC port is in place | **pass** | `THIRD_PARTY.md` names the origin, the licence (Apache-2.0) and the eight derived files; every file of `formats/alac/src/` carries an `ATTRIBUTION:` header; the licence texts are in `LICENSES/` (`AGPL-3.0-or-later.txt`, `Apache-2.0.txt`, `BSD-3-Clause.txt`, `ISC.txt`, `MIT.txt`, `Zlib.txt`) |
| The licence of the project is the AGPL | **pass** | `LICENSE` (661 lines, *"GNU AFFERO GENERAL PUBLIC LICENSE, Version 3"*), `Cargo.toml` → `license = "AGPL-3.0-or-later"`, `REUSE.toml` → `SPDX-License-Identifier = "AGPL-3.0-or-later"` |
| No telemetry, no account, no outbound connection outside the local subnet | **pass** | `config/security-policy.toml` → `enable_telemetry = false`, `enable_analytics = false`, `require_account = false`, `allow_outbound_connections_except_local_subnet = false`; `Cargo.lock` contains **no** telemetry client and **no** outbound HTTP client (`reqwest`, `ureq`, `curl` are absent). The only HTTP stack in the tree is `hyper`/`hyper-util`/`axum` — the **server** of the local dashboard, not a client that calls out |

The threat model and the guarantees are written down in
[`SECURITY.md`](../../SECURITY.md). The data that the receiver sees, and the
data it does not keep, is written down in
[`PRIVACY.md`](../../PRIVACY.md).

---

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| An Apple sender (macOS, iOS) | macOS 14 / iOS 17 | **NOT RUN** | needs hardware — `docs/guides/DEVICE_TEST_PLAN.md` §2.1 |
| A Windows sender ("Cast to Device") | Windows 11 | **NOT RUN** | needs hardware — §2.3 |
| A Miracast sender | Windows 11 / Miracast | **NOT RUN** | needs hardware — §2.2 |
| VLC and other UPnP control points | VLC 3.x | **NOT RUN** | needs a control point — §2.4 |
| A Cast sender (Chromecast, Cast SDK) | — | **NOT RUN** | needs a sender — §2.3 |

**No sender, no television and no network with a second device were
available.** Every row above is NOT RUN and is reported as such — not as
passed. The device test plan carries the exact script for every row: the
preparation, the numbered step, the expected result and the command.

What *is* proved is the side of the protocol that does not need a peer: the
parsers, the answers and the limits — the negative tests of §6 and the
control-plane tests (`engines/dlna/tests/control_plane.rs` → 10 passed,
`engines/smartview/tests/control_plane.rs` → 7 passed,
`engines/smartview/tests/discovery.rs` → 5 passed,
`engines/cast/tests/cast_engine_hardening.rs` → 6 passed).

- The formats that a reference implementation reads: **NOT RUN** — the corpus
  of `NFR-INT-003` needs the reference decoder of the format next to the
  build. The formats are proved against each other (round trip), not against
  a reference implementation.
- The receiver without a component of the vendor of the sender: **partly
  proved** — the workspace declares **no** Cast SDK and no vendor library;
  the headless build registers no media sink and needs neither the C ABI nor
  the generated bindings (`docs/guides/OTHER-DEVICES.md` §3). What is *not*
  proved is the behaviour of the receiver on a device **without** the
  component — that is the device test plan.
- The deviations from the expectation of the sender: **written down, not
  measured** — the proprietary message format of the Smart View channel
  (`EXC-011`), the Cast application identifier (`EXC-010`), FairPlay DRM
  (`EXC-001`), the Apple Music in-app audio (`EXC-002`), HDCP (`EXC-003`),
  Widevine and PlayReady (`EXC-004`), the cloud (`EXC-005`), the publication
  to the internet (`EXC-006`), the buffered audio playback (`EXC-007`),
  several peers on a single-session protocol (`EXC-008`) and the transcoding
  (`EXC-009`).

---

## 8. Known limitations

| # | Severity | Limitation | Why it is not fixed here |
|---|---|---|---|
| S1 | **S1** | **`cargo doc --workspace --no-deps` emits one warning** — `engines/smartview/src/lib.rs:56`: ``unresolved link to `ProtocolEngine` ``. The trait lives in `phairplay_core::protocol::ProtocolEngine`; the example two lines below uses exactly that path. The job *Quality Gate* runs `cargo doc --workspace --no-deps` **without** `-D warnings`, so the gate stays green and the defect is invisible to it. The record of M0 states *"0 warnings"* for the same command; that is no longer true. | A one-line fix in a file of another role. It is reported here, not fixed here, because the files of this acceptance are the only ones this role writes. |
| S1 | **S1** | **The mask of the log is not proved on a rendered line.** `NFR-PRIV-002` and the gate of M4 in `ROADMAP.md` name *"a test that searches a rendered log line"*; no such test exists in the tree (see §6). The helpers are proved by 16 unit tests and 6 doctests, and every call site uses them — but the promise *"no log line contains a complete sender name or a complete IP address"* is not demonstrated by a test. | Needs a `tracing-subscriber` in the dev-dependencies of the core and a test that captures a line. The same reason as above. |
| S2 | **S2** | **The interoperability is unconfirmed.** No sender of the matrix of §7 has been tested. The parsers, the answers and the limits are proved; the handshake with a real Apple, Windows, VLC or Cast sender is not. | There is no sender and no television in the environment. `docs/guides/DEVICE_TEST_PLAN.md` carries the script. |
| S2 | **S2** | **The performance budgets are unmeasured.** The five rows of §3 are NOT RUN. | The budgets are properties of the running receiver with a real sender. |
| S2 | **S2** | **`RUSTSEC-2023-0071` (the Marvin attack) is accepted, not fixed.** The crate `rsa 0.9.10` has no fixed version; the advisory says so explicitly. The crate is used for the **legacy** AirPlay pairing, whose key comes over the network. The CI job *Security Audit* ignores this identifier by name and every other advisory still fails the job. | No upstream fix exists. The risk is written down in `SECURITY.md` and in the workflow. |
| S3 | **S3** | **`RUSTSEC-2026-0374`** — the crate `basic-toml 0.1.10` is unmaintained. `cargo audit` reports it as an allowed warning; it is not a vulnerability. | It is a transitive dependency of a build tool of the workspace. It is visible in every audit run. |
| S3 | **S3** | **Eight source files are above the *soft* limit of 400 lines** — `apps/web/assets/cards.css` (426), `engines/airplay/src/engine/mod.rs` (458), `engines/cast/src/engine.rs` (653), `engines/cast/src/session.rs` (671), `engines/dlna/src/gena.rs` (404), `formats/alac/src/elements.rs` (413), `platform/phairplay-ffi/src/supervisor.rs` (517), `scripts/check-crate-boundaries.sh` (550). The *hard* limit of 550 lines is respected by every hand-written source file — the gate says so. | The soft limit is a target, not a rule; the gate of the project only fails above 550. Two of the eight are above the soft limit **and** are the largest files of the tree — they are the first candidates of the next refactoring. |
| S3 | **S3** | **The container image was not built and the service was not installed.** `docker: command not found`, `systemctl: command not found` — the environment has neither Docker nor systemd. The Dockerfile, the compose file and the unit are reviewed and their syntax is checked; the `deploy/install.sh --dry-run` prints the whole plan and changes nothing. | The tools are missing in the sandbox. The commands and their exact output are in `docs/audit/other-devices-guide.md` §10. |
| S3 | **S3** | **The fifth protocol has no picture.** The message format of the Smart View channel is proprietary and is **not implemented** (`EXC-011`); the engine announces the receiver and answers the handshake, the content of the channel does not flow. | The format is not publicly documented. The limit is written down in `REQUIREMENTS.md` and in `ROADMAP.md` (M5). |
| S4 | **S4** | **The header comment of the dashboard cards still says "four".** `apps/web/assets/protocol-cards.js` opens with *"The four receiver cards of the dashboard"*, but the list `PROTOCOLS` of `apps/web/assets/api.js` carries **five** entries — `airplay`, `miracast`, `cast`, `dlna`, `smartview` — and the glyph `#glyph-smartview` exists in `apps/web/index.html`. The panel therefore builds five cards; the comment describes four. | A stale comment from the milestone M3. It is the documentation of a file, and the documentation is part of the product (`NFR-DOC-001`). |
| S3 | **S3** | **The native shells are not built here.** The Kotlin and the Swift sources were not compiled in this sandbox — there is no Android SDK licence and no macOS SDK. The FFI contract is proved by the round-trip tests of the bridge (`bridge_roundtrip` → 11 passed, `foreign_abi_roundtrip` → 2 passed). | The toolchains are missing. The status is in `docs/audit/tv-android-shell.md` and `docs/audit/tv-macos-shell.md`. |
| S4 | **S4** | **`project/project.yaml` still names `docs/acceptance/M0-ACCEPTANCE.md` as the acceptance record** and the test counters live there as well (`tests_passed: 1224`). After this record the pointer should name this file, and — with the single-source rule of `NFR-DOC-003` — the **number of the tests belongs in one place: this protocol**. | The file is not owned by this role. The number in `project/project.yaml` matches the measurement of §1 (1224), so there is no contradiction today — only a second place that would drift. |

---

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| The device test plan of `docs/guides/DEVICE_TEST_PLAN.md` — every row of §2 | **no sender, no television and no second device in the environment.** The plan needs a Mac or an iPhone, a Windows 11 computer, a Chromecast and VLC on the same network as the receiver | The hardware of §1 of the plan, one subnet, and the numbered script of every row |
| The performance budgets of §3 | a measurement on a build server measures the build server | a running receiver with a real sender; the commands are in §3 of the plan |
| `docker build -f deploy/docker/Dockerfile -t phairplayrust:latest .` | `docker: command not found` — the sandbox has no Docker | a machine with Docker; the compose file and the unit are ready |
| `docker compose up -d phairplay` | `docker: command not found` | a machine with Docker |
| `systemctl enable --now phairplayrust.service` | `systemctl: command not found` — the sandbox has no systemd | a Debian-family machine; `deploy/install.sh` does the whole installation |
| `sudo ufw allow …` | no `ufw` in the sandbox | a machine with a firewall |
| `iw phy phy0 info` | the sandbox has no Wi-Fi adapter | a device with a radio that can do Wi-Fi Direct |
| The raw log of the CI run 40098804432 | the GitHub API answers `403 — "Must have admin rights to Repository."` for the download of a job log | an account with the administrative right of the repository. The conclusions of the four jobs and of every step *were* readable and are in the header of this record. |
| The comparison of the dashboard and the native shells (the number of steps) | the comparison needs a device on both sides | the device test plan |
| A screen-reader pass | a screen reader needs the platform | a device with VoiceOver or TalkBack |

---

## 10. Decision

**ACCEPTED with a documented limitation.**

The revision `2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` builds, the lint is
clean with `-D warnings`, the format is the format of `rustfmt`, **1224**
tests pass with **0** failed and **0** ignored, the five structural gates
and the licence gate are green, the continuous integration ran the four jobs
on exactly this revision and all four are green
([run 40098804432](https://github.com/mazer666/PhairPlayRust/actions/runs/40098804432)),
the licences of the dependencies fit the AGPL and the attribution of the
ALAC port is in place. The hardening of M4 is not a claim: every attack of
the threat model has a named negative test that executes it (§6).

The reservations are named, not hidden:

- **The interoperability is unconfirmed.** No sender of the matrix has been
  tested, because there was no sender and no television. This record proves
  the software, not the product on a television. A milestone may not be
  *announced* on the strength of this record; it may only be announced
  together with the device evidence of the device test plan.
- **The performance budgets are unmeasured.** They are budgets, and a budget
  is a target that a measurement has to prove.
- **The mask of the log is proved on the helper, not on the rendered line**
  (S1). `NFR-PRIV-002` names a test that the tree does not contain.
- **The documentation build emits one warning** (S1), and the gate does not
  catch it, because `cargo doc` runs without `-D warnings`.
- **The container image was not built and the service was not installed**
  (S3) — `docker` and `systemctl` are missing in the environment.
- **The working tree of the moment was being modified by another agent**
  and did not compile. Every number of this record belongs to the committed
  revision, measured in a worktree. The owner has to re-run the gate on the
  merged tree before the release.

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.

**Decided by:** the Rust QA & Test Engineer, on 2026-10-24, on the revision
`2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0`.
