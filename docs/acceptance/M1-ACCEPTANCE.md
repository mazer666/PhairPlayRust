# Acceptance — M1 AirPlay 2 receiver

**Requirement IDs:** FR-001 … FR-031, FR-052 … FR-069, FR-086 … FR-095, NFR-PERF-001 … NFR-PERF-005, NFR-RES-001, NFR-RES-002, NFR-SEC-001 … NFR-SEC-006, NFR-PRIV-001 … NFR-PRIV-004, NFR-INT-001 … NFR-INT-004, NFR-REL-001 … NFR-REL-003, NFR-DOC-001 … NFR-DOC-003, EXC-001, EXC-002, EXC-007, EXC-008
**Milestone:** M1 — AirPlay 2 receiver (`formats/{plist,h264,aac,alac}`, `engines/discovery`, `engines/airplay`)
**Build/commit:** branch `feat/other-devices` at `2d0c4b4` ("fix(ci): the licence gate has to pass on a clean checkout")
**Tester/agent roles:** Rust QA Engineer (this record)
**Date:** 2026-10-24
**The head of the branch moved during the round:** the commit `bbc39c8` ("fix: the receiver announces two services over one multicast group", 23:22) landed **after** the measurement of this record. Every number of this document belongs to `2d0c4b4`; nothing of M1 is changed by that commit — it touches the shared SSDP listener of the core.
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, loopback plus the routable address of the sandbox. **No iPhone, no iPad, no Mac, no Apple TV, no television and no second host were available** — see §3, §7 and §9.

**How the numbers were taken.** Every command of this record was run in a
**git worktree of the commit** `2d0c4b4`, not in the working tree of the
repository: other workstreams were editing the tree while this round was
running, and a number has to belong to a commit. The worktree was

```text
git worktree add /workspace/scratch/qa12-m1m2 2d0c4b4
cd /workspace/scratch/qa12-m1m2
CARGO_TARGET_DIR=/workspace/target-qa12 cargo test --offline --workspace
```

The raw output of every command is in
[`docs/audit/acceptance-m1-m2.md`](../audit/acceptance-m1-m2.md). **The test
numbers of this document live here**; the other documents of the round link
to this table instead of repeating it.

**The sentence that this milestone stands on:** the AirPlay 2 control plane
is implemented, bounded and hardened, and its parsers are proven against
the attacks they name. **No Apple sender was available**, so nothing here
proves that an iPhone finds the receiver, that a session reaches a sink, or
that a picture appears. Those rows say **NOT RUN**.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The property-list codec reads and writes the XML format | round trip | **pass** | `phairplay-plist` → `test result: ok. 84 passed; 0 failed` — `xml::encode::tests::a_dictionary_round_trips`, `xml::encode::tests::round_trips_every_value_type`, `xml::encode::tests::round_trips_an_array_of_every_length`, `xml::decode::tests::decodes_every_leaf_type` |
| The property-list codec reads and writes the binary format `bplist00` | round trip | **pass** | the same run — `binary::encode::tests::round_trips_containers`, `binary::encode::tests::round_trips_data_of_every_length`, `binary::encode::tests::round_trips_an_array_of_every_length`, `binary::decode::tests::decodes_a_hand_written_dictionary`, `binary::decode::tests::agrees_with_the_encoder` |
| The binary decoder refuses a cyclic object graph | a typed error | **pass** | `binary::tests::a_document_with_a_reference_cycle_is_rejected` |
| A declared length never becomes an allocation size | a typed error, nothing allocated | **pass** | `binary::decode::tests::rejects_a_length_field_that_exceeds_the_input`, `rejects_a_data_blob_of_one_gigabyte`, `rejects_an_array_that_is_too_large`, `rejects_an_object_graph_that_expands_too_far`, `rejects_an_absurd_object_count` |
| The XML decoder refuses a document with an internal subset | a typed error | **pass** | `xml::decode::tests::rejects_an_internal_dtd_subset`, `xml::reader::tests::rejects_an_internal_dtd_subset`, `binary::encode::tests::rejects_a_tree_that_is_too_deep`, `xml::decode::tests::enforces_the_depth_limit` |
| The H.264 reader parses a sequence parameter set | width, height, profile | **pass** | `phairplay-h264` → `51 passed; 0 failed` — `sps::tests::parses_a_baseline_sps`, `vui::tests::parses_a_minimal_vui`, `vui::tests::parses_the_timing_information`, `pps::tests::parses_a_baseline_pps` |
| The H.264 reader splits a byte stream into NAL units | the expected units | **pass** | the same run — `nal::tests::splits_a_three_byte_start_code_stream`, `nal::tests::splits_a_four_byte_start_code_stream` |
| The exponential-Golomb reader does not overflow on 31 leading zero bits | an error, no wrap-around | **pass** | the same run — `bits::tests::thirty_one_leading_zeros_do_not_overflow`, `bits::tests::thirty_two_leading_zeros_are_clamped`, `bits::tests::too_many_leading_zeros_are_refused` |
| The AAC crate reads the ADTS header and frames AAC-ELD | sample rate, channels, sample count | **pass** | `phairplay-aac` → `24 passed; 0 failed` — `adts::tests::parses_a_low_complexity_frame`, `counts_the_samples_of_a_frame`, `splits_a_stream_into_frames` |
| The ALAC decoder decodes a frame | the samples of the frame | **pass** | `phairplay-alac` → `61 passed; 0 failed` — `decoder::tests::decodes_an_uncompressed_frame`, `decodes_a_compressed_frame`, `decodes_a_stereo_frame` |
| The ALAC decoder cannot overflow on a narrow sample with a shift | a typed error | **pass** | the same run — `decoder::tests::refuses_a_frame_with_a_broken_shift_flag`, `decoder::tests::refuses_a_shift_that_is_wider_than_the_sample`, `decoder::tests::refuses_a_stereo_shift_that_is_wider_than_the_sample`, `decoder::tests::an_escaped_element_is_not_shifted_with_a_stale_buffer` |
| The discovery registers `_airplay._tcp` **and** `_raop._tcp` together | two service types | **pass** | `phairplay-discovery` → `52 passed; 0 failed` — `mdns::tests::the_airplay_pair_is_registered_together`, `service_types::tests::the_airplay_pair_is_complete`, `mdns::tests::the_raop_service_uses_the_mac_prefixed_name` |
| The instance name carries no control character and respects the DNS-SD cap | a printable name of at most the cap | **pass** | the same run — `mdns::tests::the_instance_name_is_sanitised`, `service_types::tests::the_raop_name_strips_control_characters`, `service_types::tests::the_raop_name_is_capped_at_the_dns_sd_limit`; `phairplay-core` → `identity::tests::control_characters_are_removed`, `identity::tests::names_are_capped_at_the_dns_sd_limit` |
| The RTSP control plane answers `OPTIONS` with the public verbs | `200` + `Public` | **pass** | `phairplay-airplay` → `230 passed; 0 failed` — `rtsp::router::tests::an_options_request_is_answered_with_the_public_verbs` |
| The RTSP control plane answers `ANNOUNCE`, `RECORD`, `GET_PARAMETER`, `SET_PARAMETER` | the status code of the method | **pass** | the same run — `rtsp::router::tests::an_announce_with_a_valid_sdp_is_accepted`, `a_record_without_an_announce_is_rejected`, `the_volume_is_answered_with_the_last_value`, `the_volume_is_stored_when_the_sender_sets_it` |
| The RTSP control plane answers `SETUP`, `TEARDOWN`, `FLUSH`, `PAUSE` | the status code of the method | **implemented, NOT TESTED** | the four verbs are routed in `engines/airplay/src/rtsp/router.rs` (`RtspMethod::Setup => setup(...)`, `Teardown => teardown(...)`, `Flush => ok(protocol)`, `Pause => ok(protocol)`), but **no test of the four verbs exists** — see §8, L-1 |
| A malformed SDP is rejected, never repaired | a typed error | **pass** | `rtsp::router::tests::an_announce_with_a_broken_sdp_is_rejected`, `sdp::tests::an_empty_sdp_is_rejected`, `sdp::tests::an_sdp_without_a_media_section_is_rejected`, `sdp::tests::an_oversized_sdp_is_rejected` |
| The SDP of a session yields the H.264 parameter sets, the ALAC geometry, the key material | the parsed values | **pass** | `sdp::tests::the_h264_parameter_sets_are_extracted`, `an_alac_session_carries_the_frame_geometry`, `the_aes_key_and_iv_are_extracted`, `a_video_and_audio_session_is_parsed` |
| The SPS of the SDP cannot overflow the exponential-Golomb reader | an error | **pass** | `h264::tests::a_run_of_zero_bits_cannot_overflow_the_exp_golomb_reader`, `an_all_zero_sps_is_rejected` |
| The RTP payloads are reassembled into frames | the complete NAL unit at the sink | **pass** | `rtp::tests::a_fragmented_nal_unit_is_reassembled`, `a_complete_nal_unit_is_delivered_unchanged`, `queue::tests::the_frames_reach_the_sink` |
| An overlong fragment sequence is refused | a typed error | **pass** | `rtp::tests::an_oversized_frame_is_rejected`, `queue::tests::a_frame_that_is_larger_than_the_budget_is_refused`, `queue::tests::the_oldest_frame_is_dropped_when_the_budget_is_reached`; `phairplay-h264` → `rtp::tests::rejects_a_fragment_that_is_larger_than_the_limit`, `rtp::tests::rejects_a_reassembled_unit_that_is_larger_than_the_limit` |
| A second AirPlay peer is refused, not queued | the status of the protocol | **pass** | `engine::tests::a_second_sender_is_rejected_with_service_unavailable`, `rtsp::response::tests::the_second_peer_is_answered_with_service_unavailable` |
| The receiver answers with the configured identity | the configured name | **pass** | `phairplay-core` → `identity::tests::identity_uses_the_sanitised_name_for_the_service_instance`; `phairplay-airplay` → `rtsp::http::tests::the_info_answer_is_a_binary_plist` |
| The HomeKit-style pairing completes and the sender is verified | the shared secret | **pass** | `pairing::verify::tests::pair_setup_returns_the_receiver_public_key`, `a_complete_exchange_produces_a_shared_secret`, `the_sender_can_verify_the_receiver_signature`, `a_forged_signature_is_rejected`, `the_second_message_without_the_first_is_rejected` |
| The legacy SRP PIN pairing completes, and a wrong PIN is refused | the session key / the refusal | **pass** | `pairing::legacy::tests::a_client_that_knows_the_pin_completes_the_exchange`, `a_wrong_pin_is_rejected`, `step_two_before_step_one_is_rejected`, `the_session_key_is_forty_bytes` |
| The failed pairing attempts are counted | a rising counter | **pass** | `pairing::store::tests::the_failed_attempt_counter_grows_and_can_be_reset` |
| The pairing material is stored with owner-only permissions | the mode `0600` of the file | **NOT IMPLEMENTED** | the only store is `InMemoryPairingStore` (`engines/airplay/src/pairing/store.rs`); `store_permissions = "0600"` of [`config/security-policy.toml`](../../config/security-policy.toml) is read by **no** Rust code (`grep -rn "store_permissions" --include=*.rs .` → no hit) — §8, L-2 |
| The `fp-setup` handshake answers in version 2 and in version 3 | 142 bytes, then 32 bytes | **pass** | `fairplay::tests::the_first_phase_answers_with_142_bytes`, `the_second_phase_answers_with_32_bytes`, `every_mode_has_its_own_answer`, `the_negotiated_version_is_remembered` |
| The legacy `rsaaeskey` exchange unwraps the AES key | the key | **pass** | `pairing::raop_rsa::tests::a_wrapped_key_is_recovered`, `the_embedded_key_parses`, `a_garbage_blob_is_rejected` |
| The session key that protects the stream is decrypted and the stream is decrypted with it | the decrypted frames | **NOT MET** | `fairplay::tests::the_key_decryption_reports_the_product_boundary` — `FairPlayHandshake::decrypt()` returns `AirPlayError::unsupported` for every input (`engines/airplay/src/fairplay.rs:164`); the key material of the mirroring path is **derived** (`mirror::tests::the_derivation_is_deterministic`, `the_derived_material_has_the_expected_length`) but the media is **not** decrypted (`engines/airplay/src/engine/connection.rs:199` hands the NAL units to the sink unchanged) — §8, L-3 |
| FairPlay DRM content is not decrypted, and the receiver says so | the boundary is named | **pass** | `config/security-policy.toml` → `supports_fairplay_drm = false`; `fairplay::tests::the_key_decryption_reports_the_product_boundary`; `EXC-001` |
| The DMAP metadata of the sender is parsed | title, artist, album | **pass** | `dmap::tests::a_flat_document_yields_the_metadata`, `a_nested_listing_item_yields_the_metadata`, `a_length_that_overruns_the_buffer_is_rejected` |
| The metadata reaches the session of the core | the folded snapshot | **NOT MET (partly)** | the engine emits the event (`engines/airplay/src/rtsp/router.rs:297`), but the **core drops it**: `core/phairplay-core/src/session.rs:186` folds `MetadataChanged` and `ArtworkChanged` with `=> {}` — §8, L-4 |
| The reverse remote builds the DACP request | the path and the token | **pass** (the builder) | `dacp::tests::a_command_has_a_stable_path`, `the_service_name_carries_the_identifier`, `the_request_carries_the_token`, `a_request_without_an_identity_is_rejected` — the command **reaching a sender** is **NOT RUN** (§7) |
| The NTP timing of the sender is read | the offset and the reference | **pass** (the parser) | `timing::tests::a_probe_updates_the_clock_offset`, `the_response_echoes_the_sender_transmit_time_as_the_reference`, `the_ntp_timestamp_is_in_the_ntp_epoch` — the **budget** is **NOT RUN** (§3) |
| A photo is received, and a photo that is too large is refused | JPEG/PNG accepted, the oversize refused | **pass** | `photo::tests::a_jpeg_is_accepted`, `a_png_is_accepted`, `an_oversized_payload_is_rejected`, `random_bytes_are_rejected`, `a_mismatched_content_type_is_rejected`; `rtsp::http::tests::a_photo_is_accepted_and_published`, `a_photo_with_random_bytes_is_rejected`, `a_photo_delete_clears_the_published_photo` |
| A session without a video section is recognised | an audio-only session | **pass** | `sdp::tests::a_session_with_only_audio_is_recognised` |
| The session is torn down and the peer is released | the state of the engine | **implemented, NOT TESTED end to end** | the teardown is routed (`rtsp/router.rs:233`) and the helper is tested (`rtsp::http::tests::a_teardown_without_a_body_names_no_stream`); there is **no test that a live session is torn down and that the engine returns to `Advertising`** — §8, L-1 |
| The receiver writes no log with the name or the address of the peer | a masked line | **pass** | `phairplay-core` → `privacy::tests::a_name_is_not_readable_in_its_pseudonym`, `the_host_part_of_an_ipv4_address_is_masked`, `the_loopback_address_is_masked_as_well`, `the_metadata_address_is_masked_as_well`, `a_url_without_a_port_is_masked_too` |
| The advertisement comes back after a network change | the services are registered again | **NOT IMPLEMENTED** | the default `discovery.re_advertise_on_network_change = true` is in [`config/app-defaults.toml`](../../config/app-defaults.toml), but **no Rust code reads it** (`grep -rn "re_advertise_on_network_change" --include=*.rs .` → no hit) and the discovery crate has no re-advertisement path — §8, L-8 |
| The receiver checks that its advertisement is still alive | the service is registered again | **NOT IMPLEMENTED** | the key `discovery.advertisement_check_interval_seconds` exists in the configuration and is read by **no** code (`grep -rn "advertisement_check_interval_seconds" --include=*.rs .` → no hit) — §8, L-8 |
| A session without a video section reaches the audio sink | the audio packets at the sink | **implemented, NOT TESTED** | the audio stream is routed by its stream type (`engines/airplay/src/engine/ports.rs:99` → `pump_audio`) and delivered through a bounded mailbox; **no test exercises it** — and the audio frame carries `pts_us: 0` (`ports.rs:160`), so the synchronisation of the audio and the video cannot be computed from the delivered frames — §8, L-11 |

### The test numbers of M1 — the single source

The command was `cargo test --offline --workspace` in the worktree of
`2d0c4b4`; the exit code was 0 and the whole workspace carried **1224
passed, 0 failed, 0 ignored**. The part that belongs to M1:

| Crate | Unit | Integration | Doctests | Total | The line of the run |
|---|---:|---:|---:|---:|---|
| `phairplay-plist` | 84 | — | 7 | **91** | `test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `phairplay-h264` | 51 | — | 9 | **60** | `test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `phairplay-aac` | 24 | — | 6 | **30** | `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `phairplay-alac` | 61 | — | 5 | **66** | `test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `phairplay-discovery` | 52 | — | 8 | **60** | `test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `phairplay-airplay` | 230 | 9 | 30 | **269** | `test result: ok. 230 passed; …` / `test result: ok. 9 passed; …` / `test result: ok. 30 passed; …` |
| **M1** | **502** | **9** | **65** | **576** | |

The integration test of the engine is `engines/airplay/tests/xml_plist_hardening.rs`
— the decoder of the property list of the engine: `an_external_entity_is_never_resolved`,
`an_entity_that_points_at_a_network_service_is_never_resolved`,
`a_doctype_with_an_internal_subset_is_refused`,
`a_document_that_nests_too_deeply_is_refused`,
`a_document_that_is_too_large_is_refused_before_it_is_parsed`,
`a_truncated_document_is_refused`, `an_unknown_element_is_refused`,
`a_dictionary_without_a_key_is_refused`.

The shared core (`phairplay-core`) carries **113** tests (94 unit + 19
doctests) that M1 leans on — the configuration, the identity, the session
registry, the error codes and the privacy masking. Their number belongs to
the record of the core and is quoted here only as a dependency.

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh` → exit 0, *13 crates, 15 internal edges, 0 violations*.
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh` → exit 0, *59 files, 488 internal links, 0 broken, 16 budgets compared, 7 budgets owned alone*.
- The dependency direction is unchanged: **yes** — `cargo tree --offline --workspace --depth 1` shows every engine of M1 (`phairplay-airplay`, `phairplay-discovery`) depending on `phairplay-core` and on **no other engine**; the only crate that depends on the engines is the bridge `phairplay-ffi` (ADR-0006).
- The tests are green: **yes** — **576 passed, 0 failed, 0 ignored** for the crates of M1 (the table above); the whole workspace: **1224 passed, 0 failed**.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **yes** — `cargo clippy --offline --workspace --all-targets -- -D warnings` → exit 0, **no finding**, 7 min 24 s.
- The format check is green (`cargo fmt --all --check`): **yes** — `cargo fmt --all -- --check` → exit 0, no output.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** — exit 0, 0 broken links.
- The length of the source files is within the limit: **yes** — `bash scripts/check-file-size.sh` → exit 0, *365 files measured, 0 above the hard limit*.
- The licence headers are present: **yes** — `bash scripts/check-license-headers.sh` → exit 0, *13 manifests, 251 source files, 10 derived files, 0 violations*.
- The dependency policy is satisfied: **yes** — `bash scripts/audit-licenses.sh` → exit 0, `bans ok, licenses ok, sources ok`, with the **pinned** cargo-deny **0.20.2**.
- The security audit: **green with the one named exception** — `cargo audit --ignore RUSTSEC-2023-0071` → exit 0, *1 allowed warning* (`RUSTSEC-2026-0374`, `basic-toml` unmaintained). Without the exception `cargo audit` → exit 1 with **one vulnerability**: `RUSTSEC-2023-0071` ("Marvin attack", `rsa 0.9.10`, severity 5.9). The crate `rsa` is used by the **legacy AirPlay pairing** of this milestone; the exception is written down in the workflow `.github/workflows/ci.yml` and in [`SECURITY.md`](../../SECURITY.md).
- The versioning or the migration is checked: **not applicable** — the project is at version 0.1.0 and has no migration to prove.

### The format crates are pure — and who uses them

| Question | Answer | Evidence |
|---|---|---|
| Does a format crate do I/O? | **no** | `grep -rn "std::net\|std::fs" formats/` → no hit; the four manifests of `formats/*` carry **only** `thiserror` as a dependency |
| Is the ALAC port a native Rust port? | **yes** | `formats/alac/src/lib.rs` names the derived files (`ALACDecoder.cpp`, `ALACBitUtilities.c`, `ag_dec.c`, `dp_dec.c`, `matrix_dec.c`), the copyright and the Apache-2.0 licence; every file of the crate carries the attribution header; `THIRD_PARTY.md` lists the reference |
| Does the AirPlay engine use the crates of M1.0? | **no — it carries its own copies** | `grep -rn "phairplay-h264\|phairplay-aac\|phairplay-alac" --include=*.rs .` → **no hit outside `formats/`**; the engine has its own `engines/airplay/src/h264.rs` (287 lines) and its own property-list codec (`engines/airplay/src/plist/`, 1 533 lines). Only `phairplay-plist` is used — by the **Cast** engine. → §8, L-5 |

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

**Why:** every one of these five budgets is a property of a **running**
receiver on a **real** device with a **real** sender. The sandbox has
neither a television nor an Apple sender; a number measured on a build
server would be a number without meaning.

**How they will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) §2.1
and §3) names the sender, the step and the command for every one of them:

| Budget | The measurement | The place in the plan |
|---|---|---|
| `latency.first_frame_ms` | the stop watch of step 2 of the AirPlay script | §2.1, step 2 |
| `throughput.minimum_fps` | the counters of the dashboard over the measurement window | §2.1, §3 |
| `resources.max_memory_mb` | `ps -o rss= -C phairplay-web` during the session | §2.1, step 11; §3 |
| `resources.max_cpu_percent` | `ps -o pcpu= -C phairplay-web` during the session | §2.1, step 11; §3 |
| `latency.audio_video_sync_ms` | the difference of the presentation timestamps over one minute | §2.1, §3 |

## 4. Usability

- The number of steps of the task: **not measured on a device** — M1 delivers **no user-visible surface**. The shells (Android, macOS) are M3, the dashboard is M2.
- The discoverability notes: the name that the sender's device picker shows is the configured name of the receiver, sanitised and capped — `phairplay-core` → `identity::tests::identity_uses_the_sanitised_name_for_the_service_instance`, `names_are_capped_at_the_dns_sd_limit`; `phairplay-discovery` → `tests::the_instance_name_is_sanitised`.
- The distinction of the states ("disabled", "advertising", "streaming", "error"): **defined and readable** — the state machine of the core is `Idle → Starting → Advertising → Streaming → Stopping → Stopped/Failed` (`core/phairplay-core/src/protocol.rs`); the engine exposes it through `ProtocolEngine::status` — `phairplay-airplay` → `engine::tests::the_status_is_readable_from_another_thread`, `a_new_engine_is_idle`, `a_started_engine_advertises_and_stops_again`.
- The clarity of an error message: every failure of the engine carries a typed error and a stable numeric code — `phairplay-airplay` → `error::tests::a_limit_error_maps_onto_the_core_limit_variant`, `a_protocol_violation_maps_onto_the_core_protocol_variant`, `the_core_error_kind_of_a_limit_is_resource_exhausted`; `phairplay-core` → `error::tests::error_codes_are_stable`, `describe_includes_the_cause_chain`.
- The mobile / native behaviour: **NOT RUN** — no Apple device and no native shell in the scope of M1; the device test plan §2.1.

## 5. Accessibility

- The keyboard operation: **not applicable to M1** — the milestone delivers no panel. The dashboard (the first panel of the product) is **M2** and is measured in the record of M2.
- The screen reader (VoiceOver / TalkBack): **NOT RUN** — needs a device; the native shells are M3.
- The scaling, the high contrast, the reduced motion: **not applicable to M1** — the panel is M2.

## 6. Security and privacy

| Measure | Result | Evidence |
|---|---|---|
| Every network input is length-limited | **pass** | the limits live in `AppConfig::limits`; `phairplay-core` → `config::engine_limits::tests::the_limits_of_every_protocol_come_from_the_configuration`, `the_default_limits_are_the_documented_ones`; `phairplay-airplay` → `limits::tests::the_limits_come_from_the_configuration`, `the_message_limit_comes_from_the_protocol_configuration` |
| Every blocking read carries a deadline | **pass** | `rtsp::reader::tests::a_stalled_peer_hits_the_deadline`; the deadline is `latency.idle_timeout_ms` — `phairplay-core` → `config::timeouts::tests::the_timeouts_are_the_documented_ones` |
| Every queue is bounded by an item count **and** by a byte budget | **pass** | `phairplay-core` → `queue::tests::the_byte_budget_holds_under_the_ninety_frames_of_eight_megabytes_attack`, `the_oldest_entry_is_dropped_when_the_byte_budget_is_reached`, `an_entry_that_is_larger_than_the_budget_is_refused`; `phairplay-airplay` → `queue::tests::a_frame_that_is_larger_than_the_budget_is_refused` |
| The property-list decoder refuses a cyclic object graph | **pass** | `phairplay-plist` → `binary::decode::tests::rejects_a_cycle`; `phairplay-airplay` → `plist::binary::tests::a_document_with_a_reference_cycle_is_rejected` |
| The SPS parser cannot overflow on 31 leading zero bits | **pass** | `phairplay-h264` → `exp_golomb::tests::thirty_one_leading_zeros_do_not_overflow`; `phairplay-airplay` → `h264::tests::a_run_of_zero_bits_cannot_overflow_the_exp_golomb_reader` |
| The ALAC decoder cannot overflow on a narrow sample with a shift | **pass** | `phairplay-alac` → `elements::tests::refuses_a_shift_that_is_wider_than_the_sample`, `refuses_a_stereo_shift_that_is_wider_than_the_sample` |
| The XML decoder of the engine refuses an external entity | **pass** | `engines/airplay/tests/xml_plist_hardening.rs` → `an_external_entity_is_never_resolved`, `an_entity_that_points_at_a_network_service_is_never_resolved` |
| The header flood and the oversized message are refused | **pass** | `rtsp::reader::tests::too_many_headers_are_refused`, `an_overlong_header_line_is_rejected`, `an_overlong_request_line_is_rejected`, `a_declared_body_longer_than_the_limit_is_rejected`, `rtsp::request::tests::an_oversized_message_is_rejected` |
| The peer is never named in the log | **pass** | `phairplay-core` → `privacy::tests::a_name_is_not_readable_in_its_pseudonym`, `the_host_part_of_an_ipv4_address_is_masked`, `the_host_part_of_an_ipv6_address_is_masked` |
| The pairing material is protected on the disk | **NOT IMPLEMENTED** | the only store is in memory; `store_permissions = "0600"` is read by no code — §8, L-2 |
| No unsafe code | **pass** | the workspace denies `unsafe_code`; `phairplay-ffi` carries the only local exemption, for the generated UniFFI scaffolding |
| No panicking path in production code | **pass** | the workspace denies `unwrap_used` and `expect_used`; the lint of the workspace is green (§2) |
| No telemetry, no analytics, no crash upload | **pass** | `grep -rniE "telemetry\|analytics\|sentry\|crashlytics\|mixpanel\|amplitude\|segment\.io\|google-analytics" core engines formats platform apps` → the only hits are the two documentation comments that **promise** the absence (`core/phairplay-core/src/lib.rs:6`, `apps/web/src/lib.rs:11`) and the word `status` in the Cast crate; the hits under `apps/android/app/build` and `apps/macos/.build` are build artefacts of the Gradle and Swift caches, not sources |
| No account, no sign-in | **pass** | `grep -rniE "sign[ -]?in\|sign[ -]?up\|log[ -]?in\|account" core engines formats platform apps` → only the documentation comment and the words `signing_key` / `Signature` of the pairing |
| The outbound policy of the security policy | **pass** | `config/security-policy.toml` → `allow_outbound_connections_except_local_subnet = false` |
| A dependency has no known advisory | **green with the named exception** | `cargo audit --ignore RUSTSEC-2023-0071` → exit 0; without the exception → 1 finding, `RUSTSEC-2023-0071` (`rsa 0.9.10`), the legacy pairing of this milestone |

The threat model and the guarantees are written down in
[`SECURITY.md`](../../SECURITY.md). The data that the receiver sees, and the
data it does not keep, is written down in [`PRIVACY.md`](../../PRIVACY.md).

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| macOS — screen mirroring | macOS 14 | **NOT RUN** | no Mac in the sandbox — [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) §2.1 |
| iOS / iPadOS — screen mirroring | iOS 17 | **NOT RUN** | no iPhone in the sandbox — the same plan |
| macOS — Music (audio only) | macOS 14 | **NOT RUN** | the same plan, step for `FR-030` |
| iOS — Photos (the photo transfer) | iOS 17 | **NOT RUN** | the same plan, step 9 |
| Apple TV app (a FairPlay-DRM title) | tvOS | **NOT RUN** | the same plan, step 7 |
| A reference property-list implementation | — | **NOT RUN** | `NFR-INT-003` asks for a corpus against a reference decoder; the sandbox has no `plutil` and no reference decoder — the corpus is not in the repository |

What **is** proved is the side of the protocol that does not need a peer:
the parsers, the answers, the limits, the state machine and the
hardening. The interoperability with a real sender is a device test and is
recorded as **NOT RUN** here — not as passed.

## 8. Known limitations

| # | The limitation | Severity | Where | What the owner has to decide |
|---|---|---|---|---|
| **L-1** | **Four RTSP verbs have no test.** `SETUP`, `TEARDOWN`, `FLUSH` and `PAUSE` are routed in `engines/airplay/src/rtsp/router.rs`, but `FR-017` demands "a test per method" and none of the four has one. The teardown of a **live** session — the pass condition of `FR-031` — is therefore not proven either. | **S2** | `engines/airplay/src/rtsp/router.rs`, `engines/airplay/src/rtsp/router/tests.rs` | four tests, one per verb, with the status code and the headers |
| **L-2** | **The pairing store is in memory only.** `FR-023` and `NFR-PRIV-003` name a file with the mode `0600`; the only implementation is `InMemoryPairingStore`, and `store_permissions` is read by no code. A pairing therefore does **not** survive a restart of the receiver, and the lock-out counter restarts with it. | **S2** | `engines/airplay/src/pairing/store.rs`, `config/security-policy.toml` | a file store with the configured mode, and a test that reads the mode of the written file |
| **L-3** | **The mirrored stream is not decrypted.** The key material is derived (`mirror::stream_key_material`), but the media reaches the sink **undecrypted** (`engine/connection.rs:199`). `FR-024` asks for the session key to be decrypted and the stream to be decrypted with it; the FairPlay unwrap refuses with a typed `Unsupported` (that part is `EXC-001` and is correct), but the *mirroring* AES-128-CTR path — which [`docs/guides/PROTOCOL_OVERVIEW.md`](../guides/PROTOCOL_OVERVIEW.md) §2 describes as implemented ("decrypts the stream with the session key of the pairing") — is not in the code. | **S1** | `engines/airplay/src/fairplay.rs:164`, `engines/airplay/src/engine/connection.rs:199` | either the AES-128-CTR decryption of the mirroring path, or the correction of the guide |
| **L-4** | **The core drops the metadata.** `SessionEvent::MetadataChanged` and `ArtworkChanged` are folded with `=> {}` in `core/phairplay-core/src/session.rs:186`; `FR-026` names a test of that folding. The event reaches the dashboard through the event channel, but the **snapshot of the session** of the core carries no title, no artist and no artwork. | **S2** | `core/phairplay-core/src/session.rs:186` | fold the events into `SessionSnapshot` and a test of the fold |
| **L-5** | **The format crates of M1.0 are not the ones the AirPlay engine uses.** `formats/h264`, `formats/aac` and `formats/alac` have **no** consumer in the workspace; the engine carries its own SPS parser and its own property-list codec. The hardening of one copy does not protect the other. | **S3** | `engines/airplay/src/h264.rs`, `engines/airplay/src/plist/`, `formats/{h264,aac,alac}` | one implementation per format, and the engine that uses it |
| **L-6** | **An unknown route answers `501`, not `404`.** `FR-018` asks for `404`; `engines/airplay/src/rtsp/http/mod.rs` answers `not_implemented()` for an unknown path (`rtsp::http::tests::an_unknown_get_route_is_not_implemented`). The requirement and the code disagree. | **S3** | `engines/airplay/src/rtsp/http/mod.rs:80,103` | change the code or change the requirement — one of the two, in the same commit |
| **L-7** | **The advertised codecs are not derived from the capabilities.** `MediaCapabilities::supports_cast_baseline` and `supports_miracast_baseline` exist in the core and are tested, but **no engine calls them**; the Miracast engine hard-codes `wfd_audio_codecs: LPCM 00000003 00` (`engines/miracast/src/rtsp.rs:243`). `FR-007` and `FR-034` therefore hold only in the core, not in the advertisement. | **S2** | `engines/miracast/src/rtsp.rs:243`, `core/phairplay-core/src/capabilities.rs` | the engines ask the core before they advertise |
| **L-8** | **The re-advertisement after a network change and the liveness check do not exist.** The two keys of the configuration (`discovery.re_advertise_on_network_change`, `discovery.advertisement_check_interval_seconds`) are read by **no** Rust code, and the discovery crate has no path that re-registers a service. `FR-012` and `FR-013` describe a behaviour that the tree does not have. | **S2** | `engines/discovery`, `config/app-defaults.toml` | the path, or the removal of the two keys and the two requirements |
| **L-9** | **No hardware test has been run.** Every interoperability row and every performance row of this record is **NOT RUN**. The project does not claim otherwise. | **S1** | §3, §7, §9 | the device test plan, on a network with an Apple sender |
| **L-10** | **The media path is the platform's job.** The engine delivers *encoded* frames to a sink; the decoding belongs to the native shell (M3). A receiver that cannot decode a codec does not advertise it — but the advertisement does not follow the capabilities yet (L-7). | **S2** | `ARCHITECTURE.md`, the shells of M3 | the shells of M3 and the wiring of L-7 |
| **L-11** | **The audio path is not tested.** `pump_audio` (`engines/airplay/src/engine/ports.rs:145`) delivers the audio packets to the audio sink through a bounded mailbox — and **no test runs it**. The audio frame is delivered with `pts_us: 0` (`ports.rs:160`), so the receiver cannot compute the deviation between the audio and the video from what it delivers. `FR-030` asks that the audio reaches the sink, `FR-028` asks for the synchronisation. | **S2** | `engines/airplay/src/engine/ports.rs:145` | a test of the audio path, and the presentation timestamp of the audio |

The exclusions that apply to this milestone: `EXC-001` (FairPlay DRM — the
receiver says so and refuses cleanly), `EXC-002` (the in-app audio of the
Apple Music app is protected on every AirPlay path), `EXC-007` (the buffered
audio mode is accepted by the handshake and not played back), `EXC-008` (one
peer at a time on AirPlay).

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| The time to the first picture | a budget of a **running** receiver with an Apple sender | the device test plan §2.1, step 2 |
| The sustained frame rate | same | the plan §3, the counters of the dashboard |
| The resident memory | same | the plan §2.1, step 11 — `ps -o rss= -C phairplay-web` |
| The CPU usage | same | the plan §2.1, step 11 — `ps -o pcpu= -C phairplay-web` |
| The synchronisation of the audio and the video | a recording of a minute of a real session | the plan §3 |
| The handshake of an iPhone or a Mac | no Apple device in the sandbox | the plan §2.1, steps 1 and 2 |
| The rejection of the second AirPlay peer **by a sender** | two Apple devices | the plan §2.1, step 3 (the unit test of the engine is green) |
| The re-advertisement after a Wi-Fi cycle | a network whose interface is taken down and brought up again | the plan §2.1, step 6 |
| The PIN flow with a real sender | an Apple device | the plan §2.1, step 8 |
| The photo transfer with a real sender | an iPhone | the plan §2.1, step 9 |
| The reverse remote against a real sender | a Mac | the plan §2.1, step 10 |
| The corpus of the formats against a reference implementation | no reference decoder in the sandbox (`NFR-INT-003`) | a machine with `plutil` and a reference H.264/AAC decoder |
| The screen reader and the keyboard of a panel | M1 delivers no panel; the shells are M3 | the record of M2 (the dashboard) and the record of M3 (the shells) |
| The Gradle and the Xcode build of the shells | M1 delivers no shell; the sandbox has no Android SDK licence and no macOS SDK | the record of M3 |

## 10. Decision

**ACCEPTED with a documented limitation — the code is accepted, the product
is not.**

What is measured: the workspace compiles, the lint is clean, the format is
clean, the documentation links resolve, the licence and the security gates
are green, **576 tests of the crates of M1 pass** (1224 of the whole
workspace), and the parsers of the AirPlay path reject the attacks that the
requirements name — the cyclic property list, the oversized length, the
SPS with 31 leading zero bits, the oversized fragment, the header flood,
the stalled peer, the external entity.

The reservations are named, not hidden:

1. **The device tests are outstanding.** This record proves the software, not
   the product on a television. The milestone may **not** be announced on the
   strength of this record; it may only be announced together with the
   device evidence of [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md).
2. **The performance budgets are unmeasured.** They are budgets; a budget is
   a target that a measurement has to prove.
3. **Four RTSP verbs have no test** (L-1) and **the teardown of a live
   session is not proven** — the gate of the milestone says "Green" for the
   teardown, and the honest answer is "implemented, not proven".
4. **The mirrored stream is not decrypted** (L-1 → L-3, severity **S1**).
   The sender encrypts the stream, the receiver derives the key and hands
   the **encrypted** frames to the sink. This is the biggest gap of the
   milestone, and it contradicts a sentence of
   [`docs/guides/PROTOCOL_OVERVIEW.md`](../guides/PROTOCOL_OVERVIEW.md) §2.
   Until it is closed, the milestone is a **control plane**, not a
   **picture**.
5. **The pairing does not survive a restart** (L-2) and **the core drops
   the metadata** (L-4).

The decision was taken by the Rust QA Engineer on 2026-10-24 against the
commit `2d0c4b4` of the branch `feat/other-devices`. The owner of the
project has to prioritise L-1 … L-5 before the milestone is announced.

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.
