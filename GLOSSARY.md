# PhairPlayRust Glossary

The terms of the protocols this project speaks. Every entry says what the
term is **and** why it matters in this repository.

---

## The protocols

**AirPlay** — Apple's protocol family for mirroring a screen and streaming
audio to a receiver. AirPlay 2 adds the `RAOP` audio path, the pairing
procedures and the session encryption. A sender picks the receiver in the
device picker of the operating system.

**AirPlay 2** — The version of AirPlay that this project implements. It uses
RTSP as the control plane and RTP for the media.

**RAOP** — *Remote Audio Output Protocol*. The audio half of AirPlay 2. A
receiver that advertises only `_airplay._tcp` but not `_raop._tcp` is
invisible to some senders — the project therefore advertises both.

**Miracast** — The Wi-Fi Display standard (the WFD standard of the Wi-Fi
Alliance). A sender pushes a screen to a receiver. It is the protocol that
Windows uses for *Cast to Device* and that many Android devices offer as
*Screen cast*.

**WFD** — *Wi-Fi Display*. The name of the Miracast standard itself. The
protocol has a control plane (RTSP) and a media path (usually MPEG-TS). A
WFD sink needs a system-level permission on the platforms that provide one;
see `FR-035`.

**UPnP** — *Universal Plug and Play*. A family of protocols for discovering
and controlling devices in a local network. UPnP is the framework; DLNA is a
profile of it.

**DLNA** — *Digital Living Network Alliance*. The profile of UPnP that
defines what a *MediaRenderer* must offer so that a controller can push
audio, video or pictures to it. The receiver implements the DLNA media
renderer.

**SSDP** — *Simple Service Discovery Protocol*. The UDP-based discovery of
UPnP: a controller sends an `M-SEARCH` multicast datagram, the devices
answer with a `NOTIFY` or with a direct answer. The receiver answers SSDP
for the DLNA path.

**SOAP** — *Simple Object Access Protocol*. The XML-based request/response
protocol that UPnP uses for its control. A request carries a `SOAPAction`
header, the body is XML, and the answer is a SOAP envelope.

**GENA** — *Generic Event Notification Architecture*. The UPnP mechanism that
pushes a state change to a subscriber. The subscriber hands over a callback
URL, and the receiver then connects to that URL. That makes the callback URL
an address the receiver connects to — and therefore a server-side request
forgery vector, which is why the project has an SSRF guard for it.

**SCPD** — *Service Control Protocol Description*. The XML document that
describes the actions and the state variables of a UPnP service. The receiver
serves one per service.

**CastV2** — The framing of the Google Cast control plane: length-prefixed
messages over a TLS connection, with a heartbeat and an application
identifier.

**RTSP** — *Real Time Streaming Protocol*. The request/response control plane
of AirPlay and of Miracast. The methods are `OPTIONS`, `ANNOUNCE`, `SETUP`,
`RECORD`, `TEARDOWN`, `GET_PARAMETER`, `SET_PARAMETER`, `FLUSH` and `PAUSE`.

**HLS** — *HTTP Live Streaming*. Apple's adaptive streaming format. The
receiver does not implement HLS as such; it matters here because FairPlay
DRM is defined for HLS content, and that content is out of scope (`EXC-001`).

**Samsung Smart View** — The screen mirroring of the Galaxy phones and
tablets: the user opens *Smart View* on the phone and picks the receiver.
The protocol belongs to the *MultiScreen Framework* (MSF), and the receiver
announces itself under the service type `_samsungmsf._tcp`. The API of the
framework listens on the port 8001 — the owner of that number is
`core/phairplay-core/src/config/ports.rs`.

*What the project implements:* the discovery (the answer to the search of the
framework), the description document and the handshake of the sender. *What
it does not implement:* the message format of the channel. That format is
proprietary and not publicly documented, so a Galaxy device finds the
receiver and connects to it, but no picture flows yet (`EXC-011`).

**MSF** — *MultiScreen Framework*. The framework behind Samsung Smart View. A
Galaxy device looks for a receiver with an SSDP search for
`urn:samsung.com:service:MultiScreenService:1`, reads the description
document that the answer points at, and posts its own address and its name
to the receiver. The three steps are documented by Samsung; the message
format that follows them is not.

**WebSocket** — The protocol that upgrades an HTTP connection into a
persistent, bidirectional channel. The project uses it twice: for the event
channel of the dashboard (the live counters) and for the channel of the
Samsung Smart View API.

---

## The negotiation and the media

**SDP** — *Session Description Protocol*. The text format in which a sender
describes a session: the media sections, the payload types and the
codec-specific parameters. In AirPlay, the SDP carries the H.264 parameter
sets, the ALAC parameters and the key material.

**RTP** — *Real-time Transport Protocol*. The transport of the media: a
sequence number, a timestamp, a payload type and the payload. In AirPlay the
RTP packets are interleaved into the RTSP connection.

**RTCP** — *Real-time Control Protocol*. The companion of RTP: the sender
reports the lost packets and the jitter. The receiver counts what the sender
reports in `StreamCounters::packets_lost`.

**NTP** — *Network Time Protocol*. The clock that aligns the audio and the
video of a session. The sender is the time source; the receiver follows it.

**DACP** — *Digital Audio Control Protocol*. The *reverse* remote of AirPlay:
the receiver controls the playback of the sender — play, pause, skip — and
the sender publishes its state with DACP as well.

**DMAP** — *Digital Media Access Protocol*. The metadata format of Apple: the
title, the artist, the album and the artwork of the playing track.

**EDID** — *Extended Display Identification Data*. The block of information
that a display announces about itself: its capabilities, its native
resolution and its supported modes. A receiver that wants to offer a picture
mode consults it.

**HDCP** — *High-bandwidth Digital Content Protection*. The link protection
of a video path. In Miracast it is negotiated inside the WFD handshake and
enforced by the hardware. The receiver does not implement the key exchange
and refuses a session it cannot protect (`EXC-003`).

**H.264** — *AVC*. The video codec that every protocol of this project
requires as a baseline. The project parses the sequence and the picture
parameter set and reassembles the fragmented RTP payloads; it does not decode
the picture itself.

**FU-A** — *Fragmentation Unit A*. The RTP payload format that carries a
large NAL unit in several packets. The reassembly belongs to the format
layer.

**SPS / PPS** — *Sequence / Picture Parameter Set*. The H.264 headers that
describe the picture. The sender delivers them in the SDP, and the receiver
parses them to learn the resolution and the profile.

**ALAC** — *Apple Lossless Audio Codec*. The lossless codec of Apple, used by
AirPlay for music. The project carries a native Rust port of the Apple
reference decoder, with the attribution of the reference in every derived
file.

**AAC-ELD** — *Enhanced Low Delay AAC*. The audio codec that AirPlay uses for
mirroring, because it needs a low delay.

**AAC-LC** — *Low Complexity AAC*. The ordinary AAC profile; it is one of the
baseline audio codecs of the Cast protocol.

**ADTS** — *Audio Data Transport Stream*. The frame header of a raw AAC
stream. The AAC crate of the project parses it.

**LPCM** — *Linear Pulse Code Modulation*. Uncompressed audio. It is the
baseline audio codec of the WFD protocol and the cheapest path for a
receiver, because no decoder is needed.

**MIME type** — The string that names a codec on the wire, for example
`video/avc` or `audio/mp4a-latm`. The capability model of the core produces
the list of the MIME types that the receiver advertises.

**property list (plist)** — The Apple serialisation format, in an XML and in a
binary form (`bplist00`). The sender uses it for the pairing, the identity
and the metadata.

---

## The television and the remote control

**D-pad** — The direction pad of a remote control: the arrow keys *up*,
*down*, *left* and *right*, plus the button in the middle that the platform
calls *OK*. A television application is built for the D-pad, not for a mouse.

**Leanback** — The launcher of Android TV. An application that carries the
`LEANBACK_LAUNCHER` category in its manifest appears in the list of the
television instead of in the list of the applications of a phone. The
application also needs a banner (a picture of 320 × 180 dp that the launcher
scales) and the `uses-feature` of the television. The Android shell of
PhairPlayRust carries all three, and it stays one application for the phone
and for the television.

**roving tabindex** — The pattern that makes a group of controls one tab stop
instead of many. Exactly one element of the group carries `tabindex="0"`, the
others carry `tabindex="-1"`, and the arrow keys move the focus inside the
group. The benefit is that a remote control — which usually has no tab key —
can reach every control of the group with the arrow keys alone.

**focus frame** — The drawn frame around the element that holds the focus.
The project draws its own frame instead of leaning on the frame of the
system, because the frame of the system is thin and dark and is nearly
invisible on a dark surface. The frame of the dashboard and the frame of the
Android shell are measured against the surface they lie on.

**ten-foot interface** — An interface that is read from three metres away:
larger text, a visible focus, a state that is carried by a word and a shape
and not by a colour alone. The shells and the dashboard of PhairPlayRust
follow this rule.

---

## The device and the headless build

**headless** — A receiver that runs **without a screen**: no window, no
click, no browser. The program starts the protocols itself at the start of
the process and reports the outcome of every one of them on the console. On
a Raspberry Pi or on a NAS the protocol *is* the product — the dashboard is a
control panel and nothing more.

**device profile** — The configuration of a **class of devices**, not of one
device: a Raspberry Pi, a NAS, a container, a desktop, or a device nobody
described yet. A profile is one TOML file in `config/profiles/`, and the name
of the profile is the name of the file. A profile decides which protocols are
switched on, what the device announces and what its resource budgets are; it
never decides the security model.

**the installer** — `deploy/install.sh`: it builds the receiver, installs it
under a prefix, installs the configuration and the systemd unit, and enables
the service as an unprivileged user. It prints every command it runs and has
a `--dry-run` that changes nothing.

**the composition root** — The one place that builds the engines of the
receiver and hands them to the supervisor: `apps/web/src/receiver.rs`. A
second registration of the engines would be a second truth about what runs,
so there is only one.

**the supervisor** — The object of the bridge (`ReceiverCore` in
`platform/phairplay-ffi`) that starts, stops and watches the engines. The
engines are handed to it once, in the composition root, and are never talked
to again: the status of a protocol is read from the supervisor, not from the
engine.

**the device test plan** — The list of the checks that need a real sender, a
real television or a real device: `docs/guides/DEVICE_TEST_PLAN.md`. Every
row of it is **NOT RUN** until somebody executes it on a device.

---

## The discovery

**mDNS** — *Multicast DNS*. The UDP-based name resolution and service
discovery in a local network. The receiver announces itself with it.

**DNS-SD** — *DNS Service Discovery*. The service discovery on top of mDNS:
the service types (`_airplay._tcp`), the instance name and the TXT record.
The receiver uses it for AirPlay, Miracast and Cast.

**service type** — The name under which a service is announced, for example
`_airplay._tcp`. The AirPlay receiver needs two of them.

**instance name** — The name of the announced service, for example
*Living Room*. The project sanitises it and caps it at the DNS-SD limit.

**TXT record** — The key/value list that belongs to a DNS-SD announcement. The
AirPlay receiver announces its capabilities and its features in it.

---

## The security

**FairPlay** — Apple's DRM system. **Important:** FairPlay Streaming is the
DRM of licensed content and is *not* decryptable by an open-source receiver
(`EXC-001`). The *session key* of the AirPlay mirroring path is a different
mechanism: the receiver decrypts the stream with it, and that part *is*
implemented.

**Widevine** — Google's DRM system. It belongs to the DRM framework of the
device, not to the receiver (`EXC-004`).

**PlayReady** — Microsoft's DRM system. It belongs to the DRM framework of the
device (`EXC-004`).

**pairing** — The procedure that lets a sender and a receiver trust each
other once, so that the connection is accepted afterwards without a PIN.

**SRP** — *Secure Remote Password*. The protocol of the *legacy* AirPlay
pairing, in which a PIN is typed on the sender. The modern AirPlay pairing
uses Ed25519 and X25519 instead.

**Ed25519** — The signature scheme of the modern AirPlay pairing. The sender
signs, the receiver verifies.

**X25519** — The key agreement of the modern AirPlay pairing. The two sides
derive a shared secret that neither can compute alone.

**TLS** — The transport encryption. The control plane of the Cast protocol
is TLS; the control plane of AirPlay is plaintext RTSP.

**SSRF** — *Server-Side Request Forgery*. The attack in which a service is
made to connect to an address that the attacker chose. The GENA callback of
the DLNA path is such a case, and the project guards it.

---

## The implementation

**UniFFI** — The tool that generates the bindings of a Rust library for other
languages. The project uses it for the Kotlin and the Swift shells. The
generated files are never edited by hand.

**Tokio** — The asynchronous runtime of Rust that the receiver uses. It
provides the tasks, the timers, the channels and the sockets.

**mpsc** — *Multi-producer, single-consumer*. The shape of the channel that
carries the session events from the engines to the event pump.

**sink** — The trait that receives the encoded frames of an engine. A sink
belongs to the platform and does the decoding.

**encoded frame** — A frame in its compressed form, together with the
information that a decoder needs. The engine delivers encoded frames; it
never decodes.

**the job executor** — The bounded pool of the core that runs the work that
must not block the event loop of a protocol.

**the session registry** — The structure of the core that folds the session
events into one live session per protocol.

**the build contract** — The binding document
[`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) that fixes
the interfaces between the crates.

**ADR** — *Architecture Decision Record*. A short document that records a
decision, its reason and its consequences. An ADR is never deleted; a
superseded decision stays readable.

**CR** — *Change Request*. The document that a material product change
requires.

**DCO** — *Developer Certificate of Origin*. The sign-off that external
contributions carry instead of a CLA.

**REUSE** — The specification that keeps the license information with the
files. The project carries a `REUSE.toml` for that.

**best in class** — A measurable quality target, not a marketing claim and
not a feature count. The framework is
[`docs/BEST_IN_CLASS.md`](docs/BEST_IN_CLASS.md).
