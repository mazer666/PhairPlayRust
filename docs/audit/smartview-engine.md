# The Samsung Smart View engine — audit report

> **Status:** the discovery and the handshake are implemented and green.
> The message format of the Smart View channel is **not** implemented and is
> not claimed.
>
> **Crate:** `phairplay-smartview` (`engines/smartview`), branch `feat/other-devices`.
> **Contract:** `docs/design/BUILD-CONTRACT.md`.
> **Date of the report:** 2026-10-24.

---

## 1. What was built

The crate implements the part of Samsung Smart View that is publicly
documented: the **discovery** of a Smart View target and the **handshake**
that follows it.

| Step of the protocol | Where it is implemented | State |
|---|---|---|
| The SSDP announcement (`M-SEARCH` answer, `NOTIFY` alive, `NOTIFY` byebye) | `src/ssdp.rs` | done |
| The description document behind the `LOCATION` header | `src/description.rs` | done |
| The HTTP API below `/api/v2/` (root, resource description, base URL, handshake of the sender) | `src/api/routes.rs` | done |
| The WebSocket handshake (RFC 6455) and the held-open channel | `src/api/channel.rs` | done |
| The state machine of the engine (`Idle → Starting → Advertising → Streaming → Stopping → Stopped`) | `src/engine.rs`, `src/lifecycle.rs` | done |
| The single-session rule (one sender at a time) | `src/session.rs` | done |
| **The content of the channel** | — | **not implemented, not claimed** |

The wording that is used in the code, in the crate documentation and in
this report:

> The discovery and the handshake are implemented. The message format of
> the channel is proprietary and not publicly documented; it is not
> implemented. A Galaxy device finds the device and connects to it, but no
> picture flows yet.

Smart View needs **no** HDCP protection and **no** decryption — it is an
open Samsung feature on Galaxy devices, not a DRM pipeline like FairPlay.
Nothing in the crate invents a restriction that the protocol does not have.

---

## 2. The file list

| Path | Lines | Purpose |
|---|---:|---|
| `engines/smartview/Cargo.toml` | 48 | The manifest. The dependencies and why each of them is already in the tree of the workspace. |
| `engines/smartview/README.md` | 23 | The role of the crate and the honest limit, in the wording of the documentation contract. |
| `engines/smartview/src/lib.rs` | 91 | The crate root: WHAT / WHY / HOW, the table of the modules, the re-exports. |
| `engines/smartview/src/error.rs` | 110 | The errors of the crate and the distinction between a broken receiver and a misbehaving peer. |
| `engines/smartview/src/lifecycle.rs` | 113 | The public state of the engine: the state, the ports, the reason of the last failure. |
| `engines/smartview/src/lifecycle/tests.rs` | 66 | The tests of every accessor of the lifecycle. |
| `engines/smartview/src/session.rs` | 175 | The single-session registry: the peer that announced itself and the channel that is open. |
| `engines/smartview/src/session/tests.rs` | 115 | The tests of the registry, including the rejection of a second peer. |
| `engines/smartview/src/ssdp.rs` | 339 | The announcement: the codec, the responder, the socket with `SO_REUSEADDR`. |
| `engines/smartview/src/ssdp/tests.rs` | 170 | The tests of the codec, including the refusal of a malformed search. |
| `engines/smartview/src/description.rs` | 209 | The description document as a string builder, with the sanitising and the escaping. |
| `engines/smartview/src/description/tests.rs` | 151 | The tests of the builder, including a control character and a markup character in the name. |
| `engines/smartview/src/api.rs` | 213 | The module root of the API: the context, the connection handler, the answer writer. |
| `engines/smartview/src/api/types.rs` | 216 | The vocabulary of the API: the limits, the method, the request, the answer. |
| `engines/smartview/src/api/reader.rs` | 236 | The bounded HTTP reader: every read has a limit and a deadline. |
| `engines/smartview/src/api/routes.rs` | 286 | The handlers of the paths and the routing of a request. |
| `engines/smartview/src/api/channel.rs` | 236 | The WebSocket handshake of RFC 6455 and the reading of the frames. |
| `engines/smartview/src/api/server.rs` | 125 | The accept loop and the release of the socket. |
| `engines/smartview/src/api/tests.rs` | 292 | The tests of the routes, of the handshake and of the limits. |
| `engines/smartview/src/engine.rs` | 344 | The `ProtocolEngine` implementation and the configuration of the engine. |
| `engines/smartview/src/engine/tests.rs` | 233 | The tests of the lifecycle, including the status from another thread. |
| `engines/smartview/tests/common/mod.rs` | 159 | The helpers of the end-to-end tests: the engine, the raw HTTP client, the WebSocket frames. |
| `engines/smartview/tests/discovery.rs` | 191 | The end-to-end tests of the discovery over a real UDP socket. |
| `engines/smartview/tests/control_plane.rs` | 143 | The end-to-end tests of the API over a real TCP socket. |
| `engines/smartview/tests/hardening.rs` | 225 | The negative tests: every hardening measure is exercised over a real socket. |

**The dependency on the core.** The engine implements
`phairplay_core::protocol::ProtocolEngine`, and that trait returns a
`ProtocolKind`. The core therefore needs a fifth variant,
`ProtocolKind::SmartView`, with the arms that belong to it
(`as_str`, `display_name`, `mdns_service_types`, `carries_video`, the
`ALL` constant and the aliases of `FromStr`), the port
`ports::SMARTVIEW_API` (8001) and the service type
`service_types::SAMSUNG_MSF_TCP` (`_samsungmsf._tcp`). These additions are
in the tree — the ports, the service type and the mapping of the bridge
(`platform/phairplay-ffi/src/dto/protocol.rs`) come from the work on the
core and on the bridge that runs in parallel; **the two tests of the core
that this crate needs were added here**
(`a_smart_view_receiver_advertises_the_multi_screen_service` and the
`SmartView` arm of `only_streaming_protocols_carry_video`), because the
build contract asks for it: *"If a crate needs something that is not here,
add it to `phairplay-core` first, in its own commit, and extend its test
suite."*

The core is green with them:

```
$ CARGO_TARGET_DIR=/workspace/target-smartview cargo test --offline -p phairplay-core

running 94 tests
test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

**The change of the core needs a change request.** A fifth variant of
`ProtocolKind` is a change of the shared vocabulary, and the build
contract asks for a change request in `docs/cr/` for it — it has not been
written, because writing it is the job of the architect role and this
report may not commit anything. See §7.

---

## 3. The proven facts and their sources

| Fact | Source | Where it is used |
|---|---|---|
| The search target is `ST: urn:samsung.com:service:MultiScreenService:1`. | Samsung, *Device Discovery, Authentication, and Pairing* (Smart TV Legacy documentation, `d3unf4s5rp9dfh.cloudfront.net/SmartTV_Legacy/convergence/convergence/Device+Discovery,+Authentication,+and+Pairing.pdf`). | `ssdp::SEARCH_TARGET`, the answer of the `M-SEARCH`, the `NT` of the announcement. |
| A device that offers the service answers `HTTP/1.1 200 OK` with a `LOCATION` header. | The same document. | `ssdp::search_response`, the `LOCATION` of the announcement. |
| The sender fetches the description XML at that location and reads the display name out of it. | The same document. | `description::description_document`, the path `/description.xml`. |
| The sender then posts its MAC address, its IP address and its device name to the device. | The same document. | `api::routes::handshake` — the fields `mac`, `ip`, `name`. |
| The API root is `http://<IP>:8001/api/v2/`. | Samsung's own debug instructions for the SmartView SDK server; the port is also `ports::SMARTVIEW_API` in the core. | `api::API_ROOT_PATH`, `engine::DEFAULT_API_PORT`. |
| The DNS-SD service type is `_samsungmsf._tcp`. | The service library `wac-discovery` (`SAMSUNG_SMART_VIEW`); also `config::service_types::SAMSUNG_MSF_TCP` in the core. | The core table of the protocols — **not** the announcement of this crate (see §7). |
| The sender opens a channel with `service.channel('CHANNEL_NAME')`. | The SmartView SDK of the sender side (`window.msf.local`). | The WebSocket upgrade — the name of the channel is proprietary and is not used. |
| The message format of the channel is proprietary. | The SmartView SDK is a closed framework of `.jar` libraries; no public specification of the channel format exists. | The reason why the content of the channel is **not** implemented. |
| The WebSocket handshake answers with `Sec-WebSocket-Accept = base64(sha1(key + GUID))`. | RFC 6455, section 4.2.2 and the example of section 1.3. | `api::channel::accept_key` — the example of the RFC is a test. |

**What is *not* proven and was therefore not implemented:** the names of
the elements of the description document, the shape of the JSON bodies, the
name of the channel and the format of its messages. The crate documents its
own shapes and says so.

---

## 4. The finished/open matrix

### 4.1 Finished

| Item | Evidence |
|---|---|
| The `M-SEARCH` for the MultiScreen service is answered over a real socket with the `LOCATION`, the `ST` and the `USN`. | `tests/discovery.rs::a_search_for_the_multi_screen_service_is_answered_over_the_network` |
| The announcement reaches the multicast group and the port the responder listens on. | `tests/discovery.rs::the_announcement_reaches_the_multicast_group` |
| The description document is served at the path of the `LOCATION` header. | `tests/control_plane.rs::the_description_document_is_served_at_the_location` |
| The API root, the resource description and the base URL answer. | `tests/control_plane.rs::the_api_root_answers_with_the_base_url`, `src/api/tests.rs` |
| The handshake of the sender is accepted and the announced peer is recorded. | `tests/control_plane.rs::the_handshake_of_the_sender_is_accepted` |
| The WebSocket upgrade is answered with `101` and the accept key of the RFC. | `tests/control_plane.rs::the_websocket_upgrade_is_answered_with_the_accept_key` |
| The channel is held open; the engine counts the sender and reports `streaming`. | `tests/control_plane.rs::the_channel_is_held_open_and_the_sender_is_counted` |
| A second sender is **rejected** with `409`, not queued. | `tests/control_plane.rs::a_second_sender_is_refused_while_the_channel_is_open`, `src/session/tests.rs::a_second_peer_is_rejected_and_not_queued` |
| The engine implements the state machine of the contract; the status is readable from another thread during a transition. | `src/engine/tests.rs::the_status_is_readable_while_the_engine_starts` |
| The engine can be stopped and started again; the port is free afterwards. | `src/engine/tests.rs::the_engine_stops_and_frees_the_ports`, `the_engine_can_be_started_again_after_it_was_stopped` |
| Every hardening measure has a negative test that executes the attack over a real socket. | `tests/hardening.rs` (12 tests) |

### 4.2 Open — and why

| Item | Why it is open |
|---|---|
| **The content of the channel** (the picture) | The format is proprietary and not publicly documented. Implementing it would mean reverse engineering a closed SDK — out of scope and dishonest to claim. |
| **The authentication and the pairing of Samsung** | The document that describes them ("Device Discovery, Authentication, **and Pairing**") documents the discovery; the pairing itself is part of the closed SDK. The crate implements the discovery and the handshake, not the pairing. |
| **The mDNS advertisement of `_samsungmsf._tcp`** | The advertisement belongs to `engines/discovery`. Its `DiscoveryConfig::specs()` has no case for `SAMSUNG_MSF_TCP` and falls through to `continue`, so **no `_samsungmsf._tcp` record is published today**. The core already lists the type under `ProtocolKind::SmartView::mdns_service_types()`. See §7. |
| **The wiring into `apps/web`** | The web backend is the composition root of the receiver. It does not build the SmartView engine yet, so the engine is not started by the dashboard. See §7. |
| **The port 1900 next to the DLNA engine** | Both engines announce on the SSDP group. See §5.3 and §7. |

---

## 5. The decisions

### 5.1 The HTTP part is written with `tokio`, not with `axum`

**Decision:** the API is a hand-written HTTP/1.1 server on top of `tokio`
(`src/api/reader.rs`, `src/api/types.rs`, `src/api/server.rs`).

**Why:**

1. *An engine crate must not drag a web stack into every binary of the
   workspace.* The UniFFI library and the native shells (Android, macOS)
   link the engines. `axum` pulls `hyper`, `tower`, `http` and
   `tokio-tungstenite` into the dependency graph of the library — a
   framework that the receiver of a Raspberry Pi would carry for a handful
   of routes.
2. *The house style already has the pattern.* The DLNA engine
   (`engines/dlna/src/http.rs`, `engines/dlna/src/server.rs`) has a bounded
   reader and an accept loop of a few hundred lines. A second framework in
   the same workspace would be the inconsistency.
3. *The hardening is easier to prove with an own reader.* The reader of
   this crate refuses a request line of 8 KiB, a header block of 512 KiB, a
   body of 64 KiB and a frame of 1 MiB — and each of the four limits has a
   negative test that executes the attack over a real socket.

**What `axum` would have given:** the router, the tower middleware and the
WebSocket implementation of the `ws` feature. The router is not needed
(seven paths), the middleware is not needed (the limits are in the reader)
and the WebSocket implementation is not needed (the channel content is not
implemented — only the handshake and the frame bounds are).

**The dependencies that were added** — all of them were already in the tree
of the workspace, so no new third-party code enters the build:

| Dependency | Already in the workspace through | Why it is needed |
|---|---|---|
| `serde_json` | the dashboard, the bridge | The JSON bodies of the API. |
| `base64` | the pairing material of the AirPlay engine | The `Sec-WebSocket-Accept` header. |
| `sha1` (0.10) | the `rsa` crate (its `sha1` feature) | The SHA-1 of the handshake nonce that RFC 6455 prescribes. It is **not** a security claim — the value only proves that the answer belongs to the request. |
| `socket2` (0.6) | the Tokio runtime | `SO_REUSEADDR` on the multicast socket. The standard library cannot set it, and `unsafe` is denied workspace-wide. |

### 5.2 The description document is a string builder

The document is a small, static file. It is built by a string builder and
not by an XML tree, and every text that goes into it is sanitised (the
control characters that XML 1.0 forbids are removed) and escaped. The
negative test `a_control_character_in_the_name_never_reaches_the_document`
runs the attack over a real socket
(`tests/hardening.rs::a_control_character_in_the_name_of_the_device_never_reaches_the_document`).

### 5.3 The announcement goes to the port the responder listens on

The responder binds the SSDP port and announces on the group
`239.255.255.250` **and on the port it is bound to**. In the default
configuration of the engine that is the well-known port 1900 — the port a
sender searches (`ports::SSDP`). The pair "the port I listen on" and "the
port I announce on" can therefore never disagree.

The socket is opened with `SO_REUSEADDR`, so that a second responder on the
same host does not fail with "address already in use" when the first one
also sets the option.

### 5.4 The channel is read, counted and discarded

The frames of the channel are read with a limit and a deadline, a ping is
answered with a pong, the close of the peer ends the session — and the
payload is **not interpreted**. `api::channel::hold_channel` says so in its
documentation. A frame that announces more than 1 MiB and a frame that is
not masked break the channel; both have a negative test.

---

## 6. The test output

```
$ CARGO_TARGET_DIR=/workspace/target-smartview cargo test --offline -p phairplay-smartview

running 75 tests
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.33s

running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s

running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.08s

running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

The five blocks are the unit tests, the end-to-end tests of the control
plane, the end-to-end tests of the discovery, the negative tests and the
doctests — **112 tests in total**. The complete output is in
[`smartview-engine-test.log`](smartview-engine-test.log).

```
$ CARGO_TARGET_DIR=/workspace/target-smartview cargo clippy --offline -p phairplay-smartview --all-targets -- -D warnings

    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.67s
```

The complete output is in [`smartview-engine-clippy.log`](smartview-engine-clippy.log).

The core, which the crate depends on, is green as well:

```
$ CARGO_TARGET_DIR=/workspace/target-smartview cargo test --offline -p phairplay-core

running 94 tests
test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

The documentation gate of the repository reports **0 broken internal links**
(464 links checked over 57 documents) and one **budget drift** that is not
part of this change: the value of `max_pairing_attempts` is written down in
`README.md` and `ROADMAP.md` as well. Neither file was touched by this
task; the drift belongs to the change that is in the tree.

**NOT RUN — the interoperability with a real Galaxy device.** No Galaxy
phone and no Samsung television were available in the sandbox. The tests
prove the protocol against the *documented* behaviour (the search target,
the `LOCATION`, the API root, the handshake, RFC 6455); they cannot prove
that a specific firmware accepts them.

**NOT RUN — the receiver as a service on a real device.** No Raspberry Pi,
no NAS and no television were available in the sandbox.

---

## 7. The open points that need an owner decision

1. **The change of the core needs a change request.** The engine cannot
   exist without the fifth variant `ProtocolKind::SmartView` in
   `core/phairplay-core/src/protocol.rs`, with its port
   (`ports::SMARTVIEW_API`), its service type
   (`service_types::SAMSUNG_MSF_TCP`) and its mapping in the bridge
   (`FfiProtocolKind::SmartView`). These additions are in the tree; the
   build contract asks for a change request in `docs/cr/` for a change of
   the shared vocabulary — it has not been written, because writing it is
   the job of the architect role and this report may not commit anything.
   *Decision needed:* the owner confirms the variant (and its arms) with a
   change request, or the engine has to be taken out of the tree again.

2. **The SSDP port 1900 is contested on a host that runs DLNA and Smart
   View.** The DLNA engine binds `0.0.0.0:1900` **without**
   `SO_REUSEADDR`; the Smart View responder binds it **with**
   `SO_REUSEADDR`. On Linux, two UDP sockets may share an address and a
   port only when *both* set the option — so on a host that runs both
   engines, the second one fails with `EADDRINUSE` and the engine reports
   `the SSDP responder could not be started: …`.
   *Decision needed:* either the DLNA responder also sets `SO_REUSEADDR`
   (one line in `engines/dlna/src/ssdp.rs`, outside the scope of this
   task), or the SSDP group is served by one shared responder in
   `engines/discovery` that answers for every protocol of the host. The
   second solution is the architecturally correct one — but it needs a
   change request, because the boundary rule of this task says that the
   SmartView engine owns its announcement.

3. **The `_samsungmsf._tcp` advertisement is missing.**
   `engines/discovery/src/mdns/config.rs` has no case for
   `SAMSUNG_MSF_TCP`; the specification falls through to `continue`, so no
   mDNS record for the MultiScreen service is published. A Galaxy device
   that finds the receiver by mDNS will not find it.
   *Decision needed:* the discovery crate gets a case for the type (with
   the port 8001 and a TXT record), or the product accepts that Smart View
   is discovered by SSDP only.

4. **The engine is not wired into the composition root.** `apps/web` does
   not build the SmartView engine yet, so the dashboard does not start it
   and does not show it in the protocol list.
   *Decision needed:* the web backend (the role of the platform bridge)
   adds the engine to the supervisor, and the FFI bridge exposes it in the
   protocol list of the native shells.

5. **The path of the WebSocket channel is our own.** The real path of the
   SmartView channel is part of the proprietary format. The crate uses
   `/api/v2/websocket` and documents that the name is our own. If a real
   Galaxy device uses another path, the path becomes a configuration value
   — one line in the configuration, one field in the engine.
   *Decision needed:* the owner decides whether the path is configurable
   before the first interoperability test.

6. **The wire format of the handshake is our own.** The crate accepts a
   JSON object with the fields `mac`, `ip` and `name`. The field names are
   the documented ones, the format is not documented.
   *Decision needed:* after the first test against a real device, either
   the format is confirmed or a second format is accepted.

---

## 8. The limits of the product, in the words of the product

- **No picture yet.** The channel is opened and held; its content is not
  interpreted. This is the proprietary part of Smart View.
- **No HDCP and no decryption.** Smart View is an open Samsung feature on
  Galaxy devices. The crate does not invent a restriction that the
  protocol does not have.
- **No cloud, no account, no telemetry.** The crate answers on the local
  network only.
- **One sender at a time.** A second Galaxy device is refused with `409`,
  never queued.
- **The receiver is a target, not a controller.** The crate does not
  search for televisions and does not send anything to them.
