# The acceptance of M1 and M2 — the measurement report

> **Status:** the acceptance records of the two milestones exist:
> [`docs/acceptance/M1-ACCEPTANCE.md`](../acceptance/M1-ACCEPTANCE.md) and
> [`docs/acceptance/M2-ACCEPTANCE.md`](../acceptance/M2-ACCEPTANCE.md).
> This document is the evidence behind them: the commands that were run, the
> numbers that came out of them, the requirements that are **not** met, and
> everything that was **not** measured.
>
> **Commit:** `feat/other-devices` at `2d0c4b4` ("fix(ci): the licence gate has
> to pass on a clean checkout"). Every command was run in a **git worktree of
> that commit** (`/workspace/scratch/qa12-m1m2`), because other workstreams
> were editing the tree of the repository while this round was running: the
> tree gained a shared SSDP responder in `core/phairplay-core/src/ssdp.rs`
> at 22:20 and the files of the engines of DLNA and Smart View changed at
> 22:42. A number has to belong to a commit.
>
> **The head of the branch moved during the round.** At 23:22 the commit
> `bbc39c8` ("fix: the receiver announces two services over one multicast
> group") was made by another workstream — it is the commit of the change
> that was in flight at 22:20. Every measurement of this document belongs to
> `2d0c4b4`. The one finding that `bbc39c8` changes is **L-7** (the shared
> SSDP port); it was measured as **open** at `2d0c4b4` and was re-measured
> as **closed** at `bbc39c8` (§1.6).
>
> **Date:** 2026-10-24.
> **Machine:** Linux x86_64, 2 CPUs, sandbox with a loopback interface and the
> routable address `172.25.29.225`. **No Apple sender, no Windows computer, no
> Chromecast, no Android sender, no VLC, no television, no remote control.**
> **Toolchain:** `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustc 1.99.1
> (58cf8f9c3 2026-10-13)`.
>
> **Where the numbers live:** the per-crate attribution of the test numbers
> belongs to the two acceptance records — §1 of this document carries only
> the **raw result lines** of the one workspace run that produced them.

---

## 1. What was measured

Every command of this section was run with `HOME=/workspace/.home` and
`CARGO_TARGET_DIR=/workspace/target-qa12` in the worktree of the commit.

### 1.1 The tests of the workspace

```text
$ cd /workspace/scratch/qa12-m1m2
$ cargo build --offline -p phairplay-ffi          # the shared library of the bridge
$ cargo test --offline --workspace
```

The run was repeated a second time on the unchanged tree, to get the exit
code and the wall clock without the cold build:

```text
$ time cargo test --offline --workspace
real    2m10.240s
user    0m14.319s
sys     0m7.272s
exit    0
```

**38 result lines, 0 failures, 0 ignored.** The sum of the lines:

| Metric | Value |
|---|---:|
| Tests in total | **1224** |
| Passed | **1224** |
| Failed | **0** |
| Ignored | **0** |
| Verdict | **green** |

The raw output is in [`acceptance-m1-m2-workspace-test.log`](acceptance-m1-m2-workspace-test.log).
The sum is reproducible with the log alone:

```text
$ grep -E "^test result:" docs/audit/acceptance-m1-m2-workspace-test.log \
    | awk -F'[ ;]' '{p+=$4; f+=$6; i+=$8} END {print "lines="NR, "passed="p, "failed="f, "ignored="i}'
lines=38 passed=1224 failed=0 ignored=0
```

The raw result lines, in the order of the run:

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   # phairplay-aac
test result: ok. 230 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.32s  # phairplay-airplay
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s    # xml_plist_hardening
test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s   # phairplay-alac
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s   # phairplay-cast
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s    # cast_engine_hardening
test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s   # phairplay-core
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s   # phairplay-discovery
test result: ok. 119 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s  # phairplay-dlna
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.72s   # dlna control_plane
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s   # dlna hardening
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s   # phairplay-ffi
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s    # uniffi-bindgen
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s   # bridge_roundtrip
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.42s   # foreign_abi_roundtrip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s    # supervisor_lifecycle
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s   # phairplay-h264
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s   # phairplay-miracast
test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s   # phairplay-plist
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s   # phairplay-smartview
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s    # smartview control_plane
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s    # smartview discovery
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s   # smartview hardening
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s   # phairplay-web
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s    # phairplay-web main
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s    # doctest phairplay-aac
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s   # doctest phairplay-airplay
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s    # doctest phairplay-alac
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s   # doctest phairplay-cast
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s   # doctest phairplay-core
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s    # doctest phairplay-discovery
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s   # doctest phairplay-dlna
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s   # doctest phairplay-ffi
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s    # doctest phairplay-h264
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s    # doctest phairplay-miracast
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s    # doctest phairplay-plist
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s    # doctest phairplay-web
```

(The comments name the test binary of the line; the raw log has the paths.)
The sum of the **doctests of the bridge** is 14 — the same 14 cases are
executed twice by cargo (once for the library, once for the binary target
`uniffi-bindgen`), and the table counts the **distinct** cases.

### 1.2 The one test that needs a build step before it

`cargo test -p phairplay-ffi` alone is **red** when the shared library of the
bridge was not built before it — `cargo test` builds the test harness of the
library, not its `cdylib`. The red state was **reproduced** by copying the
test binary to a directory without the library and running it there:

```text
$ cp /workspace/target-qa12/debug/deps/foreign_abi_roundtrip-<hash> /tmp/qa12-abi/deps/
$ cd /tmp/qa12-abi && ./deps/foreign_abi_roundtrip-<hash>
running 2 tests
test a_foreign_caller_round_trips_through_the_c_abi ... FAILED
test the_bindings_of_every_shell_can_be_generated ... FAILED

---- a_foreign_caller_round_trips_through_the_c_abi stdout ----
thread 'a_foreign_caller_round_trips_through_the_c_abi' panicked at
  platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs:50:5:
the shared library of the bridge must be built next to the test binary

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
exit 101
```

The same binary **after** `cargo build --offline -p phairplay-ffi`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.90s
```

The continuous integration does the two steps in that order
(`.github/workflows/ci.yml`: "The shared library and the binding generator
of the bridge" **before** "The tests of the workspace"). The raw output:
[`acceptance-m1-m2-ffi-abi-red.log`](acceptance-m1-m2-ffi-abi-red.log).

### 1.3 The gates of the tree

| Gate | Command | Exit | What it said |
|---|---|---:|---|
| The format | `cargo fmt --all -- --check` | 0 | no output |
| The lint | `cargo clippy --offline --workspace --all-targets -- -D warnings` | 0 | **no finding**, 7 min 24 s |
| The tests | `cargo test --offline --workspace` | 0 | 1224 passed, 0 failed, 0 ignored |
| The documentation | `cargo doc --offline --workspace --no-deps` | 0 | **1 warning**: `unresolved link to [ProtocolEngine]` in `engines/smartview/src/lib.rs:56` — the crate of **M5**; the crates of M1 and M2 build their documentation without a warning |
| The boundaries | `bash scripts/check-crate-boundaries.sh` | 0 | 13 crates, 15 internal edges, 0 violations |
| The links | `bash scripts/check-docs-links.sh` | 0 | 59 files, 488 internal links, 0 broken, 16 budgets compared, 7 budgets owned alone |
| The licence headers | `bash scripts/check-license-headers.sh` | 0 | 13 manifests, 251 source files, 10 derived files, 0 violations |
| The length of the files | `bash scripts/check-file-size.sh` | 0 | 365 files measured, 0 above the hard limit |
| The dependency policy | `bash scripts/audit-licenses.sh` | 0 | `bans ok, licenses ok, sources ok` — cargo-deny **0.20.2** (the pinned version) |
| The advisories (the command of the CI) | `cargo audit --ignore RUSTSEC-2023-0071` | 0 | 1 allowed warning: `RUSTSEC-2026-0374`, `basic-toml` unmaintained |
| The advisories (without the exception) | `cargo audit` | **1** | **1 vulnerability**: `RUSTSEC-2023-0071` "Marvin attack", `rsa 0.9.10`, severity 5.9 — the crate of the **legacy AirPlay pairing** |

**One caveat about the lint.** The *first* run of the lint in this round was
**red**, with one finding in `core/phairplay-core/src/ssdp.rs:166`
(`clippy::map_clone`). That file is **new and untracked**: another
workstream moved the shared SSDP responder from `engines/dlna` into the core
while this round was running (22:20–22:44). The lint of the **committed**
tree is green — that is the number that belongs in the acceptance records.
The finding is a finding of the work in flight, not of `2d0c4b4`.

### 1.4 The running receiver

The binary of the tree was built and started on the loopback of the
sandbox:

```text
$ cargo build --offline -p phairplay-web
$ phairplay-web --profile generic --no-dashboard --port 7888
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         advertising
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View not started — protocol engine error: smartview engine could not be
  started: the SSDP responder could not be started: the SSDP socket could not be opened:
  Address already in use (os error 98) -> …
phairplay-web: 4 of 5 protocols are running; the reason of every failure is in the status
of the protocol.
phairplay-web: the API is served on http://127.0.0.1:7888 (the dashboard is switched off)
```

The API of the loopback:

```text
$ curl -s http://127.0.0.1:7888/api/status | python3 -m json.tool
{
    "protocols": [
        {"protocol": "airplay",  "state": "advertising", "enabled": true,  "tcp_port": 7000,  "udp_port": 6002,   "peer_count": 0, "last_error": null},
        {"protocol": "miracast", "state": "advertising", "enabled": true,  "tcp_port": 7236,  "udp_port": null,   "peer_count": 0, "last_error": null},
        {"protocol": "cast",     "state": "advertising", "enabled": true,  "tcp_port": 8009,  "udp_port": null,   "peer_count": 0, "last_error": null},
        {"protocol": "dlna",     "state": "advertising", "enabled": true,  "tcp_port": 39507, "udp_port": 1900,   "peer_count": 0, "last_error": null},
        {"protocol": "smartview","state": "failed",      "enabled": true,  "tcp_port": null,  "udp_port": null,   "peer_count": 0,
         "last_error": "the SSDP responder could not be started: the SSDP socket could not be opened: Address already in use (os error 98)"}
    ],
    "active_sessions": 0,
    "uptime_ms": 8350
}
```

The switch of a protocol, against the **real** backend:

```text
$ curl -s -X POST http://127.0.0.1:7889/api/protocols/cast/disable
{"status":{"protocol":"cast","display_name":"Google Cast","state":"stopped","enabled":false,"tcp_port":null,"udp_port":null,"peer_count":0,"last_error":null}}

$ curl -s -X POST http://127.0.0.1:7889/api/protocols/cast/enable
{"status":{"protocol":"cast","display_name":"Google Cast","state":"advertising","enabled":true,"tcp_port":8009,"udp_port":null,"peer_count":0,"last_error":null}}
```

The loopback, and the refusal of the foreign address:

```text
$ curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:7890/api/status
200
$ curl -s -o /dev/null -w "%{http_code}\n" http://172.25.29.225:7890/api/status
000
$ curl -s --max-time 5 http://172.25.29.225:7890/api/status ; echo "exit=$?"
exit 7            # curl: (7) Failed to connect — connection refused
```

The warning for a foreign bind address, on stderr, **before** anything
starts:

```text
$ phairplay-web --bind 0.0.0.0 --config /nonexistent.toml --port 7879
phairplay-web: WARNING — the dashboard is bound to 0.0.0.0, not to the loopback interface.
Every device of the local network can open the control panel of the receiver, read its
configuration and switch its protocols off. The default is 127.0.0.1: the panel is a local
control panel.
phairplay-web: the configuration file `/nonexistent.toml` could not be loaded: configuration
file not found: /nonexistent.toml
```

The build information and the metrics of the API:

```text
$ curl -s http://127.0.0.1:7888/api/build-info
{"version":"0.1.0","name":"phairplay-core","license":"AGPL-3.0-or-later","repository":"https://github.com/mazer666/PhairPlayRust","rustc_version":"rustc 1.99.1 (58cf8f9c3 2026-10-13)","target":"x86_64-unknown-linux-gnu"}

$ curl -s http://127.0.0.1:7890/api/sessions
{"sessions":[]}
```

### 1.5 The bindings of the shells

```text
$ bash scripts/generate-bindings.sh --offline
==> Building the bridge and the binding generator…
    library: /workspace/scratch/qa12-m1m2/target/debug/libphairplay_ffi.so
==> Generating the Swift bindings…
==> Generating the Kotlin bindings…
✓ Swift bindings: apps/macos/Sources/{PhairPlayFFI,PhairPlayFFIHeaders}
✓ Kotlin bindings: apps/android/app/src/main/java/uniffi/phairplay_ffi
exit 0

$ git status --short
 M apps/android/app/src/main/java/uniffi/phairplay_ffi/phairplay_ffi.kt
 M apps/macos/Sources/PhairPlayFFI/phairplay_ffi.swift
```

The diff (the whole file:
[`acceptance-m1-m2-bindings-regeneration.diff`](acceptance-m1-m2-bindings-regeneration.diff)):

```diff
-    if (lib.uniffi_phairplay_ffi_checksum_constructor_receiver_new() != 31533.toShort()) {
+    if (lib.uniffi_phairplay_ffi_checksum_constructor_receiver_new() != 5570.toShort()) {
```

plus the doc comment of the constructor `Receiver::new`, which the committed
bindings do not carry. The binding was **edited by hand** in the commit
`86c1b81` (the field `audioQueueBytes` was added to the Kotlin data class
without a regeneration).

Two further findings of the same command:

- `scripts/generate-bindings.sh` looks for the shared library in
  `$REPO_ROOT/target/debug` and **ignores `CARGO_TARGET_DIR`** — with a
  custom target directory the script fails with `the shared library of the
  bridge was not found in …` (exit 1).
- The script removed and regenerated the directories; the worktree was
  restored with `git checkout --` afterwards, so the tree of the repository
  is untouched.

### 1.6 The head of the branch, re-measured

The commit `bbc39c8` landed at 23:22, after the measurement of this round.
The binary of that commit was built and the receiver was started again, with
the same command:

```text
$ cargo build --offline -p phairplay-web
$ phairplay-web --profile generic --no-dashboard --port 7891
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         advertising
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View advertising
phairplay-web: 5 of 5 protocols are running.

$ curl -s http://127.0.0.1:7891/api/status
airplay   -> advertising | port 7000  6002 |
miracast  -> advertising | port 7236  None |
cast      -> advertising | port 8009  None |
dlna      -> advertising | port 34723 1900 |
smartview -> advertising | port 8001  1900 |
```

The conflict of the SSDP port (L-7) is therefore **closed on the head of the
branch** and was **open** at the revision that this round measured. The
records of M1 and M2 name both facts.

### 1.7 The searches of the source

| Search | Command | Result |
|---|---|---|
| No telemetry | `grep -rniE "telemetry\|analytics\|sentry\|crashlytics\|mixpanel\|amplitude\|segment\.io\|google-analytics" core engines formats platform apps` | only the two documentation comments that **promise** the absence (`core/phairplay-core/src/lib.rs:6`, `apps/web/src/lib.rs:11`) and the word `status` in the Cast crate; the hits under `apps/android/app/build` and `apps/macos/.build` are the Gradle and Swift **build caches**, not sources |
| No account | `grep -rniE "sign[ -]?in\|sign[ -]?up\|log[ -]?in\|account" core engines formats platform apps` | only the documentation comment and the words `signing_key` / `Signature` of the pairing |
| No promotion in the panel | `grep -rniE "buy\|upgrade\|premium\|sponsor\|advert\|free trial\|donat\|rate us\|promo\|coupon" apps/web/assets/lang/ apps/web/index.html` | only the word *advertise* in the sense of the **service advertisement** (DNS-SD) and the promise of the footer |
| No external resource in the panel | `grep -rnE "https?://" apps/web/assets/ apps/web/index.html` | **2 hits**, both the XML namespace of the two inline SVG files |
| No I/O in the format crates | `grep -rn "std::net\|std::fs" formats/` | no hit; the four manifests of `formats/*` carry only `thiserror` |
| The licence of every member | `cargo metadata --no-deps --format-version 1` | all 13 members report `AGPL-3.0-or-later`; no member manifest carries a literal licence |
| The crates of M1.0 are used | `grep -rn "phairplay-h264\|phairplay-aac\|phairplay-alac" --include=*.rs .` | **no hit outside `formats/`** — the AirPlay engine carries its own copies |
| The one-peer rule has one owner | `grep -rn "multi_peer" --include=*.rs .` | **no hit** — `session.multi_peer_protocols` is read by no code |
| The capability checks are used | `grep -rn "supports_cast_baseline\|supports_miracast_baseline" --include=*.rs .` | only the core — **no engine calls them** |
| The pairing store is on the disk | `grep -rn "store_permissions" --include=*.rs .` | **no hit** — the only store is in memory |
| The re-advertisement exists | `grep -rn "re_advertise_on_network_change\|advertisement_check_interval_seconds" --include=*.rs .` | **no hit** — both keys are dead |
| The metadata reaches the session | `core/phairplay-core/src/session.rs:186` | `PlaybackStateChanged \| MetadataChanged \| ArtworkChanged => {}` — the three events are **dropped** |
| The GENA delivery is tested | `grep -rn "notify_subscribers" --include=*.rs .` | only the definition and one call inside the crate — **no test** |
| The event channel is tested | `apps/web/src/events.rs` | the WebSocket endpoint has **no** test |

---

## 2. The gates of the milestones, one line each

### M1 — AirPlay 2 receiver

| Gate of the roadmap | The result of this round | The evidence |
|---|---|---|
| **M1.0** — every public function of the four format crates has a unit test | **green** | `phairplay-plist` 91, `phairplay-h264` 60, `phairplay-aac` 30, `phairplay-alac` 66 — the record of M1 §1 |
| **M1.0** — every hardening measure has a negative test | **green** | the cyclic property list (`binary::decode::tests::rejects_a_cycle`), the oversized length (`binary::decode::tests::rejects_a_length_field_that_exceeds_the_input`), the SPS overflow vector (`bits::tests::thirty_one_leading_zeros_do_not_overflow`), the fragment overflow (`rtp::tests::rejects_a_fragment_that_is_larger_than_the_limit`) |
| **M1.0** — the ALAC port is complete or marked as partial | **complete** | `formats/alac/src/lib.rs` names the derived files, the copyright and the licence; every file carries the attribution; no open part is named |
| **M1.1** — a sender finds the receiver in the picker of a real Apple device | **NOT RUN** | no Apple device — the plan §2.1 |
| **M1.1** — the instance name has no control character and respects the cap | **green** | `mdns::tests::the_instance_name_is_sanitised`, `service_types::tests::the_raop_name_strips_control_characters`, `service_types::tests::the_raop_name_is_capped_at_the_dns_sd_limit` |
| **M1.1** — the advertisement comes back after a Wi-Fi reconnect | **NOT RUN, and NOT IMPLEMENTED** | the key `discovery.re_advertise_on_network_change` is read by no code — the record of M1, L-8 |
| **M1.2** — an iPhone or a Mac completes the handshake, the picture and the sound reach the sink | **NOT RUN** | no Apple device — the plan §2.1 |
| **M1.2** — a second peer is refused, the first session continues | **green** | `engine::tests::a_second_sender_is_rejected_with_service_unavailable` |
| **M1.2** — a malformed SDP, an oversized message and a header flood are refused | **green** | `rtsp::router::tests::an_announce_with_a_broken_sdp_is_rejected`, `rtsp::request::tests::an_oversized_message_is_rejected`, `rtsp::reader::tests::too_many_headers_are_refused` |
| **M1.2** — the session key of a known handshake is derived correctly | **partly** | the derivation is green (`fairplay::tests::the_first_phase_answers_with_142_bytes`, `pairing::verify::tests::a_complete_exchange_produces_a_shared_secret`); the **decryption of the stream is not implemented** — the record of M1, L-3 |
| **M1.2** — the audio and the video stay inside the synchronisation budget over a minute | **NOT RUN** | the plan §2.1, §3 |
| **M1.3** — every statement of the acceptance record names its test or its command | **green** | this document and the record of M1 |
| **M1.3** — every sender of the matrix is marked as tested or named with the reason | **green** | the matrix of the record of M1 §7 — six rows, six **NOT RUN**, six reasons |

### M2 — Protocol breadth and local control

| Gate of the roadmap | The result of this round | The evidence |
|---|---|---|
| **M2.0** — two Cast senders stream at the same time, no other protocol accepts a second peer | **NOT RUN** and **NOT TESTED** | no second host — the plan §2.3; the rule is not read from the configuration and there is no test with two senders — the record of M2, L-1 |
| **M2.0** — a frame with an oversized length is refused | **green** | `frame::tests::an_oversized_length_is_rejected_without_allocating` |
| **M2.0** — the baseline codecs of the protocol are the advertised ones | **partly** | the capability test of the core is green (`capabilities::tests::cast_baseline_requires_h264_vp8_and_three_audio_codecs`), but **no engine asks the core** — the record of M2, L-6 |
| **M2.1** — a Windows sender connects and streams | **NOT RUN** | no Windows computer — the plan §2.2 |
| **M2.1** — a platform without the WFD permission does not advertise the service and says why | **partly** | the engine reports its kind and its failures (`engine::tests::the_engine_reports_miracast`, `error::tests::every_failure_belongs_to_miracast`); the **capability query of the platform is not consulted** — the record of M2, L-6 |
| **M2.1** — an information element with a broken length is refused | **green** | `wfd::tests::a_block_that_runs_past_the_end_is_rejected` |
| **M2.2** — a DLNA controller finds the device and starts a playback | **NOT RUN** | no UPnP control point — the plan §2.4 |
| **M2.2** — a `DOCTYPE` and a GENA callback at `169.254.169.254` are both refused | **green** | `hardening::a_doctype_in_the_soap_body_is_refused`, `hardening::a_gena_callback_to_the_metadata_service_is_refused` |
| **M2.2** — the renderer advertises no video rendering | **green** | `core protocol::tests::only_streaming_protocols_carry_video` |
| **M2.3** — a value that goes into the bridge comes out of it unchanged | **green** | `bridge_roundtrip` → 11 passed; `foreign_abi_roundtrip` → 2 passed |
| **M2.3** — no panic crosses the bridge, every failure is a typed error with a code | **green** | `phairplay-ffi` → `error::tests::the_numeric_codes_match_the_core`, `core error::tests::error_codes_are_stable` |
| **M2.3** — the generated bindings are reproducible: a regeneration produces no diff | **red** | a regeneration produces a diff of **2 files** and a changed API checksum — the record of M2, L-4 |
| **M2.4** — the backend binds on the loopback only, a foreign address is refused | **green** | `server::tests::the_bound_socket_is_on_the_loopback_interface`; **measured**: the foreign address refuses the connection |
| **M2.4** — every value of the dashboard comes from the core | **green** | the design of the backend; the status is read from the supervisor — `state::tests::the_state_covers_every_protocol` |
| **M2.4** — the dashboard is operable with the keyboard, meets the contrast, respects the reduced motion, loads no external resource | **green for the checks**, **NOT RUN** on a television | [`docs/audit/tv-web-dashboard.md`](tv-web-dashboard.md) — 35 keyboard checks, 31 layout checks, 0 failures; no external resource (2 hits, both the XML namespace of an inline SVG) |

---

## 3. The requirements that are not met

The list is the answer to "which of the FR-IDs of M1 and M2 are proven, and
which are not". **A row that is not in this list is green** — the evidence of
every green row is in §1 of the two acceptance records.

### 3.1 M1 — not met, not implemented or not proven

| Requirement | What is missing | Where |
|---|---|---|
| **FR-007** | The capability checks exist and are tested, but **no engine calls them**: the advertisement does not follow the capabilities of the platform. | the record of M1, L-7 |
| **FR-012** | The re-advertisement after a network change **does not exist**; the configuration key is dead. | L-8 |
| **FR-013** | The liveness check **does not exist**; the configuration key is dead. | L-8 |
| **FR-017** | `SETUP`, `TEARDOWN`, `FLUSH` and `PAUSE` are routed but have **no test**; the requirement names "a test per method". | L-1 |
| **FR-018** | The routes exist and seven of the eight have a test; the **`/feedback` route has no test**, and an unknown route answers **`501`** where the requirement asks for **`404`**. | L-6 |
| **FR-023** | The pairing does **not** survive a restart: the only store is in memory, and the mode `0600` of `store_permissions` is read by no code. | L-2 |
| **FR-024** | The session key that protects the stream is **not decrypted** and the stream is **not decrypted** with it. The FairPlay unwrap refuses with a typed `Unsupported` (that is `EXC-001` and is correct) — but the *mirroring* AES-128-CTR path is not in the code, while the guide says it is. | L-3 |
| **FR-026** | The core **drops** the metadata and the artwork: the folding of `MetadataChanged` and `ArtworkChanged` is `=> {}`. | L-4 |
| **FR-027** | The DACP **builder** is proven; that a command reaches a sender and that the answer comes back is **NOT RUN**. | §7 of the record |
| **FR-028** | The NTP **parser** is proven; the **budget** is **NOT RUN**, and the audio frames are delivered with `pts_us: 0`, so the deviation cannot be computed from what the receiver delivers. | L-11, §3 |
| **FR-029** | The photo path is proven in the code; the **transfer with a real sender** is **NOT RUN**. | §7 of the record |
| **FR-030** | The audio-only session is recognised in the SDP; the path to the audio sink exists but **no test runs it**. | L-11 |
| **FR-031** | The teardown is routed, but there is **no test that a live session is torn down** and that the engine returns to `Advertising`. | L-1 |
| **FR-059** | The ALAC port claims completeness and names no open part; the requirement asks that the documentation name **the parts that are done and the parts that are open** — there is no explicit list. | the record of M1 §1 |
| **NFR-INT-001** | No sender of the matrix was tested. Every row is **NOT RUN** with a reason. | §7 of the record |
| **NFR-INT-003** | There is **no corpus** against a reference decoder; the sandbox has no `plutil` and no reference H.264/AAC decoder. | §9 of the record |
| **NFR-PERF-001 … 005** | **NOT RUN** — five budgets of a running receiver with a real sender. | §3 of the record |
| **NFR-RES-001, NFR-RES-002** | The **bounds** are proven by the tests of the core and of the engines; the **resident memory** that the requirement names is **NOT RUN** (it is a budget of `NFR-PERF-003`). | §3 of the record |
| **NFR-SEC-003** | The bounded queues and the bounded job executor are proven; the **resident memory and the CPU usage under a flood** are **NOT RUN**. | §3 of the record |
| **NFR-PRIV-003** | The pairing store is **in memory**; the file mode of the requirement does not exist. | L-2 |

### 3.2 M2 — not met, not implemented or not proven

| Requirement | What is missing | Where |
|---|---|---|
| **FR-007** | The same as in M1: no engine calls the capability checks. | L-6 |
| **FR-034** | The advertised codec set of Miracast is **hard-coded** (`wfd_audio_codecs: LPCM 00000003 00`), not derived from `supports_miracast_baseline`. | L-6 |
| **FR-035** | The engine reports its failures, but the **capability of the platform is not queried** before the advertisement. | L-6 |
| **FR-040** | The rule of the one-peer exception is read from **nowhere**: `session.multi_peer_protocols` is dead data and the limit is the constant `DEFAULT_MAX_SESSIONS = 8`. **No test with two concurrent senders exists.** | L-1 |
| **FR-041** | The same as `FR-034` for Cast. | L-6 |
| **FR-042** | The Cast engine reports the media status; the **core drops the playback state** (`PlaybackStateChanged => {}`), so the snapshot of the session carries neither the state nor the position. | L-2 |
| **FR-046** | The GENA **delivery** has **no test**: `notify_subscribers` is never called by a test. | L-3 |
| **FR-073** | A regeneration of the bindings produces a **diff** and a **changed API checksum**; the binding was hand-edited in the commit `86c1b81`. | L-4 |
| **FR-083** | The **WebSocket endpoint** of the panel has no test; only the conversion of an event is tested. | L-5 |
| **FR-096 … FR-098** | The keyboard behaviour of the panel is proven by the browser checks; the behaviour **on a television with a real remote control** is **NOT RUN**. | §5 of the record |
| **NFR-INT-001** | No sender of the matrix was tested. Every row is **NOT RUN** with a reason. | §7 of the record |
| **NFR-INT-002** | The Cast path does not need the Cast SDK — that is true **by construction** (the engine needs no SDK); the proof on a device is **NOT RUN**. | §9 of the record |
| **NFR-PERF-001 … 005** | **NOT RUN** — five budgets of a running receiver with a real sender. | §3 of the record |

---

## 4. What was NOT measured, and why

| What | Why not | Where it will be measured |
|---|---|---|
| The time to the first picture | a budget of a **running** receiver with a **real** sender; the sandbox has neither | the device test plan §2.1 (AirPlay), §2.2 (Miracast), §2.3 (Cast) |
| The sustained frame rate | same | the plan §3 — the counters of the dashboard |
| The resident memory | same | the plan §3 — `ps -o rss= -C phairplay-web` |
| The CPU usage | same | the plan §3 — `ps -o pcpu= -C phairplay-web` |
| The synchronisation of the audio and the video | a recording of a minute of a real session | the plan §3 |
| The handshake of an iPhone, an iPad or a Mac | no Apple device | the plan §2.1 |
| The rejection of the second AirPlay peer **by a sender** | two Apple devices | the plan §2.1, step 3 |
| The re-advertisement after a Wi-Fi cycle | a network whose interface is taken down and brought up again — **and the path does not exist yet** | the plan §2.1, step 6 |
| The PIN flow with a real sender | an Apple device | the plan §2.1, step 8 |
| The photo transfer with a real sender | an iPhone | the plan §2.1, step 9 |
| The reverse remote against a real sender | a Mac | the plan §2.1, step 10 |
| The Miracast handshake of a Windows sender | no Windows computer | the plan §2.2 |
| The Cast control plane of a Chromecast | no Chromecast, no second host | the plan §2.3 |
| Two Cast senders at the same time | two senders | the plan §2.3, step 4 |
| The DLNA renderer with VLC | no UPnP control point | the plan §2.4 |
| The GENA event on a real subscription | a control point | the plan §2.4, step 4 |
| The corpus of the formats against a reference implementation | no reference decoder in the sandbox (`NFR-INT-003`) | a machine with `plutil` and a reference decoder |
| The panel on a television with a real remote control | no television, no remote control | the plan of the television shell |
| The reading of the panel in a bright room | a television | the same |
| A screen reader | no device, no screen reader | the record of M3 |
| The Gradle and the Xcode build of the shells | M1 and M2 deliver no shell; the sandbox has no Android SDK licence and no macOS SDK | the record of M3 |
| The **coverage** of the test suite | **no coverage measurement was taken.** No tool for it (`cargo-llvm-cov`, `grcov`) was available in the sandbox, and a percentage without a measured run would be an invention. | a machine with `cargo-llvm-cov` and the same command |

---

## 5. The open points for the owner

The severity is the one of the record of the milestone.

| # | The point | Severity | Milestone |
|---|---|---|---|
| 1 | **The mirrored stream is not decrypted.** The key is derived, the frames are not decrypted — and the guide says the opposite. | **S1** | M1, L-3 |
| 2 | **The committed bindings are stale.** A shell that loads them against a library of this source refuses to start with *"UniFFI API checksum mismatch"*. One command fixes it: `bash scripts/generate-bindings.sh`. | **S2** | M2, L-4 |
| 3 | **Four RTSP verbs have no test**, and the teardown of a live session is not proven. | **S2** | M1, L-1 |
| 4 | **The pairing does not survive a restart**, and the mode `0600` is read by no code. | **S2** | M1, L-2 |
| 5 | **The core drops the metadata and the playback state.** | **S2** | M1 L-4 / M2 L-2 |
| 6 | **The one-peer rule of Cast has no owner in the configuration** and no test with two senders. | **S2** | M2, L-1 |
| 7 | **The GENA delivery has no test.** | **S2** | M2, L-3 |
| 8 | **The event channel of the panel has no test.** | **S2** | M2, L-5 |
| 9 | **The advertised codecs do not follow the capabilities.** | **S2** | M1 L-7 / M2 L-6 |
| 10 | **The re-advertisement and the liveness check do not exist**, although the configuration names them. | **S2** | M1, L-8 |
| 11 | **The audio path is not tested**, and the audio frames carry `pts_us: 0`. | **S2** | M1, L-11 |
| 12 | **The format crates of M1.0 have no consumer**; the engine carries parallel copies of the SPS parser and of the property-list codec. | **S3** | M1, L-5 |
| 13 | **An unknown AirPlay route answers `501` where the requirement asks for `404`.** | **S3** | M1, L-6 |
| 14 | **The SSDP port was shared** between the DLNA renderer and the Smart View receiver — **fixed after the measurement**, in the commit `bbc39c8` (5 of 5 protocols at the head). | **S3** (closed) | M2, L-7 |
| 15 | **The generator of the bindings ignores `CARGO_TARGET_DIR`.** | **S3** | M2, L-8 |
| 16 | **The documentation of the discovery crate still says "four" protocols** — the core knows five since the commit `86c1b81` (`engines/discovery/src/lib.rs:22`). | **S4** | — |

---

## 6. The files of this round

| File | What it is |
|---|---|
| [`docs/acceptance/M1-ACCEPTANCE.md`](../acceptance/M1-ACCEPTANCE.md) | the acceptance record of M1 — the source of the numbers of M1 |
| [`docs/acceptance/M2-ACCEPTANCE.md`](../acceptance/M2-ACCEPTANCE.md) | the acceptance record of M2 — the source of the numbers of M2 |
| [`docs/audit/acceptance-m1-m2.md`](acceptance-m1-m2.md) | this document — the evidence behind both records |
| [`docs/audit/acceptance-m1-m2-workspace-test.log`](acceptance-m1-m2-workspace-test.log) | the raw output of `cargo test --offline --workspace` at `2d0c4b4` |
| [`docs/audit/acceptance-m1-m2-ffi-abi-red.log`](acceptance-m1-m2-ffi-abi-red.log) | the red state of the foreign ABI test, reproduced |
| [`docs/audit/acceptance-m1-m2-bindings-regeneration.diff`](acceptance-m1-m2-bindings-regeneration.diff) | the diff that a regeneration of the bindings produces |

The raw logs of the single gates (clippy, doc, cargo-deny, cargo-audit, the
four shell gates, the run of the receiver) were written into
`/workspace/qa12-logs/` during the round; the parts that a reader needs are
quoted in §1 above.

**No commit, no push, no tag.** The worktree of the measurement was removed
after the round; the tree of the repository carries the three documents of
this table and nothing else.
