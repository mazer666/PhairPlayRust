# Acceptance — M2 Protocol breadth and local control

**Requirement IDs:** FR-001 … FR-007, FR-009 … FR-016, FR-032 … FR-051, FR-061 … FR-069, FR-070 … FR-085, FR-086 … FR-095, FR-100 … FR-103, NFR-PERF-001 … NFR-PERF-005, NFR-RES-001, NFR-RES-002, NFR-SEC-001 … NFR-SEC-006, NFR-PRIV-001 … NFR-PRIV-004, NFR-INT-001 … NFR-INT-004, NFR-REL-001 … NFR-REL-003, NFR-DOC-001 … NFR-DOC-003, EXC-003, EXC-004, EXC-008, EXC-009, EXC-010
**Milestone:** M2 — Protocol breadth and local control (`engines/{cast,miracast,dlna}`, `platform/phairplay-ffi`, `apps/web`)
**Build/commit:** branch `feat/other-devices` at `2d0c4b4` ("fix(ci): the licence gate has to pass on a clean checkout")
**Tester/agent roles:** Rust QA Engineer (this record); the accessibility measurements of the panel belong to the workstream of the television shell and are cited from [`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md)
**The head of the branch moved during the round:** the commit `bbc39c8` ("fix: the receiver announces two services over one multicast group", 23:22) landed **after** the measurement of this record. Every number of this document belongs to `2d0c4b4`; the two places where `bbc39c8` changes the picture are named in the row and in L-7.
**Date:** 2026-10-24
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, loopback plus the routable address `172.25.29.225` of the sandbox. **No Windows computer, no Chromecast, no Android sender, no VLC, no television and no remote control were available** — see §3, §7 and §9.

**How the numbers were taken.** Every command was run in a **git worktree of
the commit** `2d0c4b4`, not in the working tree of the repository: other
workstreams were editing the tree while this round was running, and a number
has to belong to a commit.

```text
git worktree add /workspace/scratch/qa12-m1m2 2d0c4b4
cd /workspace/scratch/qa12-m1m2
CARGO_TARGET_DIR=/workspace/target-qa12 cargo test --offline --workspace
```

The raw output of every command is in
[`docs/audit/acceptance-m1-m2.md`](../audit/acceptance-m1-m2.md). **The test
numbers of this document live here**; the other documents of the round link
to this table instead of repeating it.

**The sentence that this milestone stands on:** the three other protocols
answer a real peer, the bridge carries the value without losing it and
without letting a panic through, and the panel is a **local** control
surface that a remote control can operate. **No sender and no television
were available**, so nothing here proves that a Chromecast finds the
receiver or that a Windows computer shows it under "Cast to Device". Those
rows say **NOT RUN**.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The Miracast engine is a protocol engine with a state machine | `Idle → … → Advertising` | **pass** | `phairplay-miracast` → `test result: ok. 35 passed; 0 failed` — `engine::tests::a_fresh_engine_is_idle`, `starting_binds_the_port_and_advertises`, `stopping_a_fresh_engine_fails`, `starting_twice_fails`, `the_engine_reports_miracast` |
| The WFD control plane is answered | the public methods, the sink parameters, the session | **pass** | the same run — `rtsp::tests::options_announces_the_public_methods`, `get_parameter_answers_with_the_sink_parameters`, `setup_answers_with_the_session_and_the_transport`, `the_serialised_answer_carries_the_sequence_number` |
| A WFD information element with a broken length is refused | a typed error, no allocation | **pass** | the same run — `wfd::tests::a_block_that_runs_past_the_end_is_rejected`, `a_trailing_type_byte_without_a_length_is_rejected`, `an_oversized_element_is_rejected`, `too_many_blocks_are_refused`, `a_version_block_of_the_wrong_length_is_rejected` |
| A second Miracast peer is refused | the first session continues | **pass** | the same run — `engine::tests::a_second_sender_is_refused` |
| The Cast engine is a protocol engine | the state is readable | **pass** | `phairplay-cast` → `test result: ok. 80 passed; 0 failed` — `engine::tests::the_engine_starts_in_the_idle_state`, `starting_binds_the_listener_and_advertises_the_service`, `stopping_an_engine_that_does_not_run_fails`, `a_failing_advertisement_fails_the_start` |
| The CastV2 framing is spoken | a frame round trip | **pass** | the same run — `frame::tests::a_framed_message_round_trips`, `a_truncated_frame_is_an_io_error`, `an_empty_frame_is_rejected` |
| A frame with an oversized length is refused without allocating | a typed error | **pass** | the same run — `frame::tests::an_oversized_length_is_rejected_without_allocating`, `writing_an_oversized_payload_is_refused`; `proto::tests::a_declared_length_beyond_the_buffer_is_rejected`, `a_declared_length_beyond_the_limit_is_rejected` |
| The heartbeat is answered | `PING → PONG` | **pass** | the same run — `session::tests::a_ping_is_answered_with_a_pong`, `a_pong_is_answered_with_nothing` |
| The Cast control plane speaks CastV2 over a **real TLS socket** | the session stays open | **pass** | `engines/cast/tests/cast_engine_hardening.rs` → `test result: ok. 6 passed; 0 failed` — `a_valid_message_is_answered_and_the_session_stays_open`, `a_client_that_does_not_speak_tls_is_dropped`, `a_message_that_is_not_a_protobuf_ends_the_session`, `a_peer_that_stays_silent_is_dropped_after_the_idle_timeout`, `the_receiver_publishes_its_service_and_withdraws_it_again` |
| The application identifier is answered | the default media receiver | **pass** | `phairplay-cast` → `session::tests::the_receiver_status_names_the_application`, `a_launch_of_the_own_application_is_accepted`, `a_launch_of_an_unknown_application_is_refused` |
| **Two Cast senders stream at the same time** | two sessions | **NOT TESTED** | no test with two concurrent senders exists in the crate, and the rule is **not** read from the configuration: `session.multi_peer_protocols = ["cast"]` in [`config/app-defaults.toml`](../../config/app-defaults.toml) is read by **no** Rust code (`grep -rn "multi_peer" --include=*.rs .` → no hit); the limit is the constant `DEFAULT_MAX_SESSIONS = 8` in `engines/cast/src/engine.rs:57` — §8, L-1 |
| The media status of a Cast session is reported | the status entry with the item | **pass** (the engine) | `phairplay-cast` → `media::tests::the_status_entry_carries_the_media_item`, `a_load_reports_the_metadata_it_carried`, `play_and_pause_change_the_player_state`, `a_stop_clears_the_session`; `session::tests::a_load_is_answered_with_a_media_status`, `a_load_reports_the_metadata` — the **snapshot of the core** does not carry it: `core/phairplay-core/src/session.rs:186` folds `PlaybackStateChanged` with `=> {}` — §8, L-2 |
| The DLNA engine answers an SSDP search | the answer with `LOCATION`, `USN`, `ST` | **pass** | `phairplay-dlna` → `test result: ok. 119 passed; 0 failed` — `ssdp::tests::a_search_for_everything_is_answered_as_a_renderer`, `the_search_response_carries_the_location_and_the_usn`, `a_search_for_another_device_is_ignored`, `a_search_without_the_man_header_is_ignored` |
| The `alive` and `byebye` announcements are complete | the three headers | **pass** | the same run — `ssdp::tests::the_alive_announcement_is_complete`, `the_byebye_announcement_is_complete`, `the_renderer_announces_itself_under_three_targets` |
| The SSDP responder answers over a real socket | the answer of the running receiver | **pass** | `engines/dlna/tests/control_plane.rs` → `test result: ok. 10 passed; 0 failed` — `the_responder_answers_a_query_on_its_own_port` (unit), `the_device_description_is_served`, `the_service_descriptions_are_served`, `a_soap_action_is_answered`, `a_gena_callback_to_the_local_subnet_is_accepted`, `a_second_control_point_is_rejected_while_the_first_streams`, `the_engine_reports_the_streaming_state`, `the_engine_stops_cleanly_and_frees_the_port`, `an_unknown_method_is_refused`, `a_head_request_is_answered_without_a_body`, `an_unknown_path_is_a_not_found` |
| The device description and the service descriptions are served | well-formed XML with the three services | **pass** | the same run — `description::tests::the_device_description_names_the_media_renderer`, `the_device_description_lists_the_three_services`, `the_service_description_lists_the_actions`, `the_service_description_declares_the_state_variables` |
| The SOAP dispatcher answers the three services | a valid SOAP answer per action | **pass** | `phairplay-dlna` → `soap::tests::every_advertised_action_is_implemented`, `the_media_is_set_and_reported`, `the_transport_state_follows_the_actions`, `the_volume_is_set_and_clamped`, `the_mute_state_is_set`, `a_seek_sets_the_position` |
| An unknown SOAP action is refused | a SOAP fault | **pass** | the same run — `soap::tests::an_unknown_action_is_reported_as_a_fault`, `an_unknown_control_path_is_a_not_found` |
| A subscribed client receives the GENA event it subscribed to | the event | **NOT TESTED** | `gena::notify_subscribers` (`engines/dlna/src/gena.rs:330`) is **never called by a test** — `grep -rn "notify_subscribers" --include=*.rs .` → only the definition and one call inside the crate. The *subscription* is proven (`a_gena_callback_to_the_local_subnet_is_accepted`), the *delivery* is not — §8, L-3 |
| A SOAP body with a `DOCTYPE` is refused | a typed error | **pass** | `engines/dlna/tests/hardening.rs` → `test result: ok. 11 passed; 0 failed` — `a_doctype_in_the_soap_body_is_refused`; unit: `soap::tests::a_doctype_in_the_body_is_rejected`, `xml::tests::a_doctype_is_rejected`, `a_doctype_in_lower_case_is_rejected` |
| A SOAP body with an external entity is refused, and no resource is contacted | a typed error | **pass** | the same run — `an_external_entity_in_the_soap_body_is_refused`; unit: `soap::tests::an_entity_in_the_body_is_rejected`, `xml::tests::an_external_entity_reference_is_rejected`, `an_entity_declaration_is_rejected` |
| A GENA callback at `169.254.169.254` is refused, and the address is never contacted | the refusal | **pass** | the same run — `a_gena_callback_to_the_metadata_service_is_refused`; unit: `gena::tests::the_cloud_metadata_address_is_refused` |
| A GENA callback at `127.0.0.1` and at `::1` is refused | the refusal | **pass** | the same run — `a_gena_callback_to_the_loopback_address_is_refused`, `a_gena_callback_to_the_ipv6_loopback_is_refused`; unit: `gena::tests::the_loopback_address_is_refused`, `the_ipv6_loopback_is_refused` |
| A GENA callback that names a host is refused (it is never resolved) | the refusal | **pass** | the same run — `a_gena_callback_to_a_host_name_is_refused`; unit: `gena::tests::a_host_name_is_refused_because_it_is_never_resolved` |
| The payload limits of DLNA are enforced | the refusal before the allocation | **pass** | the same run — `a_body_that_is_too_large_is_refused`, `too_many_headers_are_refused`, `an_over_long_header_line_is_refused`, `an_over_long_request_line_is_refused`, `a_chunked_body_is_refused` |
| A second DLNA peer is refused | the first session continues | **pass** | `phairplay-dlna` → `state::tests::a_second_peer_is_rejected_while_a_stream_runs`, `only_an_active_transport_blocks_a_second_peer`; `soap::tests::a_second_control_point_is_rejected_while_the_first_streams` |
| The renderer advertises no video rendering | `carries_video() == false` | **pass** | `phairplay-core` → `protocol::tests::only_streaming_protocols_carry_video` (`assert!(!ProtocolKind::Dlna.carries_video())`), `protocol::tests::a_dlna_renderer_advertises_nothing_over_mdns` |
| The bridge is the only way into the core | the shells import only the generated bindings | **pass** | `bash scripts/check-crate-boundaries.sh` → exit 0, *13 crates, 15 internal edges, 0 violations*; `apps/web/Cargo.toml` depends on `phairplay-core` and `phairplay-ffi` only; the Kotlin shell imports `uniffi.phairplay_ffi.*`, the Swift shell imports `phairplay_ffiFFI` / `PhairPlayFFI` — and nothing else of the workspace |
| No panic crosses the bridge, and every failure is a typed error with a code | a typed error, never a panic | **pass** | `platform/phairplay-ffi/tests/bridge_roundtrip.rs` → `test result: ok. 11 passed; 0 failed`; `phairplay-ffi` → `error::tests::the_numeric_codes_match_the_core`, `a_core_error_keeps_its_code_and_its_reason`, `an_unknown_number_is_not_mapped_onto_a_code`, `the_details_of_an_error_are_never_empty`; `phairplay-core` → `error::tests::error_codes_are_stable` |
| A value goes into the bridge and comes out of it unchanged | a round trip per exported type | **pass** | `phairplay-ffi` → `dto::config::tests::the_configuration_survives_the_conversion`, `dto::identity::tests::the_identity_survives_the_conversion`, `dto::media::tests::the_frame_survives_the_conversion`, `dto::protocol::tests::the_status_survives_the_conversion`, `dto::session::tests::the_snapshot_survives_the_conversion`, `dto::stats::tests::the_counters_survive_the_conversion` |
| A **foreign** caller round-trips through the C ABI | the same value | **pass** | `platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs` → `test result: ok. 2 passed; 0 failed` — `a_foreign_caller_round_trips_through_the_c_abi`, `the_bindings_of_every_shell_can_be_generated` |
| The generated bindings are never edited by hand; a regeneration produces no diff | no diff | **NOT MET** | `bash scripts/generate-bindings.sh --offline` → exit 0, but `git status` afterwards shows **2 modified files**: `apps/android/app/src/main/java/uniffi/phairplay_ffi/phairplay_ffi.kt` and `apps/macos/Sources/PhairPlayFFI/phairplay_ffi.swift` — the API checksum of the constructor `Receiver::new` is **31533** in the committed bindings and **5570** in the regenerated ones. The binding was hand-edited in the commit `86c1b81` — §8, L-4 |
| The dashboard binds on the loopback interface only | the loopback, and no foreign address | **pass** | `phairplay-web` → `server::tests::the_address_of_the_server_is_the_loopback_interface`, `the_bound_socket_is_on_the_loopback_interface`, `the_socket_can_be_bound_to_another_interface`; `cli::tests::the_default_interface_is_the_loopback_interface`; **measured on the running receiver**: `GET http://127.0.0.1:7888/api/status` → `200`, `GET http://172.25.29.225:7888/api/status` → **connection refused** (curl exit 7) |
| The dashboard shows the state of every protocol | one card per protocol with the state, the port and the peers | **pass** | the same run — `state::tests::the_state_covers_every_protocol`, `api::tests::the_status_is_answered`, `dto::tests::the_status_is_serialised_with_a_readable_state`; **measured on the running receiver**: the JSON of `GET /api/status` carries the five protocols with `state`, `tcp_port`, `udp_port`, `peer_count` and `last_error` |
| The dashboard can switch a protocol on and off, and the change is visible in the next status | the switch changes the engine | **pass** | **measured on the running receiver**: `POST /api/protocols/cast/disable` → `{"state":"stopped","enabled":false,"tcp_port":null,…}`, the next `GET /api/status` carries the same; `POST /api/protocols/cast/enable` → `{"state":"advertising","enabled":true,"tcp_port":8009,…}`, the next `GET /api/status` carries the same. The unit tests cover the refusal of an unknown name (`api::tests::an_unknown_protocol_is_a_not_found`, `state::tests::an_unknown_protocol_is_refused`) and the answer `404` of the running receiver |
| The dashboard reads the core and invents nothing | every value from the core | **pass** | `apps/web/src/state.rs` reads `receiver().protocol_statuses()`, `receiver().sessions()` and `BuildInfo::current()`; the running receiver answers `GET /api/build-info` with `{"version":"0.1.0","name":"phairplay-core","license":"AGPL-3.0-or-later","repository":"https://github.com/mazer666/PhairPlayRust","rustc_version":"rustc 1.99.1 …","target":"x86_64-unknown-linux-gnu"}` |
| The dashboard delivers the events live | an event without a reload | **partly proven** | `phairplay-web` → `state::tests::a_status_event_becomes_a_dashboard_event`, `dto::tests::the_event_carries_its_kind`; the **WebSocket endpoint itself** (`apps/web/src/events.rs`) has **no** test — §8, L-5 |
| The dashboard loads no external resource | no CDN, no web font, no icon pack | **pass** | `grep -rnE "https?://" apps/web/assets/ apps/web/index.html` → **2 hits**, both the XML namespace of the two inline SVG files (`http://www.w3.org/2000/svg`) — no fetched resource |
| The dashboard is operable with the keyboard, meets the contrast, respects the reduced motion | the checks of the panel | **pass (the checks)** | [`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §1 and §7 — **35 keyboard checks, 31 layout checks, 0 failures**, the focus ring **3 px** and **9.96:1** against the card in the dark theme and **8.46:1** in the light theme; **NOT RUN** on a television with a real remote control |
| The receiver starts the protocols itself, and every protocol gets an outcome | a named outcome per protocol | **pass** | `phairplay-ffi` → `composition::tests::every_protocol_is_answered_for`, `a_protocol_that_is_switched_off_is_not_started`, `the_start_reports_the_protocols_that_run`, `the_engines_are_registered_with_the_supervisor`; **measured on the running receiver**: the program prints `the receiver starts 5 protocols:` and names every one with its state |
| A protocol that cannot start does not stop the others | the other protocols keep running | **pass** | the same run of the receiver: `Samsung Smart View not started — … the SSDP socket could not be opened: Address already in use (os error 98)` **and** `4 of 5 protocols are running`; the four others answer `advertising` in `GET /api/status`. **The cause of this particular failure was removed after the measurement**: at the head `bbc39c8` (23:22) the same run reports `5 of 5 protocols are running` — see L-7 |
| The configuration of the receiver is read from a file that lies over the profile | the file wins, the base stays | **pass** | `phairplay-web` → `receiver::tests::a_file_lays_over_the_profile`, `the_configuration_of_a_profile_is_the_base_of_a_file`, `a_broken_configuration_file_names_the_file_and_the_reason`, `a_missing_configuration_file_names_the_file` |
| Every profile of the repository is found, loaded and valid | every profile | **pass** | the same run — `profiles::tests::every_profile_of_the_repository_is_found_and_loaded`, `a_profile_that_is_missing_is_reported_with_the_searched_directories`, `a_profile_that_cannot_be_read_is_reported_with_its_file` |
| An unknown profile is refused with the list of the known ones | the refusal | **pass** | the same run — `cli::tests::an_unknown_profile_is_refused`, `every_profile_is_understood`, `the_help_names_every_profile` |
| A `--bind` to a foreign address prints a warning on stderr **before** anything is started | the warning names the address | **pass** | **measured**: `phairplay-web --bind 0.0.0.0 --config /nonexistent.toml --port 7879` → stderr: `phairplay-web: WARNING — the dashboard is bound to 0.0.0.0, not to the loopback interface. …` — printed **before** the line about the configuration file |

### The test numbers of M2 — the single source

The command was `cargo test --offline --workspace` in the worktree of
`2d0c4b4`; the exit code was 0 and the whole workspace carried **1224
passed, 0 failed, 0 ignored**. The part that belongs to M2:

| Crate | Unit | Integration | Doctests | Total | The line of the run |
|---|---:|---:|---:|---:|---|
| `phairplay-cast` | 80 | 6 | 11 | **97** | `test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` / `… 6 passed; …` / `… 11 passed; …` |
| `phairplay-miracast` | 35 | — | 3 | **38** | `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` / `… 3 passed; …` |
| `phairplay-dlna` | 119 | 10 + 11 | 10 | **150** | `test result: ok. 119 passed; …` / `… 10 passed; …` / `… 11 passed; …` / `… 10 passed; …` |
| `phairplay-ffi` | 43 | 11 + 2 + 9 | 14 | **79** | `test result: ok. 43 passed; …` / `… 11 passed; …` / `… 2 passed; …` / `… 9 passed; …` / `… 14 passed; …` |
| `phairplay-web` | 50 | — | 9 | **59** | `test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` / `… 9 passed; …` |
| **M2** | **327** | **49** | **47** | **423** | |

The shared core (`phairplay-core`) carries **113** tests (94 unit + 19
doctests) that M2 leans on — the configuration, the capabilities, the
session registry, the error codes and the privacy masking.

**The order of the build matters for one test.** `cargo test -p phairplay-ffi`
**fails** with

```text
---- a_foreign_caller_round_trips_through_the_c_abi stdout ----
thread 'a_foreign_caller_round_trips_through_the_c_abi' panicked at
  platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs:50:5:
the shared library of the bridge must be built next to the test binary
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

when the shared library was not built before the test. The test binary was
copied to a directory without the library and run there to reproduce the red
state exactly; the same binary passes with `cargo build -p phairplay-ffi`
before the test. The continuous integration does the two steps in that
order (`.github/workflows/ci.yml`, the step "The shared library and the
binding generator of the bridge" before "The tests of the workspace").

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh` → exit 0, *13 crates, 15 internal edges, 0 violations*.
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh` → exit 0, *59 files, 488 internal links, 0 broken, 16 budgets compared, 7 budgets owned alone*.
- The dependency direction is unchanged: **yes** — `cargo tree --offline --workspace --depth 1`: `phairplay-web` depends on `phairplay-core` and `phairplay-ffi`, `phairplay-ffi` depends on the five engines (ADR-0006 gives the bridge the composition root), and **no engine depends on another engine**.
- The tests are green: **yes** — **423 passed, 0 failed, 0 ignored** for the crates of M2 (the table above); the whole workspace: **1224 passed, 0 failed**.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **yes** — `cargo clippy --offline --workspace --all-targets -- -D warnings` → exit 0, **no finding**, 7 min 24 s.
- The format check is green (`cargo fmt --all --check`): **yes** — exit 0, no output.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** — exit 0, 0 broken links.
- The length of the source files is within the limit: **yes** — `bash scripts/check-file-size.sh` → exit 0, *365 files measured, 0 above the hard limit*.
- The licence headers are present: **yes** — `bash scripts/check-license-headers.sh` → exit 0, *13 manifests, 251 source files, 10 derived files, 0 violations*.
- The dependency policy is satisfied: **yes** — `bash scripts/audit-licenses.sh` → exit 0, `bans ok, licenses ok, sources ok`, with the **pinned** cargo-deny **0.20.2**.
- The security audit: **green with the one named exception** — `cargo audit --ignore RUSTSEC-2023-0071` → exit 0, *1 allowed warning* (`RUSTSEC-2026-0374`, `basic-toml` unmaintained). Without the exception → exit 1 with **one vulnerability**, `RUSTSEC-2023-0071` (`rsa 0.9.10`, "Marvin attack"), which belongs to the **legacy AirPlay pairing** of M1.
- The documentation of the workspace builds: **yes, with one warning** — `cargo doc --offline --workspace --no-deps` → exit 0, **1 warning**: `unresolved link to [ProtocolEngine]` in `engines/smartview/src/lib.rs:56`. The warning is in the crate of **M5**; the crates of M2 build their documentation without a warning.
- The versioning or the migration is checked: **not applicable** — the project is at version 0.1.0 and has no migration to prove.

### The build contract of the two crates of this milestone

| Rule | Result | Evidence |
|---|---|---|
| The bridge is the only way into the core | **pass** | `scripts/check-crate-boundaries.sh` → `platform → engines` is legal, `apps/web → engines` is not; the shells import only `uniffi.phairplay_ffi.*` / `phairplay_ffiFFI` |
| Every read has a deadline | **pass** | the Cast engine drops a silent peer after the idle timeout — `cast_engine_hardening::a_peer_that_stays_silent_is_dropped_after_the_idle_timeout`; the DLNA reader refuses an over-long request line — `hardening::an_over_long_request_line_is_refused`; the Miracast reader refuses an oversized message — `rtsp::tests::an_oversized_message_is_rejected` |
| Every input is length-limited | **pass** | 11 negative tests over a real socket in `engines/dlna/tests/hardening.rs`; 6 in `engines/cast/tests/cast_engine_hardening.rs`; the Miracast header and message limits in `rtsp::tests` |
| The engines never decode and never query the platform | **pass** | the engines hand **encoded** frames to the sinks — `phairplay-ffi` → `sinks::tests::the_format_is_forwarded_to_the_decoder`, `without_a_decoder_nothing_is_counted` |

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | not measured | **NOT RUN** |
| Sustained frame rate | `throughput.minimum_fps` | not measured | **NOT RUN** |
| Resident memory | `resources.max_memory_mb` | not measured | **NOT RUN** |
| CPU usage | `resources.max_cpu_percent` | not measured | **NOT RUN** |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | not measured | **NOT RUN** |

The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml). The
values are not repeated in this document.

**Why:** the five budgets are properties of a **running** receiver with a
**real** sender on a **real** network. The sandbox has no television, no
Chromecast, no Windows computer and no VLC; a number measured on a build
server would be a number without meaning.

**How they will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) §2.2,
§2.3, §2.4 and §3) names the sender, the step and the command for every
one of them:

| Budget | The measurement | The place in the plan |
|---|---|---|
| `latency.first_frame_ms` | the stop watch between the click of the sender and the picture | §2.2 step 2 (Miracast), §2.3 step 2 (Cast) |
| `throughput.minimum_fps` | the counters of the dashboard over the measurement window | §3 |
| `resources.max_memory_mb` | `ps -o rss= -C phairplay-web` during the session | §3 |
| `resources.max_cpu_percent` | `ps -o pcpu= -C phairplay-web` during the session | §3 |
| `latency.audio_video_sync_ms` | the difference of the presentation timestamps over one minute | §3 |

**What was measured anyway — the control plane on the loopback.** The
receiver of this round answered the API of the dashboard from the loopback
of the sandbox while five protocols were advertising. That is **not** a
budget of the product; it is the evidence that the API of §1 answers at
all. The budgets of the configuration have **no key** for this path, so
there is nothing to compare it with.

## 4. Usability

- The number of steps of the task: **two** for the switch of a protocol — the arrow keys move the focus inside the group of the cards, *OK* (or *Enter*, or *Space*) switches the receiver that holds the focus. Measured: [`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §3 (the table of the keys) and §7 (`Enter switches the receiver (aria-pressed false → true, POST /api/protocols/cast/enable)`).
- The discoverability notes: one card per protocol with the state, the port, the number of peers and the live counters — `state::tests::the_state_covers_every_protocol`, `dto::tests::the_status_is_serialised_with_a_readable_state`; the running receiver answers `GET /api/status` with exactly those fields.
- The distinction of the states ("disabled", "advertising", "streaming", "error"): **proved.** A protocol that is switched off reads *disabled*, a protocol that failed to start reads *failed* **with the reason** — the running receiver: `{"protocol":"smartview","state":"failed","last_error":"the SSDP responder could not be started: the SSDP socket could not be opened: Address already in use (os error 98)"}`. The state is carried by a word and a shape, not by a colour alone — [`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §4.
- The clarity of an error message: every failure carries a typed error and a stable numeric code — `phairplay-core` → `error::tests::error_codes_are_stable`, `describe_includes_the_cause_chain`; `phairplay-ffi` → `error::tests::a_core_error_keeps_its_code_and_its_reason`; the running receiver: the reason of the failed protocol is a **sentence**, not a number.
- The mobile / native behaviour: the panel is a web page; the native shells are M3. **NOT RUN** on a phone — no device in the sandbox.

## 5. Accessibility

| Question | Result | Evidence |
|---|---|---|
| The keyboard operation | **pass (the checks)** | `node apps/web/tools/check-keyboard.mjs` → **35 checks passed, 0 failed** — the group of the cards is **one** tab stop, every arrow key moves the focus to the card the eye expects, the arrow key at the edge leaves the group, *Enter* and *Space* switch the receiver, *Escape* closes the banner; the raw output: [`docs/audit/tv-web-dashboard-keyboard.log`](../audit/tv-web-dashboard-keyboard.log) |
| The screen reader (VoiceOver / TalkBack) | **NOT RUN** | the panel carries the ARIA attributes and the accessible names (the keyboard check asserts `every card carries a name for the screen reader`), but **no screen reader was run** — that needs a device |
| The scaling, the high contrast, the reduced motion | **pass (the checks)** | `node apps/web/tools/check-contrast.mjs` → every measured pair reaches its threshold; `node apps/web/tools/check-layout.mjs` → **31 checks passed, 0 failed**, `the ring stays, the movement stops (solid 3 px, transform none)`; the focus ring is **3 px** and reaches **9.96:1** (dark) / **8.46:1** (light) against the card — [`docs/audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §4 and §5 |
| On a television with a real remote control | **NOT RUN** | no television and no remote control in the sandbox — [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) |

## 6. Security and privacy

| Measure | Result | Evidence |
|---|---|---|
| The dashboard is a local control panel and never accepts a foreign address | **pass** | **measured**: `GET http://127.0.0.1:7888/api/status` → `200`; `GET http://172.25.29.225:7888/api/status` → **connection refused**; `server::tests::the_bound_socket_is_on_the_loopback_interface` |
| A foreign bind address is announced on stderr before anything starts | **pass** | **measured**: the warning of `--bind 0.0.0.0` names the address and precedes the configuration |
| The SOAP body is parsed with the XML rules of the policy | **pass** | `hardening::a_doctype_in_the_soap_body_is_refused`, `an_external_entity_in_the_soap_body_is_refused`; `xml::tests::an_external_entity_reference_is_rejected` |
| The GENA callback is checked against the SSRF guard | **pass** | `hardening::a_gena_callback_to_the_metadata_service_is_refused`, `a_gena_callback_to_the_loopback_address_is_refused`, `a_gena_callback_to_the_ipv6_loopback_is_refused`, `a_gena_callback_to_a_host_name_is_refused`; the host name is **never resolved** (`gena::tests::a_host_name_is_refused_because_it_is_never_resolved`) |
| The payload limits of DLNA are enforced before the allocation | **pass** | `hardening::a_body_that_is_too_large_is_refused`, `too_many_headers_are_refused`, `an_over_long_header_line_is_refused`, `a_chunked_body_is_refused` |
| The CastV2 frame with an impossible length is refused | **pass** | `frame::tests::an_oversized_length_is_rejected_without_allocating`; `cast_engine_hardening::a_message_that_announces_more_than_the_limit_ends_the_session` |
| A client that does not speak TLS is dropped | **pass** | `cast_engine_hardening::a_client_that_does_not_speak_tls_is_dropped` |
| A silent peer does not hold a slot | **pass** | `cast_engine_hardening::a_peer_that_stays_silent_is_dropped_after_the_idle_timeout` |
| No panic crosses the bridge | **pass** | `phairplay-ffi` → `receiver::tests::an_invalid_configuration_is_rejected`, `a_protocol_that_does_not_exist_is_refused_as_an_invalid_input`; the foreign caller of the C ABI loads the library and calls it — `foreign_abi_roundtrip::a_foreign_caller_round_trips_through_the_c_abi` |
| The log does not name a peer | **pass** | `phairplay-core` → `privacy::tests::a_name_is_not_readable_in_its_pseudonym`, `the_host_part_of_an_ipv4_address_is_masked`, `the_host_part_of_an_ipv6_address_is_masked` |
| No telemetry, no analytics, no crash upload | **pass** | `grep -rniE "telemetry\|analytics\|sentry\|crashlytics\|mixpanel\|amplitude\|segment\.io\|google-analytics" core engines formats platform apps` → only the two documentation comments that **promise** the absence and the word `status` in the Cast crate |
| The outbound policy | **pass** | `config/security-policy.toml` → `allow_outbound_connections_except_local_subnet = false` |
| The dependency policy | **pass** | `cargo-deny` 0.20.2 → `bans ok, licenses ok, sources ok` |
| A dependency has no known advisory | **green with the named exception** | `cargo audit --ignore RUSTSEC-2023-0071` → exit 0 |
| The generated bindings are not hand-edited | **NOT MET** | a regeneration produces a diff of two files and a **changed API checksum** — §8, L-4 |

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| Windows 11 "Cast to Device" (Miracast) | Windows 11 | **NOT RUN** | no Windows computer in the sandbox — the plan §2.2 |
| Windows 11 "Cast to Device" (DLNA) | Windows 11 | **NOT RUN** | the plan §2.4 |
| Google Chrome / Chromecast (Cast) | any | **NOT RUN** | no Chromecast and no second host — the plan §2.3 |
| The Cast SDK on Android or iOS | — | **NOT RUN** | no Android or iOS device — the plan §2.3 |
| VLC 3.0 (DLNA) | 3.0 | **NOT RUN** | no VLC and no UPnP control point — the plan §2.4 |
| A mobile UPnP controller | Android, iOS | **NOT RUN** | no device — the plan §2.4 |

What **is** proved is the side of the protocols that does not need a peer:
the parsers, the answers over real sockets (the DLNA control plane over a
real HTTP socket, the Cast control plane over a real TLS socket), the
limits, the state machine and the hardening. The interoperability with a
real sender is a device test and is recorded as **NOT RUN** here — not as
passed.

The **deviations from the expectation of the sender** that are known
without a sender are listed in §8: the multi-peer rule of Cast (L-1), the
missing GENA delivery (L-3) and the stale bindings (L-4).

## 8. Known limitations

| # | The limitation | Severity | Where | What the owner has to decide |
|---|---|---|---|---|
| **L-1** | **The one-peer rule has two owners.** `FR-040` says the rule lives in **one** place — `session.multi_peer_protocols = ["cast"]` in [`config/app-defaults.toml`](../../config/app-defaults.toml). No Rust code reads that list (`grep -rn "multi_peer" --include=*.rs .` → no hit); the limit is the constant `DEFAULT_MAX_SESSIONS = 8` in `engines/cast/src/engine.rs:57`. The same requirement asks for a test with two concurrent senders — **there is none**. | **S2** | `engines/cast/src/engine.rs:57`, `config/app-defaults.toml` | the configuration is the authority, and a test with two senders |
| **L-2** | **The core drops the playback state.** `SessionEvent::PlaybackStateChanged` is folded with `=> {}` in `core/phairplay-core/src/session.rs:186`; the Cast engine emits it (`engines/cast/src/peer.rs:224`), the event channel carries it to the panel — but the **snapshot of the session** of the core carries no state, no position and no metadata. `FR-042` asks for the state to be visible in the shell and in the dashboard. | **S2** | `core/phairplay-core/src/session.rs:186` | fold the events into `SessionSnapshot`, with a test of the fold |
| **L-3** | **The GENA delivery has no test.** `FR-046` asks for a test of the event delivery; `gena::notify_subscribers` is never called by a test. The subscription and the SSRF guard are proven, the **delivery** is not. | **S2** | `engines/dlna/src/gena.rs:330` | a test with a subscriber that receives the event |
| **L-4** | **The generated bindings are stale and were hand-edited.** A regeneration (`bash scripts/generate-bindings.sh --offline`) produces a diff of two files: the API checksum of the constructor `Receiver::new` is **31533** in the committed bindings and **5570** in the regenerated ones. A native shell that loads the committed binding against a library built from this source refuses to start with *"UniFFI API checksum mismatch"*. The binding was edited by hand in the commit `86c1b81` (the field `audioQueueBytes` was added to the data class without a regeneration). | **S2** | `apps/android/app/src/main/java/uniffi/phairplay_ffi/phairplay_ffi.kt`, `apps/macos/Sources/PhairPlayFFI/phairplay_ffi.swift` | regenerate the bindings and commit them; put the regeneration into the gate |
| **L-5** | **The event channel has no test.** The WebSocket endpoint of the panel (`apps/web/src/events.rs`) has **no** test; only the conversion of an event is tested (`state::tests::a_status_event_becomes_a_dashboard_event`). `FR-083` asks for the events of the core to appear in the dashboard without a reload. | **S2** | `apps/web/src/events.rs` | a test of the endpoint |
| **L-6** | **The advertised codecs are not derived from the capabilities.** `MediaCapabilities::supports_cast_baseline` and `supports_miracast_baseline` exist in the core and are tested, but **no engine calls them**; the Miracast engine hard-codes `wfd_audio_codecs: LPCM 00000003 00` (`engines/miracast/src/rtsp.rs:243`). `FR-007`, `FR-034` and `FR-041` therefore hold only in the core, not in the advertisement. | **S2** | `engines/miracast/src/rtsp.rs:243`, `core/phairplay-core/src/capabilities.rs` | the engines ask the core before they advertise |
| **L-7** | **The SSDP port was shared — and was fixed after the measurement.** At `2d0c4b4` the DLNA renderer and the Smart View receiver want the same UDP port 1900; the running receiver reports `Address already in use (os error 98)` for the second one and only one of the two can run. The receiver reports it with the reason and the other protocols keep running — that is the **correct** behaviour. The commit `bbc39c8` ("fix: the receiver announces two services over one multicast group", 23:22) moved the listener into the core; **the same run at that commit reports `5 of 5 protocols are running`**. The finding is therefore **closed on the head of the branch** and open only at the revision this record measures. | **S3** (closed at `bbc39c8`) | `core/phairplay-core/src/ssdp/`, ADR-0007 | nothing — the fix is committed; the record of M2 has to be repeated on the new head |
| **L-8** | **The generator of the bindings ignores `CARGO_TARGET_DIR`.** `scripts/generate-bindings.sh` looks for the shared library in `$REPO_ROOT/target/debug` and fails when the build went somewhere else (`the shared library of the bridge was not found in …`). The continuous integration uses the default directory and does not see it. | **S3** | `scripts/generate-bindings.sh:60` | the script honours the target directory of cargo |
| **L-9** | **No hardware test has been run.** Every interoperability row and every performance row of this record is **NOT RUN**. The project does not claim otherwise. | **S1** | §3, §7, §9 | the device test plan, on a network with a real sender |
| **L-10** | **The media path is the platform's job.** The engines deliver *encoded* frames to a sink; the decoding belongs to the native shell (M3). A receiver that cannot decode a codec does not advertise it — but the advertisement does not follow the capabilities yet (L-6). | **S2** | `ARCHITECTURE.md`, the shells of M3 | the shells of M3 and the wiring of L-6 |

The exclusions that apply to this milestone: `EXC-003` (HDCP — a session
that cannot be protected is refused cleanly), `EXC-004` (no Widevine, no
PlayReady), `EXC-008` (Google Cast is the documented exception to the
one-peer rule — the rule itself is not implemented in one place, L-1),
`EXC-009` (no transcode), `EXC-010` (no application identifier of the
project's own).

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| The time to the first picture | a budget of a **running** receiver with a real sender | the device test plan §2.2 / §2.3 |
| The sustained frame rate | same | the plan §3, the counters of the dashboard |
| The resident memory | same | the plan §3 — `ps -o rss= -C phairplay-web` |
| The CPU usage | same | the plan §3 — `ps -o pcpu= -C phairplay-web` |
| The synchronisation of the audio and the video | a recording of a minute of a real session | the plan §3 |
| The Miracast handshake of a Windows sender | no Windows computer in the sandbox | the plan §2.2 |
| The Cast control plane of a Chromecast | no Chromecast and no second host | the plan §2.3 |
| Two Cast senders at the same time | two senders | the plan §2.3, step 4 (the unit test does not exist, L-1) |
| The DLNA renderer with VLC | no UPnP control point | the plan §2.4 |
| The GENA event on a real subscription | a control point | the plan §2.4, step 4 (the unit test does not exist, L-3) |
| The panel on a television with a real remote control | no television and no remote control | the plan of the television shell; the browser checks are green |
| The reading of the panel in a bright room | a television | the same |
| A screen reader | no device and no screen reader | the record of M3 |
| The Gradle and the Xcode build of the shells | M2 delivers no shell; the sandbox has no Android SDK licence and no macOS SDK | the record of M3 |
| The composition root of the bridge against the **committed** bindings | the committed bindings are stale (L-4) | the regeneration of the bindings |

## 10. Decision

**ACCEPTED with a documented limitation — the code is accepted, the product
is not.**

What is measured: the workspace compiles, the lint is clean, the format is
clean, the documentation links resolve, the licence and the security gates
are green, **423 tests of the crates of M2 pass** (1224 of the whole
workspace), the three protocols answer a **real** socket — the DLNA control
plane over HTTP, the Cast control plane over TLS — and reject the attacks
that the requirements name: the `DOCTYPE`, the external entity, the GENA
callback at `169.254.169.254`, the callback at the loopback address, the
callback that names a host, the oversized body, the header flood, the
chunked body, the oversized CastV2 frame, the client that does not speak
TLS, the silent peer. The dashboard was **run**: it binds on the loopback
only, a foreign address is refused, the switch of a protocol changes the
engine and is visible in the next status, and the warning for a foreign
bind address is printed before anything starts.

The reservations are named, not hidden:

1. **The device tests are outstanding.** This record proves the software, not
   the product on a television. The milestone may **not** be announced on the
   strength of this record; it may only be announced together with the
   device evidence of [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md).
2. **The performance budgets are unmeasured.** They are budgets; a budget is
   a target that a measurement has to prove.
3. **The committed bindings are stale** (L-4). The shells of M3 are built
   against them; a shell that loads them against a library built from this
   source refuses to start with *"UniFFI API checksum mismatch"*. The
   regeneration is one command — `bash scripts/generate-bindings.sh` — and
   it has to happen **before** the shells are built.
4. **Three requirements of this milestone have no test at all**: the
   multi-peer rule of Cast (L-1), the delivery of a GENA event (L-3) and
   the event channel of the panel (L-5). Two of the three are named by the
   requirement itself as a verification.
5. **The core drops the playback state and the metadata** (L-2): the events
   reach the panel, the **session** of the core does not carry them.
6. **The advertised codecs do not follow the capabilities** (L-6).

The decision was taken by the Rust QA Engineer on 2026-10-24 against the
commit `2d0c4b4` of the branch `feat/other-devices`. **The head of the
branch has moved to `bbc39c8` since**; the record has to be repeated on that
revision before it is used as the proof of the branch. The owner of the
project has to prioritise L-1 … L-6 before the milestone is announced.

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.
