# Security Policy

**Applies to:** PhairPlayRust 0.1.0
**Last updated:** 2026-10-24

PhairPlayRust is a receiver that sits on a home network and accepts media
streams from devices on that network. That makes it a *network-facing*
application: the person who attacks it does not need an account, a file or a
malformed document — they need a device that can reach the television on the
same Wi-Fi. This document says what the threat model is, what the receiver
does about it, and how to report a problem.

---

## 1. The threat model in one sentence

**Every device on the local network is untrusted, because pairing is off by
default.**

The receiver does not ask for a PIN out of the box, so anyone in range can
send it a stream. Every byte that arrives over the network is therefore
treated as hostile input until a parser has proved otherwise.

| Surface | What arrives | What the receiver does |
|---|---|---|
| RTSP control plane | Request line, headers, bodies | Bounded reader: message size, header count, header line length, idle timeout |
| RTSP media plane | Interleaved RTP frames | Bounded queue with an item count **and** a byte budget |
| SDP | Session description, codec parameters | Parsed, never repaired; malformed input is refused |
| Property lists | XML and binary `bplist00` | Depth, object count and reference cycles are bounded |
| H.264 | SPS, PPS, NAL units | Exp-Golomb codes are read into `u64` and clamped — a 31-bit run of zeroes cannot overflow |
| AAC / ALAC | ADTS headers, frame payloads | Frame headers are length-checked; the Golomb coder is width-clamped |
| SSDP | `M-SEARCH` requests, `NOTIFY` | Answered on the local segment only; the response size is bounded |
| SOAP / GENA | XML bodies, callback URLs | `DOCTYPE` refused, external entities off, recursion depth bounded, callback URLs restricted to private IPv4 of the local subnet |
| CastV2 | Length-prefixed frames | A frame that announces more than the limit is refused before a byte is read |
| WFD | Information elements | Block lengths are checked against the buffer; a block that runs past the end is refused |
| Smart View | SSDP searches, HTTP requests, WebSocket frames | Bounded reader: request line, header count, header length, body size, frame size; every read has a deadline; one sender at a time |

The rules that implement this live in
[`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) §3.3 and §3.4
and are enforced by the workspace lints: `unsafe_code` is denied, `unwrap()`
and `expect()` are denied, and every public function is expected to have a
test.

## 2. Hardening guarantees

These are the guarantees the project makes. Each of them has a test that
fails without it.

- **Bounded memory.** No queue is bounded by an item count alone. Every queue
  has a byte budget as well, and the budget is a configured value
  (`config/performance-budgets.toml`, section `[queues]`) — a peer that sends
  maximum-size frames cannot make the receiver allocate an unbounded amount
  of memory. When a budget is reached, the *oldest* frame is dropped, not the
  newest: a picture that is one frame old is still worth showing, a picture
  that never arrives is not.
- **Bounded time.** Every blocking read carries a deadline
  (`latency.idle_timeout_ms`). A peer that connects and then says nothing
  does not hold a connection slot forever.
- **Bounded parsing.** No parser allocates a buffer whose size comes from a
  length field in the input without checking it first. The property-list
  decoder refuses a cyclic object graph; the information-element parser
  refuses a block that runs past the end of the buffer; the SPS parser
  refuses a channel width that does not exist.
- **One peer at a time.** AirPlay, Miracast, DLNA and Samsung Smart View
  serve one sender; a second connection is refused — with `503 Service
  Unavailable` for the streaming protocols and with `409 Conflict` for the
  Smart View channel. Google Cast is the documented exception — a Cast
  receiver serves every sender that connects.
- **The Smart View channel is read, not interpreted.** The message format of
  the channel is proprietary. The receiver reads the frames with a limit and
  a deadline and discards their content, so a frame that announces more
  than the limit and a frame that is not masked both break the channel
  instead of being read into memory.
- **No remote reach.** The GENA callback guard accepts only private IPv4
  addresses of the local subnet. `169.254.169.254`, `::1`, link-local
  addresses and IPv6 are refused: a renderer that calls back to a link-local
  address is a server-side request forgery.
- **No automatic code execution.** A property list, an SDP document or a
  SOAP body is *data*. Nothing in the receiver interprets it as a program, and
  nothing in the receiver follows a reference that arrives over the network
  outside the local subnet.
- **Loopback-only control panel.** The dashboard binds to the loopback
  interface and nowhere else. It is a local control panel, not a service on
  the network.
- **Copy protection stays copy protection.** The receiver announces
  `wfd_content_protection: none` and does not attempt to bypass HDCP. FairPlay
  DRM content stays encrypted — no part of this project attempts to decrypt it.

## 3. What the receiver deliberately does not do

- It does not collect telemetry, and there is no code path that sends data to
  a server. See [`PRIVACY.md`](PRIVACY.md).
- It does not phone home to check for updates, to count installations, or to
  report errors.
- It does not open a cloud relay for a stream that the local network can
  carry.
- It does not attempt to circumvent FairPlay DRM, HDCP, Widevine or
  PlayReady.

## 4. Known limitations

| Limitation | Why | What a user can do |
|---|---|---|
| PIN pairing is off by default, so any device on the network can start a stream | A receiver that asks for a PIN on every connection is a receiver people switch off | Turn on `protocols.<name>.require_pin` in `config/app-defaults.toml` |
| Miracast needs a Wi-Fi Direct group, which the platform layer has to form | Wi-Fi Direct is a system API of the host operating system | Use AirPlay 2 or DLNA, which work over the normal network |
| AirPlay 2 media playback needs a decoder of the host device | Decoding is a platform concern | The receiver advertises only the codecs the device really has |
| FairPlay DRM content cannot be played | Apple encrypts it and the keys are not available to a receiver | Play unprotected content, or use the official Apple receiver |

## 5. Reporting a vulnerability

Please **do not** open a public issue for a security problem.

Use GitHub's [private vulnerability reporting](https://github.com/mazer666/PhairPlayRust/security/advisories/new)
on this repository instead.

Please include:

- the affected component (`core/`, `engines/<name>/`, `formats/<name>/`,
  `platform/`) and the version or commit,
- the smallest input that reproduces the problem — a capture, a hex dump or
  a short script is ideal, and a peer on the local network can be a script,
- what the attacker gains: a crash, an out-of-bounds read, an allocation the
  peer controls, a callback outside the local subnet, or information from
  another peer,
- whether the problem is reachable **without** pairing.

We answer a report within seven days, and we will credit the reporter in the
advisory unless you would rather stay anonymous.

## 6. Supported versions

| Version | Supported |
|---|---|
| 0.1.x | yes |

PhairPlayRust is pre-1.0. Until 1.0, security fixes land on the `main`
branch and in the next release.

## 7. How the security of the project is checked

- Every hardening measure in §2 has a test that **executes the attack** and
  fails without the fix. A test that only asserts that a function returns
  an error is not accepted as evidence.
- The review of a change is adversarial by design: the reviewer tries to
  break the change before it is accepted.
- `deny.toml` and the licence gate make sure that no dependency with an
  incompatible licence enters the tree, and that the dependency set is
  audited for advisories.
