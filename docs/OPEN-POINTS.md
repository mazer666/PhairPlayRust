# The open points of the project, and what they cost

> **What this document is:** the list of the things that are **known** to be
> missing, in the order of the damage they do, with the evidence for each
> one and the work that closes it.
>
> **Status:** living. A point that is closed leaves a line here — the
> history of the project is read by people who were not there.
>
> **Why this document exists:** every acceptance record carries its own
> limitations, and every one of them is true. The problem is the *order*:
> a reader of the records cannot tell which of the twenty points is the one
> that hurts a user. This document answers that question, and it is short
> on purpose.

---

## The order of the damage

### 1. The picture does not flow from a Galaxy device — **S1**

`EXC-011`. The transport of the picture of the screen is proprietary and
not implemented. Everything else on the channel works: the discovery, the
handshake, the channel under the name that the sender gives it, the events
and the keys of the remote control.

**What it costs:** a household with a Galaxy phone cannot mirror its
screen — and the receiver is *found* by the phone, which makes it worse
than a device that is not found at all.

**What closes it:** the observations of the transport. A proprietary format
is not an unimplementable one — the Miracast, the Cast and the AirPlay
receivers of this project are implementations of formats that nobody
documents either. The work is a clean-room implementation from the
observations, and it needs a **capture**: a Galaxy phone that sends its
screen to the receiver, recorded.

**The next step that does not need the capture:** the DMP path. The
documented *Default Media Player* sends a **URL** to the receiver and the
receiver plays it — no proprietary format at all. That path gives the
Galaxy phone a real function on the receiver without the picture.

### 2. No sender has ever been tested — **S1**

Every row of the device test plan that needs a sender is **NOT RUN**. The
parsers, the answers and the limits are proved by tests; whether a real
AirPlay, Miracast, Cast, DLNA or Smart View sender works with the receiver
is not known.

**What it costs:** everything else is a statement about the code, not about
the product. A user who installs the receiver finds out whether it works.

**What closes it:** a sender and a television. The script is
[`guides/DEVICE_TEST_PLAN.md`](guides/DEVICE_TEST_PLAN.md) — the
sections 2.1 to 2.5, one per protocol. This is the **only** point that no
amount of code closes.

### 3. The advertised codecs do not follow the capabilities — **S2**

`MediaCapabilities::supports_cast_baseline` and `supports_miracast_baseline`
exist in the core and are tested there — and **no engine calls them**. The
engine of Miracast hard-codes its audio codecs
(`engines/miracast/src/rtsp.rs`).

**What it costs:** the receiver advertises a codec that the device may not
be able to decode. The sender picks it, and the picture does not arrive.
The promise of `FR-041` and `FR-007` holds in the core and not in the
network.

**What closes it:** the engines ask the core before they advertise. It is
a small change in two engines and a test that proves that a device without
the capability does not see the codec in the advertisement.

### 4. The GENA delivery has no test — **S2**

`gena::notify_subscribers` is called by the engine and is **never** tested.
The subscription and the SSRF guard are proved; the delivery is not.

**What it costs:** a DLNA control point that subscribes may never receive
the event of a state change — and nothing would notice.

**What closes it:** a test with a subscriber that receives the event. It
needs no hardware: the subscriber is a socket on the loopback interface.

### 5. The event channel of the dashboard has no test — **S2**

`apps/web/src/events.rs` has no test; only the conversion of an event is
tested. `FR-083` asks that the events of the core appear in the dashboard
without a reload.

**What it costs:** the dashboard shows a state that is stale until the user
reloads the page.

**What closes it:** a test of the endpoint. It needs no hardware.

### 6. The pairing of the AirPlay engine is lost on a restart — **S2**

The store of the pairing material is in memory only.

**What it costs:** a sender has to pair again after the receiver was
restarted. On a Raspberry Pi that is every power cut.

**What closes it:** a file of the pairing material, in the state directory
of the service — with the rights of the service user and nothing else in
it.

### 7. The metadata of a session is dropped — **S2**

The core drops the metadata of a session: the name of the sender, the
title of the piece, the album.

**What it costs:** the dashboard shows a session without a name, and the
shell cannot show what is playing.

**What closes it:** the metadata travels from the engine to the core to
the dashboard. The path exists — the field is in the snapshot — it is
simply not filled.

### 8. The AirPlay engine does not use the format crates — **S2**

The engine of M1 brought its own property-list parser and its own H.264
splitter, and the crates of the workspace (`formats/plist`, `formats/h264`)
are not the ones it uses.

**What it costs:** two parsers for one format. A defect in the crate does
not reach the engine, and a defect in the engine does not reach the crate.

**What closes it:** the engine uses the crates. It is a bigger change than
the other points, and it is worth it — the crates are the part of the
project that is meant to be reused.

### 9. The advertisement does not follow the network — **S3**

The re-advertisement after a network change and the liveness check do not
exist.

**What it costs:** a receiver whose network changed — a Raspberry Pi that
got a new address over DHCP — stays invisible until it is restarted.

**What closes it:** a watch of the address of the interface and a
re-announcement when it changes.

### 10. The format of the wire of the AirPlay engine — **S3**

Four RTSP verbs have no test, and the audio path of the engine is not
tested at all.

**What it costs:** an untested branch of the protocol. The probability
that a sender uses one of the four verbs is not small.

**What closes it:** tests. They need no hardware.

### 11. The unknown route of the AirPlay engine answers `501` — **S4**

A request to an unknown route answers `501 Not Implemented` and not
`404 Not Found`.

**What it costs:** almost nothing — but it is a lie about the state of the
server, and the rule of the acceptance records says that a lie is a defect
however small it is.

**What closes it:** one line.

### 12. The generator of the bindings ignores `CARGO_TARGET_DIR` — **S4**

`scripts/generate-bindings.sh` looks for the shared library in
`target/debug` and fails when the build went somewhere else. The CI uses
the default directory and does not see it.

**What it costs:** a contributor who builds into a different directory cannot
generate the bindings.

**What closes it:** one line — the script asks Cargo where the directory
is.

---

## What is **not** on this list, and why

| Thing | Why it is not here |
|---|---|
| The picture of the screen of a Galaxy device | it is **1** — the top of the list |
| The performance budgets | they are **NOT RUN** because they need a device; they are in the device test plan, §3 |
| The Rust security advisory `RUSTSEC-2023-0071` (the Marvin attack of `rsa`) | the crate has no fixed version; the advisory says so explicitly. The exception is accepted and written down in the record of M4 |
| The crate `basic-toml 0.1.10` is unmaintained | the advisory is a warning, not a vulnerability. It is allowed in `deny.toml` with a reason |
| The files above the soft limit of the sources | ten files are above it and none is above the hard limit. The soft limit is a target, not a rule — the gate says so, and the number is in [`../config/performance-budgets.toml`](../config/performance-budgets.toml) |
| The container image was not built | `docker` is not installed in the sandbox. The Dockerfile is in the repository and the compose file is beside it |

---

## The rule of this list

A point that is closed leaves a line here, with the commit that closed it
and the test that proves it. A point that is **not** on this list and that
somebody finds is added — the list is not a promise, it is a record.

The limits of the **product** — the things that no work closes — are in
[`REQUIREMENTS.md`](../REQUIREMENTS.md), the exclusion list
`EXC-001` to `EXC-011`. The limits of the **work** are here.
