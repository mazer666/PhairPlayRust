# The acceptance of M5 — the measurement report

> **Status:** the acceptance record of M5 exists:
> [`docs/acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md).
> This document is the evidence behind it: the commands that were run, the
> numbers that came out of them, the gates that stayed open and everything
> that was **not** measured.
>
> **Commit:** `feat/other-devices` at `2d0c4b4` ("fix(ci): the licence gate has
> to pass on a clean checkout"), working tree clean before this round.
> **Date:** 2026-10-24.
> **Machine:** Linux x86_64, 2 CPUs, 3 GB RAM, sandbox with a loopback
> interface and the multicast group `239.255.255.250`. **No Galaxy device, no
> Samsung television, no second host.**
> **Toolchain:** `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustc 1.99.1`.

---

## 1. What was measured

Every command of this section was run in the sandbox with
`HOME=/workspace/.home` and
`CARGO_TARGET_DIR=/workspace/target-qa5`. The target directory was empty at
the beginning, so the first run also carries the cold build of the
dependencies.

### 1.1 The tests of the crate of the milestone

```text
$ CARGO_TARGET_DIR=/workspace/target-qa5 cargo test --offline -p phairplay-smartview
```

| Block | Tests | Failed | Ignored | Duration (the warm run) |
|---|---:|---:|---:|---:|
| unit tests (`src/lib.rs`) | 75 | 0 | 0 | 1.12 s |
| `tests/control_plane.rs` | 7 | 0 | 0 | 0.79 s |
| `tests/discovery.rs` | 5 | 0 | 0 | 2.14 s |
| `tests/hardening.rs` | 12 | 0 | 0 | 0.30 s |
| doctests | 13 | 0 | 0 | 0.06 s |
| **sum** | **112** | **0** | **0** | **4.41 s** |

The cold run of the same command ended with
`Finished \`test\` profile [unoptimized + debuginfo] target(s) in 6m 46s`;
the warm run with an unchanged tree ended with `Finished ... in 16.76s` and
a wall clock of `real 0m44.102s` (the cargo check plus the four test
binaries plus the doctests).

**The number was confirmed at the end of the round.** The same command was
run once more after the other workstreams had touched the tree (the crate
`phairplay-core` had gained a module, the manifest of the crate had changed
the form of its `socket2` dependency): `75 / 7 / 5 / 12 / 13` — again
**112 passed, 0 failed, 0 ignored**, exit code 0. The durations of the two
runs differ (3.37 s against 4.41 s of test time); the counts do not.

**This is the only source of the test numbers of the milestone.** The
numbers in the acceptance record are the numbers of this table.

### 1.1b The tests of the bridge — the composition root

```text
$ CARGO_TARGET_DIR=/workspace/target-qa5 cargo test --offline -p phairplay-ffi --lib
running 43 tests
test composition::tests::a_protocol_that_is_switched_off_is_not_started ... ok
test composition::tests::every_protocol_is_answered_for ... ok
test composition::tests::the_engines_are_registered_with_the_supervisor ... ok
test composition::tests::the_start_reports_the_protocols_that_run ... ok
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
```

These four tests are the evidence for the gate *the fifth protocol is
registered with the supervisor and can be switched off in the
configuration*.

### 1.2 The lint, the format and the structural gates

| Gate | Command | Result |
|---|---|---|
| The lint of the crate | `cargo clippy --offline -p phairplay-smartview --all-targets -- -D warnings` | exit 0, no warning, `Finished ... in 1m 02s` |
| The format of the workspace | `cargo fmt --all -- --check` | exit 0, no output |
| The crate boundaries | `bash scripts/check-crate-boundaries.sh` | exit 0 — *13 crates, 15 internal edges, 0 violations* |
| The negative test of the boundary gate | `bash scripts/check-crate-boundaries.sh --self-test` | exit 0 — the gate answers with exit code 1 for `formats → engines`, `core → formats`, `engines → platform`, `platform → formats` and `app-backend → formats`, and with exit code 0 on the unmodified copy |
| The length of the sources | `bash scripts/check-file-size.sh` | exit 0 — *369 files measured, 2 skipped, 9 above the soft limit, 0 above the hard limit* |
| The licence headers | `bash scripts/check-license-headers.sh` | exit 0 — *13 manifests, 251 source files, 10 derived files, 0 violations* |
| The links of the documentation | `bash scripts/check-docs-links.sh` | exit 0 — *68 files, 691 internal links, 0 broken, 16 budgets compared, 7 budgets owned alone* |

The documentation gate was run twice: the first run was **red** and named a
budget drift that the new acceptance record had caused — the hard limit of
the file length had been repeated in the document, and the limit belongs to
`config/performance-budgets.toml` alone. The number was removed and the
second run was green. The red output is kept here, because a gate that has
never been red has never been tested:

```text
!!! 1 budget drift(s) found:
    hard_max_lines_per_file: the value 550 is written down in 1 place(s) as well () — it
    belongs to config/performance-budgets.toml alone
…
Broken links           : 0
FAILED: the documentation is not consistent with the repository.
EXIT=1
```

### 1.3 The receiver

The binary of the receiver was built from this tree
(`cargo build --offline -p phairplay-web`, `Finished ... in 3m29.850s`) and
run three times.

**Run 1 — the default configuration of the generic profile:**

```text
$ phairplay-web --profile generic --no-dashboard --port 7879
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         advertising
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View not started — protocol engine error: smartview engine could not be
  started: the SSDP responder could not be started: the SSDP socket could not be opened:
  Address already in use (os error 98) -> …
phairplay-web: 4 of 5 protocols are running; the reason of every failure is in the status of
the protocol.
phairplay-web: the API is served on http://127.0.0.1:7879 (the dashboard is switched off)
```

**Run 2 — the same profile with the DLNA renderer switched off:**

```text
  DLNA / UPnP      not started — the protocol is switched off in the configuration
  Samsung Smart View advertising
```

**Run 3 — the same profile with the fifth protocol switched off:**

```text
  DLNA / UPnP      advertising
  Samsung Smart View not started — the protocol is switched off in the configuration
```

**The mechanism of the conflict, measured on this host.** The DLNA
responder binds `0.0.0.0:1900` without `SO_REUSEADDR`
(`engines/dlna/src/ssdp.rs:184`), the Smart View responder with it
(`engines/smartview/src/ssdp.rs:322`). A two-socket experiment on this host
shows the rule of Linux:

```text
first socket  (no SO_REUSEADDR)  : bound 0.0.0.0:1900
second socket (SO_REUSEADDR)    : REFUSED -> Address already in use (errno 98)
both sockets  (SO_REUSEADDR)    : both bound 0.0.0.0:1900
```

The DLNA engine is started before the Smart View engine — the order of
`ProtocolKind::ALL` — so the renderer holds the port and the Smart View
responder is refused.

### 1.4 The probes against the running receiver

The receiver of run 2 was left running and was asked the questions a
Galaxy device asks.

**The status of the fifth protocol** (`GET http://127.0.0.1:7879/api/status`):

```json
{"protocol":"smartview","display_name":"Samsung Smart View","state":"advertising",
 "enabled":true,"tcp_port":8001,"udp_port":1900,"peer_count":0,"last_error":null}
```

**The search of the framework** (`M-SEARCH` to `127.0.0.1:1900`):

```text
HTTP/1.1 200 OK
CACHE-CONTROL: max-age=1800
EXT:
LOCATION: http://172.25.29.225:8001/description.xml
SERVER: Linux/1.0 UPnP/1.0 PhairPlayRust/1.0
ST: urn:samsung.com:service:MultiScreenService:1
USN: uuid:00000000-0000-0000-0000-000000000000::urn:samsung.com:service:MultiScreenService:1
```

A search for another service (`urn:schemas-upnp-org:device:MediaRenderer:1`)
was **not** answered.

**The advertisement path, on the loopback** — the first answer after
0.76 ms, twenty further searches: minimum 0.09 ms, median 0.10 ms,
maximum 0.16 ms. This is a loopback measurement of the *announcement path*;
it is **not** the budget of the milestone and does not replace it.

**The description document** (`GET /description.xml`): `HTTP/1.1 200 OK`,
`<friendlyName>PhairPlay</friendlyName>`,
`<serviceType>urn:samsung.com:service:MultiScreenService:1</serviceType>`,
`<baseURL>http://172.25.29.225:8001/api/v2/</baseURL>`.

**The API root** (`GET /api/v2/`): `HTTP/1.1 200 OK` and
`{"baseUrl":"http://172.25.29.225:8001/api/v2/",…,"service":"urn:samsung.com:service:MultiScreenService:1","version":"2"}`.

**The handshake** (`POST /api/v2/` with `{"mac":"aabbccddeeff","ip":"192.168.1.20","name":"Galaxy S23"}`):
`HTTP/1.1 200 OK` and
`{"accepted":true,…,"peer":{"ip":"192.168.1.20","mac":"aabbccddeeff","name":"Galaxy S23"}}`.

**The channel** (`GET /api/v2/websocket` with the headers of RFC 6455):
`HTTP/1.1 101 Switching Protocols` with
`Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` — the value of the
example of RFC 6455, section 1.3. A second channel while the first one is
open: `HTTP/1.1 409 Conflict`.

**The limits, over a real socket against the running receiver:**

| The hostile request | The answer |
|---|---|
| a request line of 9000 bytes | `HTTP/1.1 414 URI Too Long` |
| 200 header lines | `HTTP/1.1 431 Request Header Fields Too Large` |
| a header line of 9000 bytes | `HTTP/1.1 431 Request Header Fields Too Large` |
| a body that announces 100 MiB | `HTTP/1.1 413 Payload Too Large` |
| a content length that is no number | `HTTP/1.1 400 Bad Request` |
| a chunked body | `HTTP/1.1 400 Bad Request` |
| the receiver after the six refusals | `HTTP/1.1 200 OK` |

### 1.5 The whole workspace — attempted, **not completed**

`cargo test --offline --workspace` was attempted **twice** on this machine.
Both attempts stopped in the compilation, and both times in a file that
**another workstream was editing at that moment**:

```text
attempt 1:
error[E0774]: `derive` may only be applied to `struct`s, `enum`s and `union`s
   --> engines/dlna/src/ssdp.rs:171:1
error: could not compile `phairplay-dlna` (lib) due to 2 previous errors

attempt 2:
error[E0308]: mismatched types
   --> core/phairplay-core/src/ssdp.rs:261:9
error: could not compile `phairplay-core` (lib) due to 1 previous error
```

Both files belong to the fix of the defect **L-1** (the SSDP port of the
workspace), which another workstream was carrying out in parallel with this
acceptance. The crate of the milestone and the bridge compiled and passed
in every run; the run of the workspace did not get that far, because the
tree was moving under it.

**The run of the workspace is therefore NOT RUN** — with the exact reason
above. It is not reported as a failure of the milestone and not as a
success.

---

## 2. The gates of the milestone, one line each

The gates of the milestone M5 in [`ROADMAP.md`](../../ROADMAP.md). Every
line gets a result and a source.

| The gate of the roadmap | Result | The source of the result |
|---|---|---|
| **Functional** — a Galaxy device finds the receiver in the list of *Smart View* and connects to it | **NOT RUN** | no Galaxy device in the sandbox; the device test plan has no Smart View scenario yet |
| **Functional** — the description document is served and the handshake of the sender is answered | **green** | `tests/control_plane.rs` → 7 passed; and the running receiver, §1.4 |
| **Functional** — the engine starts, advertises, stops and frees its ports | **green** | `the_engine_advertises_after_it_started`, `the_engine_stops_and_frees_the_ports` (75 unit tests, 0 failed) |
| **Functional** — the protocol is among the protocols that the receiver starts | **NOT met in the default configuration** | the run of the receiver: `Address already in use (os error 98)`; with the DLNA renderer off: `Samsung Smart View advertising` |
| **Technical** — the engine is a `ProtocolEngine`, one peer at a time, every read has a deadline, every input is length-limited | **green** | `impl ProtocolEngine for SmartViewEngine` (`engine.rs:187`); `409` for the second sender; `ApiLimits::default()` — 5 s and 30 s; 12 negative tests |
| **Technical** — the crate pulls no web framework into the workspace | **green** | `engines/smartview/Cargo.toml` — no `axum`, `tower`, `hyper`, `tokio-tungstenite` |
| **Performance** — the time from the start of the process to the advertisement stays inside the budget | **NOT RUN** | the central budget file owns no key for the advertisement path (L-7); the loopback measurement of §1.4 is not a budget |
| **Usability** — the name of the device in the list of a Galaxy device is the configured name and nothing else | **green** | the running receiver: `<friendlyName>PhairPlay</friendlyName>` — the name of the generic profile |
| **Security and privacy** — the engine opens no connection outside the local subnet and reads no configuration of the user | **green** | the crate has no outbound client in its production code; the engine receives the identity, the capabilities and the sinks through the `EngineContext` of the core |
| **Interoperability** — the picture of a Galaxy device reaches the display of the receiver | **NOT RUN and not implementable yet** | the message format of the channel is proprietary — `EXC-011` |
| **Documentation** — the honest limit is written down in the requirements, the glossary, the architecture and the user-facing documentation | **green** | `EXC-011` in `REQUIREMENTS.md`; `GLOSSARY.md`; `ARCHITECTURE.md` §4.5; `README.md`; `engines/smartview/README.md`; `engines/smartview/src/lib.rs` |
| **QA evidence** — the acceptance record of M5 | **green** | this round: `docs/acceptance/M5-ACCEPTANCE.md` |

**The two gates of the task of this round, answered individually:**

| The gate | Result | The source of the result |
|---|---|---|
| The service type `_samsungmsf._tcp` is announced, and the search for the search target is answered | **partly green** | the *search target* `urn:samsung.com:service:MultiScreenService:1` is announced and answered — `tests/discovery.rs` → 5 passed, and the run of the receiver, §1.4. The *DNS-SD type* `_samsungmsf._tcp` is **not published**: the vocabulary of the core names it (`core/phairplay-core/src/protocol.rs:94`), but no component of the receiver publishes a record for it — the discovery crate has no case for it (`engines/discovery/src/mdns/config.rs`) and is not registered in the composition root. The discovery of the fifth protocol is SSDP-only, which is the path Samsung documents |
| The description and the API answer, and the limits (header count, header length, body length, frame length) apply | **green** | the running receiver, §1.4, and the 12 negative tests of `tests/hardening.rs` |
| The new kind of protocol is in `ProtocolKind`, can be switched off in the configuration, and the engine is registered and started in the composition root | **green** | `ProtocolKind::SmartView` (`core/phairplay-core/src/protocol.rs:36`) with the port `8001` and the service type; the switch: the run 3 of §1.3 — `not started — the protocol is switched off in the configuration`; the registration: `composition.rs:130` and the test `the_engines_are_registered_with_the_supervisor` |
| The limit of the gate rule: the bridge may now depend on the engines (ADR-0006), an application may not | **green** | the boundary gate and its self-test: `platform → engines` is legal, `platform → formats` and `app-backend → formats` are refused with exit code 1; `apps/web/Cargo.toml` depends on `phairplay-core` and `phairplay-ffi` only |

---

## 3. What was NOT measured, and why

### 3.1 The checks that did not run

| Check | Why not | What would be needed |
|---|---|---|
| The interoperability with a real Galaxy phone or tablet | no Galaxy device and no second host in the sandbox | a Galaxy device on the same network, the script of the device test plan |
| The five performance budgets | three of them describe a picture that does not flow (`EXC-011`); the other two describe the running receiver on a real device | the device test plan |
| The picture of a Galaxy device | the message format of the channel is proprietary and is not implemented | the documentation of the format by Samsung, or a port under a licence that allows it |
| The browser gate of the fifth dashboard card | the browser gate belongs to the dashboard milestone; this round did not re-run it | `node apps/web/tools/check-layout.mjs`, `check-contrast.mjs`, `check-keyboard.mjs` |
| `cargo-deny` and `cargo-audit` | the tools are not installed in the sandbox and the registry is not reachable offline | the CI jobs *Licenses (cargo-deny, pinned)* and *Security Audit (cargo-audit)* |
| The build of the native shells (Android, macOS) | no Android SDK and no macOS SDK in the sandbox | the Gradle and the Xcode build |
| The run of the whole workspace | the tree was edited by another workstream while the run was in progress — both attempts stopped in the compilation (§1.5) | `cargo test --offline --workspace` on a tree that no other workstream edits, or the CI job *Quality Gate* |

### 3.2 The limits of the method

- **The loopback is not the network.** Every measurement of the discovery
  was taken on `127.0.0.1` and on the multicast group of the sandbox. A
  Galaxy device reaches the receiver over a real network, with a real
  switch, with a real firmware. The tests prove the protocol against the
  *documented* behaviour — the search target, the `LOCATION`, the API root,
  the handshake, RFC 6455 — and they cannot prove that a specific firmware
  accepts them.
- **The tests are not a Galaxy device.** The sender in the tests is a
  socket that sends what the documentation of Samsung says. The wire format
  of the handshake (`mac`, `ip`, `name`) and the path of the channel
  (`/api/v2/websocket`) are **our own** — they are the open points 5 and 6
  of [`smartview-engine.md`](smartview-engine.md). The first test against a
  real device may show that both have to change.
- **The build was killed once.** The first attempt to build the receiver
  died while the crate `phairplay-cast` was compiled — the machine has
  3 GB of RAM and several builds ran in parallel. The second attempt
  finished. The numbers of the build in §1.3 are therefore the numbers of
  the **second** attempt, with the dependencies already compiled; the cold
  build of the whole tree was not repeated.
- **The tree moved under the measurement.** While this round ran, another
  workstream changed `engines/dlna/src/ssdp.rs` and
  `core/phairplay-core/src/ssdp.rs` — both changes concern the SSDP port of
  the workspace, that is: the defect **L-1**. The measurements of §1.1 to
  §1.4 are the measurements of the state of the tree at the moment of the
  run — the commit `2d0c4b4` for everything that the other workstreams had
  not touched. The fix of L-1 was in flight and was **not** measured here.
  The consequence is named in §1.5: the run of the whole workspace did not
  get past the compilation.
- **No coverage was measured.** There is no coverage number in this
  document, because no coverage tool was run. The claim of the record is
  the number of the tests that pass, not a percentage of the lines.

---

## 4. The open points for the owner

| # | The point | Severity | The source |
|---|---|---|---|
| L-1 | The fifth protocol does not start in the default configuration: the DLNA renderer and the Smart View responder contest the SSDP port 1900, and the renderer holds it first | **S1** | §1.3 |
| L-2 | The service type `_samsungmsf._tcp` is in the vocabulary of the core, but no component of the receiver publishes a record for it | **S2** | §2 |
| L-3 | The device test plan has no Smart View scenario, so the gate that needs hardware has no script | **S2** | `docs/guides/DEVICE_TEST_PLAN.md` |
| L-4 | The browser gate of the fifth dashboard card was not re-run | **S3** | §3.1 |
| L-5 | Two stale places of the documentation: `REQUIREMENTS.md` FR-104 names `apps/web/src/receiver.rs` as the composition root (it is `platform/phairplay-ffi/src/composition.rs` since ADR-0006), and the checklist of the roadmap marks the dashboard card as not done while the code delivers it | **S3** | `REQUIREMENTS.md`, `ROADMAP.md`, `apps/web/assets/api.js:33` |
| L-6 | The identity of the receiver is the default one (`uuid:00000000-…`) because the generic profile names no device id | **S4** | §1.4 |
| L-7 | The performance gate of the roadmap names "the budget of the advertisement path", but the central budget file owns no key for it — the gate cannot be measured against a number until the owner adds one | **S3** | `config/performance-budgets.toml` |
| L-8 | The path of the WebSocket channel and the shape of the JSON bodies are our own; a real device may use another path | **S2** | `smartview-engine.md` §7.5, §7.6 |

The severity scale: **S0** fatal or security, **S1** the function of the
milestone is blocked, **S2** the function is limited, **S3** cosmetic, **S4**
a wording.

---

## 5. The files of this round

| Path | What it is |
|---|---|
| `docs/acceptance/M5-ACCEPTANCE.md` | the acceptance record of the milestone, in the format of `docs/acceptance/ACCEPTANCE_TEMPLATE.md` |
| `docs/audit/acceptance-m5.md` | this document — the evidence behind the record |

No other file of the repository was changed by this round. The scratch
files of the measurement (the probes of §1.4, the configuration overlays of
§1.3) live outside the repository, in `/tmp`.
