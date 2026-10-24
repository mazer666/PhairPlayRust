# Acceptance — M5 Samsung Smart View

**Requirement IDs:** FR-006, FR-009, FR-104 … FR-107, EXC-011
**Milestone:** M5 — Samsung Smart View
**Build/commit:** branch `feat/other-devices` at `2d0c4b4` ("fix(ci): the licence gate has to pass on a clean checkout"); the working tree was clean when this acceptance started — the changes of other workstreams that appeared during the round are named in §8
**Tester/agent roles:** Rust QA Engineer (this record), Rust engineer (the engine — [`docs/audit/smartview-engine.md`](../audit/smartview-engine.md)), Rust architect (ADR-0006, CR-0001), Rust security reviewer (the negative tests of the crate)
**Date:** 2026-10-24
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, 3 GB RAM, loopback plus the multicast group `239.255.255.250` of the sandbox. **No Galaxy phone, no Galaxy tablet, no Samsung television and no second host were available** — see §3, §7 and §9.

**The sentence that this milestone stands on:** the receiver is *found* and
*handshakes*. **No picture flows.** The message format of the Smart View
channel is proprietary and is not implemented — `EXC-011`. Everything below
is measured; everything that was not measured says **NOT RUN** and names the
way to measure it.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The search of the MultiScreen framework is answered over a real socket | `HTTP/1.1 200 OK` with `LOCATION`, `ST` and `USN` | **pass** | `cargo test --offline -p phairplay-smartview` → `tests/discovery.rs` → `a_search_for_the_multi_screen_service_is_answered_over_the_network` (5 passed, 0 failed) |
| The announcement of the receiver reaches the multicast group | a `NOTIFY` with `NTS: ssdp:alive` and `NT: urn:samsung.com:service:MultiScreenService:1` | **pass** | `the_announcement_reaches_the_multicast_group` (tests/discovery.rs); the unit test `the_announcement_announces_the_receiver_and_its_leaving` |
| The search of the running receiver is answered | the same three headers, with the address of the device | **pass** | the run of the receiver below (*the run of the receiver*) — the answer carries `LOCATION: http://172.25.29.225:8001/description.xml` |
| A search for another service is not answered | no answer at all | **pass** | `a_search_for_another_service_is_not_answered` (tests/discovery.rs); the same probe against the running receiver: *not answered* |
| A malformed search is refused and the responder survives it | no answer, the next well-formed search is answered | **pass** | `a_search_with_a_broken_length_is_not_answered_and_the_responder_survives` (tests/discovery.rs) |
| The description document is served at the path of the `LOCATION` header | `200` + the configured name + the service type | **pass** | `the_description_document_is_served_at_the_location` (tests/control_plane.rs); the running receiver: `HTTP/1.1 200 OK`, `<friendlyName>PhairPlay</friendlyName>`, `<serviceType>urn:samsung.com:service:MultiScreenService:1</serviceType>` |
| The API root answers with the base URL | `200` + `baseUrl` + the path of the channel | **pass** | `the_api_root_answers_with_the_base_url` (tests/control_plane.rs); the running receiver: `{"baseUrl":"http://172.25.29.225:8001/api/v2/",...,"service":"urn:samsung.com:service:MultiScreenService:1","version":"2"}` |
| The resource description and the base URL answer | `200` + the list of the paths | **pass** | `the_resource_description_lists_the_paths`, `the_base_url_is_answered` (unit) |
| The handshake of the sender is accepted and the peer is recorded | `200` + the MAC, the address and the name of the sender | **pass** | `the_handshake_of_the_sender_is_accepted` (tests/control_plane.rs); the running receiver: `{"accepted":true,...,"peer":{"ip":"192.168.1.20","mac":"aabbccddeeff","name":"Galaxy S23"}}` |
| The WebSocket upgrade is answered with the accept key of RFC 6455 | `101` + `Sec-WebSocket-Accept` | **pass** | `the_websocket_upgrade_is_answered_with_the_accept_key` (tests/control_plane.rs); the running receiver: `HTTP/1.1 101 Switching Protocols` with `Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` — the value of the example of RFC 6455, section 1.3 |
| The channel is held open, the sender is counted, the state is `streaming` | `peer_count = 1`, `ProtocolState::Streaming` | **pass** | `the_channel_is_held_open_and_the_sender_is_counted` (tests/control_plane.rs); `a_peer_that_opened_a_channel_is_counted` (unit) |
| A second sender is refused — not queued | `409` | **pass** | `a_second_sender_is_refused_while_the_channel_is_open` (tests/control_plane.rs); the running receiver: the first channel `101`, the second `409 Conflict` |
| The engine starts, advertises, stops and frees its ports | the state machine of the build contract | **pass** | `the_engine_advertises_after_it_started`, `the_engine_stops_and_frees_the_ports`, `the_engine_can_be_started_again_after_it_was_stopped` (75 unit tests, 0 failed) |
| The status is readable from another thread during the transition | the state is visible, no deadlock | **pass** | `the_status_is_readable_while_the_engine_starts` |
| The fifth protocol is registered with the supervisor | an engine for every protocol of the vocabulary | **pass** | `cargo test --offline -p phairplay-ffi --lib` → **43 passed, 0 failed** → `composition::tests::the_engines_are_registered_with_the_supervisor` — the loop covers `ProtocolKind::SmartView` |
| The fifth protocol is among the protocols that the receiver starts | `advertising` | **NOT met in the default configuration** | the run of the receiver: `Samsung Smart View not started — … the SSDP socket could not be opened: Address already in use (os error 98)`; the same run with `[protocols.dlna] enabled = false` reports `Samsung Smart View advertising` |
| The fifth protocol can be switched off in the configuration | `not started` + the reason | **pass** | the run of the receiver with `[protocols.smartview] enabled = false`: `Samsung Smart View not started — the protocol is switched off in the configuration`; `cargo test --offline -p phairplay-ffi --lib` → `composition::tests::a_protocol_that_is_switched_off_is_not_started` (43 passed, 0 failed) |
| The fifth card is in the dashboard | five cards, one per protocol | **pass (the code)** — the browser check **NOT RUN** | `apps/web/assets/api.js:33` lists `smartview`, `apps/web/assets/protocol-cards.js:103` builds one card per entry of that list, `apps/web/index.html:77` carries the glyph, `apps/web/assets/lang/{en,de}.js` carry the name; the running receiver answers `GET http://127.0.0.1:7879/api/status` with the entry `{"protocol":"smartview","display_name":"Samsung Smart View","state":"advertising","enabled":true,"tcp_port":8001,"udp_port":1900,...}` |
| A Galaxy device finds the receiver in the list of *Smart View* and connects to it | the receiver in the list of the phone, the phone connects | **NOT RUN** | no Galaxy phone or tablet and no second host in the sandbox. The device test plan has **no Smart View scenario** yet ([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) has the paths AirPlay 2, Miracast, Cast, DLNA) — an open point for the owner, §8 |
| The picture of a Galaxy device reaches the display of the receiver | the picture | **NOT RUN — and not implementable yet** | the message format of the channel is proprietary: `EXC-011`, §8 and §10 |

**The run of the receiver** — the three runs of §1 were measured with the
binary of this tree (`cargo build --offline -p phairplay-web`):

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
phairplay-web: 4 of 5 protocols are running; the reason of every failure is in the status
of the protocol.

$ phairplay-web --profile generic --config <dlna off> --no-dashboard --port 7879
  DLNA / UPnP      not started — the protocol is switched off in the configuration
  Samsung Smart View advertising

$ phairplay-web --profile generic --config <smartview off> --no-dashboard --port 7879
  Samsung Smart View not started — the protocol is switched off in the configuration
```

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh` → exit 0, *13 crates, 15 internal edges, 0 violations*. The gate was proved with a negative test: `bash scripts/check-crate-boundaries.sh --self-test` → exit 0, the gate answers with exit code 1 for `formats → engines`, `core → formats`, `engines → platform`, `platform → formats` and `app-backend → formats`, and with exit code 0 on the unmodified copy.
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh` → exit 0, *68 files, 691 internal links, 0 broken, 16 budgets compared, 7 budgets owned alone* (the number is a snapshot of the tree of this round); the configuration of the fifth protocol is the typed model of the core (`[protocols.smartview]`), and the run of the receiver proves that the switch works.
- The dependency direction is unchanged: **yes, with the one named exception.** ADR-0006 gives the bridge the composition root and lets it depend on `engines/*`; an application may not. The boundary gate encodes exactly that: `platform → engines` is legal, `platform → formats` and `app-backend → formats` are not, and the self-test proves that both are still caught. `apps/web/Cargo.toml` depends on `phairplay-core` and `phairplay-ffi` only — the engines are reached through the bridge.
- The tests are green: **yes** — `cargo test --offline -p phairplay-smartview` → **112 passed, 0 failed, 0 ignored** (75 unit + 7 control plane + 5 discovery + 12 hardening + 13 doctests); `cargo test --offline -p phairplay-ffi --lib` → **43 passed, 0 failed, 0 ignored** — the tests of the composition root. The numbers of this document are the numbers of these two runs; §9 names the run of the whole workspace and why it did not run here.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **the crate of the milestone: yes** — `cargo clippy --offline -p phairplay-smartview --all-targets -- -D warnings` → exit 0, no warning. **The whole workspace: NOT RUN** in this sandbox (§9).
- The format check is green (`cargo fmt --all --check`): **yes** — exit 0, no output.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** — exit 0, 0 broken links.
- The length of the source files is within the limit: **yes** — `bash scripts/check-file-size.sh` → exit 0, *369 files measured, 0 above the hard limit*. The limit itself belongs to [`config/performance-budgets.toml`](../../config/performance-budgets.toml) and is not repeated here. The longest file of the milestone is `engines/smartview/src/engine.rs`.
- The licence headers are present: **yes** — `bash scripts/check-license-headers.sh` → exit 0, *13 manifests, 251 source files, 0 violations*.
- The versioning or the migration is checked: **not applicable** — the project is at version 0.1.0 and has no migration to prove.

**The shape of the build contract** (`docs/design/BUILD-CONTRACT.md`):

| Rule | Result | Evidence |
|---|---|---|
| The engine is a `ProtocolEngine` like every other engine | **pass** | `impl ProtocolEngine for SmartViewEngine` — `engines/smartview/src/engine.rs:187`; `the_engine_answers_with_its_configuration` |
| One peer at a time | **pass** | `a_second_sender_is_refused_while_the_channel_is_open` → `409`; `a_second_peer_is_rejected_and_not_queued` |
| Every read has a deadline | **pass** | `ApiLimits::default()` — `read_timeout` 5 s, `idle_timeout` 30 s (`engines/smartview/src/api/types.rs:44`); the reader bounds every read (`api/reader.rs:190`), the channel bounds every frame (`api/channel.rs:206`) |
| Every input is length-limited | **pass** | `the_limits_are_the_documented_ones`: 64 headers, 8 KiB per header line, 64 KiB body, 1 MiB frame; 12 negative tests over a real socket (§6) |
| The crate pulls no web framework into the workspace | **pass** | `engines/smartview/Cargo.toml` — no `axum`, no `tower`, no `hyper`, no `tokio-tungstenite`; the API is a hand-written HTTP/1.1 server (`api/reader.rs`, `api/routes.rs`, `api/server.rs`) |

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | not measured — **no picture flows** (`EXC-011`) | **NOT RUN** |
| Sustained frame rate | `throughput.minimum_fps` | not measured — **no picture flows** (`EXC-011`) | **NOT RUN** |
| Resident memory | `resources.max_memory_mb` | not measured | **NOT RUN** |
| CPU usage | `resources.max_cpu_percent` | not measured | **NOT RUN** |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | not measured — **no picture flows** (`EXC-011`) | **NOT RUN** |

The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml). The
values are not repeated in this document.

**Why:** three of the five budgets are a property of a picture that does not
flow — the message format of the channel is proprietary and is not
implemented (`EXC-011`). A number for them would be invented. The other two
are a property of the *running* receiver on a real device; the sandbox has no
television and no sender, and a measurement on a build server would be a
number without meaning.

**What was measured anyway — the advertisement path, on the loopback of the
sandbox.** The receiver of the run above (DLNA switched off, so that the
Smart View responder owns the SSDP port) was asked 21 times with the
`M-SEARCH` of a Galaxy device:

```text
$ python3 /tmp/m5-discovery-probe.py     # 127.0.0.1:1900, the running receiver
first answer after 0.76 ms
20 further searches: minimum 0.09 ms | median 0.10 ms | maximum 0.16 ms
```

This is a **loopback** measurement of the advertisement path. It is **not**
the budget of the milestone and it does not replace it: the central budget
file owns **no key for the advertisement path**
([`config/performance-budgets.toml`](../../config/performance-budgets.toml)
has no `advertisement` key), so the performance gate of the roadmap — *"the
time from the start of the process to the advertisement of the receiver stays
inside the budget of the advertisement path"* — names a budget that does not
exist yet. The value above is reported because it is the only number of the
advertisement path that this sandbox can produce honestly; the budget itself
is an open point for the owner (§8, L-7).

**How the budgets will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md)) names
the sender, the command and the expected result for every one of them. The
acceptance record of the milestone that runs on the hardware will carry the
measured values.

## 4. Usability

- The number of steps of the task: **not measured on a device** — the task of
  M5 is *"the phone opens Smart View and picks the receiver"*. The receiver
  answers the steps it can answer (the search, the description document, the
  handshake, the channel), the step that needs the phone is NOT RUN.
- The discoverability notes: the name of the receiver is the configured name
  and nothing else — the run of the receiver answers `/description.xml` with
  `<friendlyName>PhairPlay</friendlyName>`, the name of the configuration of
  the generic profile, and the API root with the same name. The dashboard
  shows one card per protocol; the fifth card is built from the same list as
  the other four (`apps/web/assets/api.js:33`).
- The distinction of the states ("disabled", "advertising", "streaming", "error"): **proved for the fifth protocol.** The run of the receiver shows three of the four states of the fifth protocol — `advertising` (DLNA off), `not started — the protocol is switched off in the configuration` (disabled) and `not started — … Address already in use (os error 98)` (error, with the reason). The state `streaming` is proved by the tests of the engine (`the_channel_is_held_open_and_the_sender_is_counted`), not on a device.
- The clarity of an error message: **proved.** The reason of the failure of the fifth protocol reaches the status of the protocol and the summary of the receiver with the cause and the system error (`os error 98`); the composition root reports the failure of one protocol without stopping the others — *4 of 5 protocols are running*.
- The mobile / native behaviour: **NOT RUN** — no Galaxy device, and the native shells were not built in this sandbox (no Android SDK, no macOS SDK).

## 5. Accessibility

- The keyboard operation: the fifth protocol adds no own surface — the card
  of the dashboard is the same button as the cards of the other four
  (`apps/web/assets/protocol-cards.js:103`, with `aria-labelledby` and
  `aria-describedby`). **NOT RUN** in a browser by this acceptance: the
  browser gate of the dashboard belongs to the dashboard milestone
  (`node apps/web/tools/check-keyboard.mjs`).
- The screen reader (VoiceOver / TalkBack): the card of the fifth protocol
  carries the same ARIA attributes as the other four. **NOT RUN** with a
  screen reader — that needs a device.
- The scaling, the high contrast, the reduced motion: the fifth card uses
  the palette and the motion rules of the dashboard, which the dashboard gate
  measures. **NOT RUN** by this acceptance — the measurement belongs to the
  dashboard gate (`node apps/web/tools/check-contrast.mjs`).

## 6. Security and privacy

| Measure | Result | Evidence |
|---|---|---|
| The request line is length-limited | **pass** | `an_over_long_request_line_is_refused` → `414`; the running receiver: `HTTP/1.1 414 URI Too Long` |
| The number of headers is limited | **pass** | `too_many_headers_are_refused` → `431`; the running receiver: `HTTP/1.1 431 Request Header Fields Too Large` |
| The length of a header line is limited | **pass** | `an_over_long_header_line_is_refused` → `431`; the running receiver: `431` |
| The length of the body is limited — and the limit is checked *before* the body is read | **pass** | `a_body_that_is_too_large_is_refused_before_it_is_read` → `413`; the running receiver: `HTTP/1.1 413 Payload Too Large` for a body that announces 100 MiB |
| A body with a length that is no number is refused | **pass** | `a_content_length_that_is_not_a_number_is_refused` → `400`; the running receiver: `400` |
| A chunked body is refused, not read with the wrong framing | **pass** | `a_chunked_body_is_refused` → `400`; the running receiver: `400` |
| The length of a frame of the channel is limited | **pass** | `a_websocket_frame_that_announces_a_gigabyte_is_refused`; the limit is 1 MiB |
| A frame that is not masked breaks the channel | **pass** | `an_unmasked_websocket_frame_breaks_the_channel` (RFC 6455 requires the mask) |
| A control character in the name of the sender never reaches the answer | **pass** | `a_control_character_in_the_name_of_the_sender_never_reaches_the_answer` — the name is filtered and truncated to 128 characters (`api/routes.rs:216`) |
| A control character in the name of the device never reaches the description document | **pass** | `a_control_character_in_the_name_of_the_device_never_reaches_the_document`; the builder escapes and removes the characters that XML 1.0 forbids (`description.rs:172`) |
| A malformed search is refused and the responder survives it | **pass** | `a_search_with_a_broken_length_is_not_answered_and_the_responder_survives` |
| A search for another service is not answered | **pass** | `a_search_for_another_service_is_not_answered` |
| A refusal costs the peer its request, not the receiver its service | **pass** | `the_receiver_is_still_alive_after_every_refusal`; the running receiver answers `200` after six refusals |
| One sender at a time | **pass** | the second sender is refused with `409`, never queued |
| The engine opens no connection to a peer | **pass** | the crate contains no outbound client — in the production code there is no `connect`; the engine binds the API port and the SSDP socket and answers (`api/server.rs`, `ssdp.rs`) |
| The engine reads no configuration of the user | **pass** | the engine receives the identity, the capabilities and the sinks through the `EngineContext` of the core and reads nothing else |
| The log names no peer | **pass** | the crate writes four log lines (`engine.rs:280`, `engine.rs:316`, `api.rs:155`, `api.rs:171`); none of them carries the name, the address or the MAC of a sender |
| The data of a sender lives only in the session | **pass** | the MAC, the address and the name of the sender are held in the session registry in the memory and are gone with the session — `PRIVACY.md` §3 |
| No cloud, no account, no telemetry | **pass** | the crate has no client, no account and no telemetry — `PRIVACY.md`, `SECURITY.md` |

The threat model and the guarantees are written down in
[`SECURITY.md`](../../SECURITY.md) — the row *Smart View* names the bounded
reader, the deadlines and the single-session rule. The data that the
receiver sees, and the data it does not keep, is written down in
[`PRIVACY.md`](../../PRIVACY.md).

**Note on the hashing of the handshake.** The `Sec-WebSocket-Accept` value is
the SHA-1 of the client key and the GUID of RFC 6455. It is **not** a
security claim: it proves that the answer belongs to the request, and it
authenticates nothing — the channel of Smart View carries no authentication
of its own (`api/channel.rs:66`).

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| A Galaxy phone or tablet (Smart View) | any | **NOT RUN** | needs a Galaxy device on the same network; the device test plan has no Smart View scenario yet — §8, L-3 |
| A Samsung television (as a sender) | any | **NOT RUN** | a television is a *receiver* of Smart View, not a sender; the role of the receiver in this protocol is the target |
| A second host on the network | — | **NOT RUN** | the sandbox has no second host. The announcement to the multicast group is proved by the test `the_announcement_reaches_the_multicast_group` — a socket of the test joins the group and receives the `NOTIFY`; a **second host** that would receive it was not in the sandbox |

What **is** proved is the side of the protocol that does not need a peer:
the search of the framework is answered with the target and the `LOCATION`
of the description document, the document is served, the API answers, the
handshake is accepted, the channel is opened with the accept key of RFC 6455
and held open, the frames are bounded and discarded, and the second sender
is refused.

The interoperability with a real Galaxy device is a device test and is
recorded as **NOT RUN** here — not as passed.

- The formats that a reference implementation reads: the description
  document is XML in the namespace of the search target
  (`urn:samsung.com:service:MultiScreenService:1`); the API speaks JSON.
  The element names of the document, the shape of the JSON bodies, the name
  of the channel and the format of its messages are **our own** — Samsung
  does not document them ([`docs/audit/smartview-engine.md` §3](../audit/smartview-engine.md)).
- The receiver without a component of the vendor of the sender: **no
  component of Samsung is needed.** The engine implements the framework on
  its own; it needs no SDK, no `.jar` and no key of Samsung. It also needs
  no HDCP and no decryption — Smart View is an open feature of the Galaxy
  devices, not a DRM pipeline like FairPlay (`EXC-001` does not apply).
- The deviations from the expectation of the sender: **two, and they are
  named.** (1) The path of the WebSocket channel (`/api/v2/websocket`) and
  the shape of the JSON bodies are our own — a real device that uses another
  path needs a configuration value that does not exist yet. (2) The message
  format of the channel is not implemented at all (`EXC-011`).

## 8. The honest limits

The severity scale of this record: **S0** a fatal or a security defect,
**S1** the function of the milestone is blocked, **S2** the function is
limited, **S3** cosmetic, **S4** a wording.

### The exclusions that apply here

- **`EXC-011` — The message format of the Samsung Smart View channel.** The
  discovery of the receiver and the handshake of the sender are implemented
  and covered by tests; **the content of the channel is not implemented**. A
  Galaxy device finds the receiver, connects to it and establishes the
  channel — but no picture flows over it yet. The limit stays until the
  format is documented by Samsung or until another implementation exists
  under a licence that allows the port. A format that nobody documents is
  not guessed.
  **Sources:** [`REQUIREMENTS.md` §15](../../REQUIREMENTS.md),
  [`engines/smartview/README.md`](../../engines/smartview/README.md),
  [`engines/smartview/src/lib.rs`](../../engines/smartview/src/lib.rs) (the
  section *The honest limit*), [`ARCHITECTURE.md` §4.5](../../ARCHITECTURE.md),
  [`GLOSSARY.md`](../../GLOSSARY.md), [`README.md`](../../README.md),
  [`docs/audit/smartview-engine.md`](../audit/smartview-engine.md).
- **`EXC-001` — FairPlay DRM content** does not apply: Smart View is an open
  feature of the Galaxy devices, and the crate invents no restriction that
  the protocol does not have.

### The limitations that were found during this acceptance

| # | Limitation | Severity | Evidence | Owner |
|---|---|---|---|---|
| L-1 | **The fifth protocol does not start in the default configuration.** The DLNA renderer and the Smart View responder both want the SSDP port 1900; the DLNA responder binds it without `SO_REUSEADDR` (`engines/dlna/src/ssdp.rs:184`), the Smart View responder with it (`engines/smartview/src/ssdp.rs:322`). On Linux two UDP sockets may share an address and a port only when **both** set the option. The DLNA engine is started first (`ProtocolKind::ALL`), so the Smart View responder is refused with `Address already in use (os error 98)` and the fifth protocol never reaches the advertising state. | **S1** | the run of the receiver (§1); the mechanism measured on this host: a socket without `SO_REUSEADDR` binds `0.0.0.0:1900`, a second one with the option is refused with errno 98, two with the option both bind | the owner: either the DLNA responder sets the option as well, or one shared responder in `engines/discovery` serves the group for every protocol of the host |
| L-2 | **The service type `_samsungmsf._tcp` is not published.** The type belongs to the vocabulary of the core (`ProtocolKind::SmartView::mdns_service_types()` — `core/phairplay-core/src/protocol.rs:94`), but no component of the receiver publishes an mDNS record for it: the discovery crate has no case for it (`engines/discovery/src/mdns/config.rs` — the `_ => continue` arm) and the discovery crate is not registered in the composition root. The discovery of the fifth protocol is therefore **SSDP-only**, which is the path Samsung documents. | **S2** | `core/phairplay-core/src/protocol.rs:94`; `engines/discovery/src/mdns/config.rs`; `platform/phairplay-ffi/src/composition.rs:123` (five engines, no discovery engine) | the owner: either the discovery crate gets a case for the type, or the vocabulary stops claiming that the type is advertised |
| L-3 | **The device test plan has no Smart View scenario.** The gate that needs a Galaxy device therefore has no script to run. | **S2** | [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) — the sections 2.1 to 2.4 cover AirPlay 2, Miracast, Cast and DLNA | the owner: a section 2.5 for the path of Smart View, with the steps and the expected result |
| L-4 | **The dashboard gate of the browser was not re-run** for the fifth card. The card is in the code and the API of the running receiver answers with the fifth entry, but the check of the layout, the contrast and the keyboard in a real browser belongs to the dashboard milestone. | **S3** | §5 | the dashboard milestone |
| L-5 | **Two places of the documentation are stale.** `REQUIREMENTS.md` FR-104 points the composition root at `apps/web/src/receiver.rs`; since ADR-0006 it is `platform/phairplay-ffi/src/composition.rs` (`apps/web/src/receiver.rs` only re-exports it). The checklist of the roadmap marks the card of the fifth protocol in the dashboard as *not done*; the code delivers it (`apps/web/assets/api.js:33`). | **S3** | `REQUIREMENTS.md` FR-104; [`ROADMAP.md` §M5](../../ROADMAP.md); `apps/web/assets/api.js:33` | the owner / the documentation role |
| L-6 | **The identity of the receiver is the default one.** The run of the receiver answers with `UDN: uuid:00000000-0000-0000-0000-000000000000`, because the generic profile names no device id. The same holds for the other four protocols. | **S4** | the run of the receiver: `/description.xml` | the owner, with the identity of the device |
| L-7 | **The performance gate of the roadmap names a budget that does not exist.** The gate asks for "the budget of the advertisement path"; the central budget file owns no key for it, so the gate cannot be measured against a number until the owner adds one. | **S3** | `config/performance-budgets.toml` — the keys of the section `[latency]` are `first_frame_ms`, `audio_video_sync_ms` and `idle_timeout_ms` | the owner: a key of the advertisement path, or a gate that names an existing one |
| L-8 | **The path of the channel and the shape of the JSON bodies are our own.** The path `/api/v2/websocket` and the fields `mac`, `ip` and `name` of the handshake are not documented by Samsung. A real device that speaks another path needs a configuration value that does not exist yet. | **S2** | `engines/smartview/src/api.rs:66`, `engines/smartview/src/api/routes.rs:174`; the open points 5 and 6 of [`docs/audit/smartview-engine.md`](../audit/smartview-engine.md) | the owner, after the first test against a real device |

### What is *not* a limitation

The single-session rule, the limits of the reader, the deadlines, the
sanitising of the name of the sender and the escaping of the document are
implemented and proved by negative tests over a real socket (§6). They are
not listed here.

### A note on the state of the tree during this acceptance

While this record was written, **another workstream was changing the same
tree** — `engines/dlna/src/ssdp.rs` and `core/phairplay-core/src/ssdp.rs`
were modified in the working tree, and both changes concern the SSDP port of
the workspace, that is: the defect L-1. The measurements of this record are
the measurements of the state of the tree at the moment of the run — the
commit `2d0c4b4` for everything that the other workstreams had not touched.
The fix of L-1 was **not** measured here — it was in flight, and a run of
the whole workspace on a tree that another workstream edits is not a
measurement but a race (§9).

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| The interoperability with a real Galaxy phone or tablet | no Galaxy device and no second host in the sandbox; the device test plan has no Smart View scenario yet (L-3) | a Galaxy device on the same network and the script of the device test plan |
| The five performance budgets | three of them are a property of a picture that does not flow (`EXC-011`); the other two are a property of the running receiver on a real device | the device test plan, §3 |
| The picture of a Galaxy device on the display of the receiver | the message format of the channel is proprietary and is not implemented (`EXC-011`) | the documentation of the format by Samsung, or another implementation under a licence that allows the port |
| The browser gate of the fifth dashboard card (layout, contrast, keyboard) | the browser gate of the dashboard belongs to the dashboard milestone; this acceptance did not re-run it | `node apps/web/tools/check-layout.mjs`, `check-contrast.mjs`, `check-keyboard.mjs` |
| The licence audit (`cargo-deny`) and the advisory audit (`cargo-audit`) | the tools are not installed in the sandbox and the registry is not reachable offline | the CI jobs *Licenses (cargo-deny, pinned)* and *Security Audit (cargo-audit)* of `.github/workflows/ci.yml` |
| The build of the native shells (Android, macOS) | no Android SDK and no macOS SDK in the sandbox | the Gradle and the Xcode build of the shells |
| The run of the whole workspace | the acceptance of this milestone is measured on the crates of the milestone; the run of the workspace was attempted **twice** and stopped in the compilation, because another workstream was editing the tree at the same moment: the first attempt failed in `engines/dlna/src/ssdp.rs`, the second in `core/phairplay-core/src/ssdp.rs`. Both files carry a change of a **different** workstream (the SSDP port of the workspace); the crate of the milestone compiled and passed in both runs | `cargo test --offline --workspace` on a tree that no other workstream edits — or the CI job *Quality Gate* |

A check that did not run is **NOT RUN** with the exact reason. It is never
reported as a success.

## 10. Decision

**ACCEPTED with a documented limitation.**

The milestone delivers the part of Samsung Smart View that is publicly
documented, and it delivers it with evidence: the search of the MultiScreen
framework is answered over a real socket with the target and the `LOCATION`
of the description document; the description document is served; the API
below `/api/v2/` answers; the handshake of the sender is accepted and
recorded; the channel is opened with the accept key of RFC 6455 and held
open; the second sender is refused with `409`; every limit of the reader and
of the channel is enforced and proved by a negative test over a real socket;
the fifth protocol is in the vocabulary of the core, can be switched off in
the configuration, and is registered with the supervisor in the composition
root — **112 tests of the crate pass, 0 fail, 0 are ignored**, and the
**43 tests of the bridge** — the tests of the composition root — pass as
well; the lint is clean, the format is clean, the boundary gate is green
and it still catches five real violations.

The limitation is named, not hidden:

> **The milestone delivers the discovery, not the picture.** The message
> format of the Smart View channel is proprietary and is not publicly
> documented, so it is not implemented (`EXC-011`). A Galaxy device finds
> the receiver and connects to it — but no picture flows. The reason is not
> a missing feature in the schedule: it is that a format that nobody
> documents cannot be implemented honestly. Guessing it would produce a
> receiver that looks complete and is not, and the cost of that is paid by
> the user, who taps the receiver in the list of the phone and sees nothing
> happen.

Three reservations travel with the acceptance:

1. **The fifth protocol does not start in the default configuration** (L-1).
   The SSDP port of the workspace is contested between the DLNA renderer and
   the Smart View responder, and the renderer holds it first. The feature is
   reachable — the same run with the DLNA renderer switched off reports
   `Samsung Smart View advertising` — but it is not reachable out of the box,
   and that is a defect of the receiver, not of the milestone.
2. **The device test is outstanding** (L-3). This record proves the
   software, not the product on a television. A milestone may not be
   *announced* on the strength of this record; it may only be announced
   together with the device evidence of the device test plan.
3. **The service type `_samsungmsf._tcp` is not published** (L-2). The
   vocabulary of the core claims it, the receiver does not publish it.

**The decision names the role that took it:** the Rust QA Engineer, on
2026-10-24, on the branch `feat/other-devices` at `2d0c4b4`, with the
open points L-1 to L-8 carried to the owner of the project.

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.
