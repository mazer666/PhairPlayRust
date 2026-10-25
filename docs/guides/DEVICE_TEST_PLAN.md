# The device test plan of PhairPlayRust

> **Status:** binding. This document names every acceptance criterion that
> cannot be proved by a test on a build machine, and it gives the exact
> script for each of them.
>
> **WHY:** the unit tests and the integration tests of the workspace prove
> that the parsers reject, that the queues are bounded and that the engines
> keep their state. They do **not** prove that an iPhone finds the
> receiver in the device picker, that a Windows computer shows it under
> "Cast to Device", or that a Chromecast can drive the control plane. Those
> are facts about the world, not about the code, and only a real sender can
> prove them.
>
> **THE HONEST RULE:** a criterion that is listed here is **NOT RUN** until
> the script below was carried out and the result was written into
> [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md). The
> continuous integration never claims to have proved any of them.

---

## 1. What the plan needs

| Item | Value |
|---|---|
| The receiver | The build of the day, on a machine of the local network |
| The dashboard | `cargo run -p phairplay-web` — it binds the loopback interface only |
| The network | One subnet, `192.168.1.0/24` in the examples below |
| The reference device | Named in every row of the tables — a name, not a class |

The address of the receiver is the address of the machine on which it runs.
The examples use `192.168.1.5`; **replace it by the real address** in
every command.

```text
# The address of the receiver, once per session:
export RECEIVER=192.168.1.5
```

The state of the receiver is read from the dashboard:

```text
# The protocols and their states:
curl -s "http://127.0.0.1:8080/api/status" | python3 -m json.tool

# The live sessions:
curl -s "http://127.0.0.1:8080/api/sessions" | python3 -m json.tool

# The counters of the protocols:
curl -s "http://127.0.0.1:8080/api/metrics" | python3 -m json.tool
```

The logs of the receiver are read with the logger of the bridge. The
sender name and the address of the peer are masked — that is the promise
of the privacy policy, and a log line that carries a name is a defect.

---

## 2. The criteria that need a sender

### 2.1 The AirPlay 2 path — a macOS or an iOS sender

| Requirement | What has to be proven | Sender |
|---|---|---|
| FR-008 | The receiver advertises `_airplay._tcp` **and** `_raop._tcp`. | macOS 14 / iOS 17 |
| FR-010 | The instance name of the service carries no control character. | macOS 14 / iOS 17 |
| FR-011 | A protocol can be switched off and the sender no longer finds it. | macOS 14 |
| FR-012 | The advertisement comes back after the Wi-Fi was switched off and on. | macOS 14 |
| FR-017 | The RTSP control plane is answered by a real sender. | macOS 14 / iOS 17 |
| FR-018 | The routes of the receiver answer a real sender. | macOS 14 |
| FR-019 | The SDP of a real session is parsed. | macOS 14 / iOS 17 |
| FR-020 | The RTP packets of a real session are reassembled into frames. | macOS 14 / iOS 17 |
| FR-021 | A second AirPlay peer is **rejected**, not queued. | two devices |
| FR-022 | The identity of the receiver is the configured one. | macOS 14 |
| FR-023 | The pairing with the HomeKit-style procedure works. | iOS 17 |
| FR-024 | The FairPlay session key is decrypted and the picture appears. | macOS 14 / iOS 17 |
| FR-025 | A FairPlay-DRM stream is refused with a clean error. | Apple TV app |
| FR-026 | The metadata and the artwork of the sender arrive. | macOS 14 / iOS 17 |
| FR-027 | The reverse remote (the volume of the sender) is answered. | macOS 14 |
| FR-028 | The audio and the video are in sync. | macOS 14 |
| FR-029 | A photo is received and shown. | iOS 17 |
| FR-030 | The audio-only session works without a picture. | macOS 14 (Music) |
| FR-031 | The session is torn down cleanly when the sender stops. | macOS 14 |
| NFR-PERF-001 | The time to the first picture is inside the budget. | macOS 14 |
| NFR-PERF-002 | The frame rate holds over a session. | macOS 14 |
| NFR-PERF-003 | The resident memory stays inside the budget. | macOS 14 |
| NFR-PERF-004 | The CPU usage stays inside the budget. | macOS 14 |
| NFR-PERF-005 | The audio and the video do not drift apart. | macOS 14 |

#### The script

**Preparation (on the receiver):**

```text
# 1. The receiver runs with the default configuration.
cargo run -p phairplay-web

# 2. The services are advertised. The names of the services must be
#    visible on the network.
dns-sd -B _airplay._tcp local.      # in another terminal, on the receiver
dns-sd -B _raop._tcp local.
```

**Step 1 — the sender finds the receiver.**

```text
# On the Mac: the screen mirroring icon of the menu bar
#   -> the device picker shows the name of the receiver.
# On the iPhone: the control centre -> screen mirroring -> the receiver.
```

**Expected:** the receiver appears in the picker under the name of
`config/app-defaults.toml` (`[device] name`).

**Step 2 — the picture arrives.**

```text
# On the Mac: choose a video in QuickTime or in a browser and play it.
# The picture must appear on the screen of the receiver.
```

**Expected:** the picture appears. The time between the click and the
picture is measured with a stop watch and written into the acceptance
record; the budget is `latency.first_frame_ms` of
`config/performance-budgets.toml`.

**Step 3 — the second peer is refused.**

```text
# With the first sender connected, open the picker on the second device
# and choose the receiver.
```

**Expected:** the second device shows an error, and the first session keeps
running. The state of the receiver stays `streaming` with **one** peer:

```text
curl -s "http://127.0.0.1:8080/api/sessions" | python3 -m json.tool
# -> exactly one session
```

**Step 4 — the metadata and the artwork arrive.**

```text
# Play a track in the music player of the sender. The title, the artist
# and the cover must appear in the dashboard.
```

**Expected:** the session of the dashboard carries the title and the
artist, and the event `MetadataChanged` was sent.

**Step 5 — the session is torn down.**

```text
# Stop the mirroring on the sender.
curl -s "http://127.0.0.1:8080/api/sessions" | python3 -m json.tool
# -> no session
```

**Expected:** the session is gone, and the reason of the end is one of the
documented reasons (`sender disconnected`).

**Step 6 — the advertisement comes back.**

```text
# On the receiver: switch the Wi-Fi off for ten seconds and on again.
# The service must be advertised again without a restart of the receiver.
```

**Expected:** after the Wi-Fi is back, the sender finds the receiver again
within a few seconds.

**Step 7 — the FairPlay-DRM stream is refused.**

```text
# Open the Apple TV app on the sender and play a title that carries DRM.
```

**Expected:** the receiver refuses the session with a clean error and
**does not** show a picture. The boundary is `EXC-001`; the receiver does
not pretend to decrypt it.

**Step 8 — the pairing.**

```text
# Set "require_pin = true" for the protocol airplay in
# config/app-defaults.toml, restart the receiver, and connect with the
# sender. The sender must ask for the PIN.
```

**Expected:** the sender asks for the PIN, the PIN is accepted, and the
picture arrives. A wrong PIN is refused, and the number of failed attempts
is counted (`limits.max_pairing_attempts`).

**Step 9 — the photo.**

```text
# On the iPhone: Photos -> share -> AirPlay -> the receiver.
```

**Expected:** the photo appears on the receiver.

**Step 10 — the reverse remote.**

```text
# On the Mac, while the mirroring runs: change the volume of the system.
```

**Expected:** the volume of the receiver follows.

**Step 11 — the memory and the CPU.**

```text
# While the mirroring runs, on the receiver:
ps -o pid,rss,pcpu,comm -C phairplay-web
# after the session of one minute:
```

**Expected:** the resident memory stays below `resources.max_memory_mb`
and the CPU usage stays below `resources.max_cpu_percent`. The values are
written into the acceptance record with the command that produced them.

---

### 2.2 The Miracast path — a Windows 11 computer or a Miracast sender

| Requirement | What has to be proven | Sender |
|---|---|---|
| FR-032 | The Miracast engine is a protocol engine that starts and stops. | Windows 11 |
| FR-033 | The WFD control plane is answered. | Windows 11 "Connect" |
| FR-034 | The advertised codecs are the ones the sender uses. | Windows 11 |
| FR-035 | The permission of the system is needed for the WFD sink. | Windows 11 |
| FR-036 | A second Miracast peer is rejected. | two Windows computers |

#### The script

**Step 1 — the discovery.**

```text
# On the Windows computer: Settings -> System -> Display ->
#   "Wireless display" / "Project" -> "Add a wireless display".
# The receiver must appear in the list.
```

**Expected:** the receiver appears.

**Step 2 — the picture.**

```text
# Choose the receiver in the list. The screen of the Windows computer
# must appear on the receiver.
```

**Expected:** the picture arrives and the mouse of the Windows computer
moves the pointer on the receiver.

**Step 3 — the second peer is refused.**

```text
# With the first Windows computer connected, repeat the step 1 with a
# second Windows computer.
```

**Expected:** the second computer sees an error; the first session keeps
running.

**Step 4 — the permission (FR-035).**

```text
# The WFD sink needs a permission of the system. The error of the
# sender must name the permission, not a generic error.
```

**Expected:** the receiver refuses the session with a message that names
the permission that is missing. The boundary is `EXC-003` (HDCP) — a
session that cannot be protected is refused cleanly.

---

### 2.3 The Google Cast path — a Windows 11 "Cast to Device", a Chromecast or an Android sender

| Requirement | What has to be proven | Sender |
|---|---|---|
| FR-037 | The Cast engine starts, advertises and stops. | Windows 11 |
| FR-038 | The CastV2 control plane is spoken over TLS. | Chromecast, Android |
| FR-039 | The application identifier is answered as the default media receiver. | Chromecast, Android |
| FR-040 | Several senders may be connected at the same time. | two senders |
| FR-041 | The advertised codecs are the ones the sender uses. | Chromecast |
| FR-042 | The media status of the session is reported. | Chromecast, Android |

#### The script

**Step 1 — the discovery.**

```text
# On the Windows 11 computer: "Cast to Device" in the task bar.
# The receiver must appear under the name of the device.
```

**Expected:** the receiver appears with the name of
`config/app-defaults.toml`.

**Step 2 — the session.**

```text
# Choose the receiver and play a video in the browser.
```

**Expected:** the video plays. The state of the receiver becomes
`streaming`, and one session is reported by the dashboard.

**Step 3 — the control plane over TLS.**

```text
# The transport of the Cast protocol is TLS on the port 8009. The
# certificate of the receiver is self-signed; the sender accepts it.
# The proof that the control plane is spoken: the sender answers the
# CONNECT with its own status, and the receiver shows the session.
```

**Expected:** the session is reported; the volume of the receiver can be
changed from the sender, and the change is visible in the dashboard.

**Step 4 — several senders (FR-040).**

```text
# With one sender connected, connect a second one.
```

**Expected:** **both** sessions run. Google Cast is the documented
exception to the one-peer rule (`EXC-008` names the other three
protocols). The number of the senders is bounded — a sender beyond the
bound is refused, and the receiver does not grow without a limit.

**Step 5 — the application identifier (FR-039).**

```text
# The receiver offers the Google "Default Media Receiver" (CC1AD845).
# The sender must show the name of the application.
```

**Expected:** the sender shows the default media receiver. The boundary is
`EXC-010`: the project ships **no** application identifier of its own and
does not invent one.

---

### 2.4 The DLNA path — VLC or a device with a UPnP client

| Requirement | What has to be proven | Sender |
|---|---|---|
| FR-043 | The SSDP answers are sent and the renderer is found. | VLC 3.0, a TV |
| FR-044 | The device description and the service descriptions are served. | VLC 3.0 |
| FR-045 | The SOAP dispatcher answers the three services. | VLC 3.0 |
| FR-046 | The GENA events are delivered. | VLC 3.0 |
| FR-049 | The payload limits of DLNA are enforced with a real sender. | VLC 3.0 |
| FR-051 | One DLNA peer at a time. | two players |

#### The script

**Step 1 — the discovery.**

```text
# On the sender: VLC -> Media -> Open Network Stream ->
#   "Browse for UPnP" / or any UPnP client.
```

**Expected:** the receiver appears as a media renderer.

**Step 2 — the description.**

```text
# The description is served over HTTP. The address of the renderer is
# the address of the receiver, the port is in the SSDP answer.
curl -s "http://${RECEIVER}:<port>/description.xml" | head -20
```

**Expected:** the XML of the description names the three services of the
renderer (`AVTransport`, `ConnectionManager`, `RenderingControl`).

**Step 3 — the SOAP.**

```text
# Play a file from the sender. The sender sends a SOAP body with an
# action of the AVTransport service.
```

**Expected:** the file is played, and the position of the playback is
reported.

**Step 4 — the GENA event (FR-046).**

```text
# The sender subscribes to the events of the renderer.
```

**Expected:** the renderer delivers the event of the state change to the
address of the sender. The callback URL of the sender is checked against
the SSRF guard (`FR-048`) — a sender that points at `169.254.169.254`
must be refused with the status `412`.

**Step 5 — the second player is refused (FR-051).**

```text
# With one player running, start a second one.
```

**Expected:** the second player is refused; the first session keeps
running.

### 2.5 The Samsung Smart View path — a Galaxy phone or tablet

| Requirement | What has to be proven | Sender |
|---|---|---|
| FR-104 | The receiver is found in the list of *Smart View* of the phone. | a Galaxy phone, Android 12 or later |
| FR-105 | The description document and the API of the framework are served. | a Galaxy phone |
| FR-106 | One sender at a time. | two phones |
| FR-108 | The channel is opened under the name that the sender gives it, and the greeting `ms.channel.connect` arrives. | a Galaxy phone |
| FR-109 | A key of the remote control reaches the receiver. | a Galaxy phone |
| FR-107 | **The receiver does not claim a picture that it cannot deliver** (`EXC-011`). | a Galaxy phone |

#### The script

**Step 1 — the discovery.**

```text
# On the phone: Settings -> Connected devices -> Smart View (or the
# screen of the quick settings). The list of the devices appears.
```

**Expected:** the receiver appears in the list with the name of the
configuration (`device.name`).

**Step 2 — the description and the API.**

```text
# The description is served over HTTP; the port of the framework is
# 8001, and the address of the receiver is the address of the SSDP
# answer.
curl -s "http://${RECEIVER}:8001/api/v2/" | head -20
```

**Expected:** the JSON of the root of the API names the service
`urn:samsung.com:service:MultiScreenService:1`, the version `2`, the base
URL and the device.

**Step 3 — the channel (FR-108).**

```text
# The channel is opened by the phone under the name that it gives it.
# To watch it, the handshake of the phone is seen in the log of the
# receiver, and the frame is seen with a WebSocket client:
#
#   websocat "ws://${RECEIVER}:8001/api/v2/channels/samsung.remote.control"
```

**Expected:** the receiver answers the upgrade with `101 Switching
Protocols`, and the **first frame** is the event `ms.channel.connect` with
the token of the session. The phone shows the receiver as connected.

**Step 4 — the key of the remote control (FR-109).**

```text
# On the phone: the remote control of Smart View. A key is pressed.
```

**Expected:** the receiver logs the key of the method `ms.remote.control`,
and the parameter `DataOfCmd` names it (for example `KEY_VOLUP`). The
session stays open.

**Step 5 — the second phone is refused (FR-106).**

```text
# With one phone connected, a second phone opens the list.
```

**Expected:** the second phone is refused, and the first session keeps
running.

**Step 6 — the honest limit (FR-107).**

```text
# The phone shows the receiver as connected. The screen of the phone is
# not mirrored.
```

**Expected:** **no picture of the screen of the phone appears on the
receiver.** The transport of the picture is proprietary and is not
implemented (`EXC-011`). The receiver finds the sender, holds the channel
open and understands the keys of the remote control — and no picture flows.
A report that claims a picture here is a false report.

---

## 3. The criteria that need a measurement, not a sender

| Requirement | The measurement | The command |
|---|---|---|
| NFR-PERF-001 | The time to the first picture | The stop watch of step 2 of the AirPlay script, against the budget `latency.first_frame_ms`. |
| NFR-PERF-002 | The frame rate over a session of `throughput.measurement_window_seconds` | The counters of the dashboard during a session of one minute. |
| NFR-PERF-003 | The resident memory | `ps -o rss= -C phairplay-web` during a session, against `resources.max_memory_mb`. |
| NFR-PERF-004 | The CPU usage | `ps -o pcpu= -C phairplay-web` during a session, against `resources.max_cpu_percent`. |
| NFR-PERF-005 | The drift of the audio and the video | The difference of the presentation timestamps of the frames at the beginning and at the end of a session, against `latency.audio_video_sync_ms`. |

**These five measurements were NOT RUN.** The continuous integration runs
on a build machine; a build machine has no sender, no display and no
network of a home. The acceptance record says NOT RUN with the reason.

---

## 4. The criteria that no device can prove

Some requirements are proved by the source itself and not by a device. They
are listed here so that nobody looks for a phone that proves them.

| Requirement | How it is proved |
|---|---|
| FR-001 | `cargo metadata --no-deps --format-version 1` and the check of the licence headers (`scripts/check-license-headers.sh`). |
| FR-002, FR-003, FR-004 | The search of the source for an account, a telemetry client and a promotional string, plus the outbound policy of `config/security-policy.toml`. |
| FR-005 | The guard of the callback URL and the policy of the sources in `config/security-policy.toml`. |
| FR-060 | The check of the crate boundaries (`scripts/check-crate-boundaries.sh`). |
| FR-078 | The same check. |
| NFR-SEC-005 | The workspace lints: `unsafe_code = "deny"`. |
| NFR-SEC-006 | The lint of the workspace and the review of the `panic!` in the production code. |
| NFR-PRIV-001 | The search of the source and the policy of the privacy. |
| NFR-PRIV-002 | The tests of the masking of the log. |
| NFR-REL-001 | The gate of the continuous integration and its self-test. |
| NFR-REL-002 | The acceptance record itself. |
| NFR-REL-003 | The review of the code and the tests: a stub returns nothing that a test could not also return. |
| NFR-DOC-001, NFR-DOC-002, NFR-DOC-003 | The check of the links of the documentation (`scripts/check-docs-links.sh`). |

---

## 5. The template of a result

Every result of this plan is written down in this form — in the acceptance
record, not in the head of the tester.

```text
### <the requirement> — <the name of the device>

- **Date:** <the date of the run>
- **Sender:** <the product, the version, the operating system>
- **Receiver:** <the revision of the repository, the host, the address>
- **Step:** <the number of the step of the script>
- **Expected:** <what the script says>
- **Result:** <what was seen>
- **Verdict:** pass / fail
- **Evidence:** <the command and its output, the photograph, the log>
```

A step without the evidence is not a result. A step that was not carried
out is **NOT RUN** with the reason — never a pass.
