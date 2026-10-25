# The five protocols, from the point of view of a sender

A guide for the developer who wants to know what a sender actually does, and
where the limits of the receiver are.

The architectural version of this is in
[`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) §4. This guide is the
practical one.

---

## 1. The shared shape

All five protocols have the same shape, and the receiver implements the shape
once, as the trait `ProtocolEngine`:

```text
advertise → accept → negotiate → stream → report → tear down
```

The differences are in the middle: how the peer is found, what the
handshake looks like, and how the media is transported.

| | AirPlay 2 | Miracast | Google Cast | DLNA / UPnP | Smart View |
|---|---|---|---|---|---|
| Discovery | DNS-SD | DNS-SD | DNS-SD | SSDP | SSDP |
| Service type | `_airplay._tcp`, `_raop._tcp` | `_miracast._tcp` | `_googlecast._tcp` | — | `_samsungmsf._tcp` |
| Control plane | RTSP over TCP | RTSP (WFD) over TCP | CastV2 over TLS | HTTP + SOAP | HTTP + WebSocket |
| Transport | RTP, interleaved | RTP, usually MPEG-TS | Cast media channels | HTTP | WebSocket |
| Picture | yes | yes | yes | no (audio and pictures) | **not implemented** (`EXC-011`) |
| Sound | yes | yes | yes | yes | **not implemented** (`EXC-011`) |
| Peers at a time | one | one | several | one | one |
| Milestone | M1 | M2 | M2 | M2 | M5 |

The "peers at a time" rule is data, not code: the
`session.multi_peer_protocols` list in
[`../../config/app-defaults.toml`](../../config/app-defaults.toml).

---

## 2. AirPlay 2

### What the sender does

1. The sender looks for a service of the type `_airplay._tcp` and, for the
   audio path, `_raop._tcp`. **A receiver that only offers the first one is
   invisible to some senders.**
2. The sender opens a TCP connection to the RTSP port of the receiver — the
   standard port of the protocol is 7000 — and asks `OPTIONS`.
3. The sender announces the session with `ANNOUNCE` and puts the description
   of the media into the **SDP**: the H.264 parameter sets, the ALAC or AAC
   parameters and the key material.
4. The sender sets up the streams with `SETUP` and starts with `RECORD`.
5. The media flows as RTP, interleaved into the same connection. The video is
   H.264, the sound is AAC-ELD, AAC-LC or ALAC.
6. The session is controlled with `TEARDOWN` and `FLUSH`; the metadata and
   the reverse remote travel over the DACP and the HTTP-like routes.

### What the receiver does with it

- parses the RTSP request under the limits of the configuration,
- parses the SDP and **rejects** it when it is malformed — it never repairs
  it,
- decrypts the stream with the session key of the pairing,
- reassembles the fragmented RTP payloads and hands the frames to the sinks,
- uses the timing of the sender to align the audio and the video.

### The limits

- **A second peer is refused**, not queued.
- **FairPlay DRM content is not decrypted** (`EXC-001`). The session
  encryption of the mirroring path is a different mechanism and *is*
  implemented.
- **The in-app audio of the Apple Music app is protected** (`EXC-002`).
- **The buffered audio mode is accepted but not played back** (`EXC-007`).

---

## 3. Miracast / Wi-Fi Display

### What the sender does

1. The sender looks for a `_miracast._tcp` service, or it uses the Wi-Fi
   Display information elements of the platform.
2. It negotiates the session with the RTSP control plane of WFD: the
   capabilities of the sink, the source, the transport.
3. The media flows as RTP, usually inside MPEG-TS. The video is H.264, the
   sound is linear PCM.

### What the receiver does with it

- parses the WFD information elements and answers the sink discovery,
- advertises the baseline codecs: H.264 and linear PCM,
- refuses a session that requires HDCP and that the platform cannot protect
  (`EXC-003`).

### The limits

- **The platform must offer the WFD feature.** A device without it does not
  advertise the service; the receiver does not work around a missing system
  permission (`FR-035`).
- **A second peer is refused.**
- The AirPlay-style pairing does not exist here; the access control of the
  local network is the same rule as for every other protocol.

---

## 4. Google Cast

### What the sender does

1. The sender looks for a `_googlecast._tcp` service and reads the TXT
   record with the application identifier.
2. It opens a **TLS** connection to the Cast port — the standard port of the
   protocol is 8009 — and speaks the CastV2 framing.
3. It sends the application identifier, answers the heartbeat and opens the
   media channels.
4. It reports the state of the playback, which the receiver shows.

### What the receiver does with it

- speaks CastV2 over TLS,
- answers the heartbeat,
- advertises the baseline codecs of the protocol: H.264 and VP8 for the
  video, AAC-LC, MP3 and linear PCM for the audio,
- serves **several senders at the same time** — the documented exception to
  the one-peer rule.

### The limits

- **The end-to-end test needs an application identifier that is registered
  with Google.** The project does not ship one and does not fake one
  (`EXC-010`).
- **The receiver does not depend on the Cast SDK being installed.** A device
  without it runs the receiver; only the Cast service is missing.
- **No Widevine, no PlayReady** — those belong to the DRM framework of the
  device (`EXC-004`).

---

## 5. DLNA / UPnP MediaRenderer

### What the controller does

1. The controller sends an SSDP `M-SEARCH` multicast datagram to the port
   1900. The receiver answers with its description.
2. The controller fetches the device description and the service description
   documents over HTTP.
3. It controls the renderer with SOAP: a request with a `SOAPAction` header
   and an XML body, for example `SetAVTransportURI` on the AVTransport
   service.
4. It subscribes to the state variables with GENA. The subscription carries a
   **callback URL** — an address the receiver will connect to.

### What the receiver does with it

- answers the SSDP search and registers itself with the `alive` notification,
- serves the device and the service descriptions,
- dispatches the SOAP actions of the ConnectionManager, the AVTransport and
  the RenderingControl,
- delivers the GENA events to the subscribers.

### The limits

- **The renderer serves audio and pictures, not a live video stream**
  (`FR-050`).
- **The callback URL is checked.** Only a private address of the local subnet
  is accepted; the link-local address of the cloud metadata service, the
  loopback address and the unspecified address are refused. The guard is the
  `callbacks` section of
  [`../../config/security-policy.toml`](../../config/security-policy.toml).
- **A SOAP body with a `DOCTYPE` or with an external entity is refused.**
- **A second peer is refused.**

---

## 6. Samsung Smart View

### What the sender does

1. The sender searches for the MultiScreen Framework with an SSDP `M-SEARCH`
   multicast datagram. The receiver answers with the description of its
   framework.
2. The sender fetches the description document over HTTP — the port of the
   API is 8001 — and reads the routes of the framework below `/api/v2/`.
3. The sender opens the channel with a WebSocket and announces its address
   and its name in the handshake.

### What the receiver does with it

- answers the SSDP search of the framework,
- serves the description document,
- answers the routes of the API below `/api/v2/`,
- performs the handshake of the sender and holds the channel open.

### The limits

- **The message format of the channel is not implemented** (`EXC-011`). The
  framework of Samsung is proprietary and only documented through the closed
  SDK. The receiver therefore **finds** the sender and **holds** the channel,
  but it does not deliver a picture. The device appears in the list of a
  Galaxy phone and connects — and no picture flows.
- **One peer at a time.**
- **The channel is not a DRM path.** The exclusion of FairPlay (`EXC-001`)
  does not apply to it.

---

## 7. The sender matrix

The matrix that decides when a protocol is accepted. A sender that could not
be tested is named with the reason — a silent gap is a false claim.

| Protocol | Sender | Platform of the sender | State |
|---|---|---|---|
| AirPlay 2 | iPhone, iPad | iOS / iPadOS | M1 |
| AirPlay 2 | Mac | macOS | M1 |
| AirPlay 2 | Apple TV | tvOS | later |
| Miracast | Windows "Cast to Device" | Windows | M2 |
| Miracast | Android "Screen cast" | Android | M2 |
| Google Cast | Chrome | any | M2 |
| Google Cast | Chromecast, Google TV | any | M2 |
| Google Cast | the Cast SDK | Android, iOS | M2 |
| DLNA | Windows "Cast to Device" | Windows | M2 |
| DLNA | VLC | any | M2 |
| DLNA | a mobile UPnP controller | Android, iOS | later |
| Smart View | a Galaxy phone or tablet | Android | M5 — the discovery and the handshake, the picture is `EXC-011` |

The device test plan with the exact steps belongs into the acceptance record
of the milestone that covers the protocol.

---

## 8. The standard ports and how to watch the traffic

This section is the **single place** where the ports of the five protocols
are written down. Every other document links here instead of repeating a
number.

| Protocol | Traffic | Port | Note |
|---|---|---|---|
| AirPlay 2 | RTSP control plane and the interleaved RTP | TCP 7000 | the standard port of the protocol |
| AirPlay 2 | the audio path of RAOP | UDP 6000/7001 | used by the audio-only session |
| Miracast | the WFD control plane | the port that the sink announced | the sender connects to the port of the Information Element; it is not a fixed number |
| Google Cast | the CastV2 control plane | TCP 8009 | TLS |
| DLNA / UPnP | SSDP discovery | UDP 1900 | multicast |
| DLNA / UPnP | the description documents and the SOAP control | TCP 8200 | the conventional port of the media renderer |
| Smart View | the API and the channel of the framework | TCP 8001 | the port of the MultiScreen Framework |
| all | DNS-SD | UDP 5353 | multicast |

The receiver does not pin these ports in the configuration: the
`protocols.<name>.port` key of
[`../../config/app-defaults.toml`](../../config/app-defaults.toml) is `0`,
which means that the engine binds the port of its protocol. The port that the
engine actually listens on is reported by `ProtocolStatus::tcp_port`.

To watch what a sender does, on a machine with the usual tools installed:

```bash
# The discovery traffic: DNS-SD and SSDP
sudo tcpdump -i any -n 'udp port 5353 or udp port 1900'

# The RTSP control plane of AirPlay
sudo tcpdump -i any -n 'tcp port 7000'

# The WFD control plane of Miracast: the port varies, so filter on the
# negotiated port that the sink announced
sudo tcpdump -i any -n 'tcp'

# The Cast control plane
sudo tcpdump -i any -n 'tcp port 8009'

# The description documents and the SOAP control of the DLNA renderer
sudo tcpdump -i any -n 'tcp port 8200'

# Everything that arrives on the interface of the receiver
sudo tcpdump -i any -n -A
```

A sender that finds no receiver usually means one of three things:

1. the multicast of the network is blocked (the router filters it),
2. the service is advertised under the wrong name — AirPlay needs **both**
   `_airplay._tcp` and `_raop._tcp`,
3. the receiver and the sender are in different subnets.

---

## 9. The parts that are common to all five

- **Discovery is not authentication.** Every device in the local network may
  reach the receiver. Whether a PIN is required is a setting of the user
  (`NFR-SEC-004`).
- **Every input is untrusted.** A length limit, a recursion limit and a
  deadline on every read.
- **Every queue is bounded twice**, by an item count and by a byte budget.
- **The receiver never decodes and never queries the platform.** It receives
  the identity, the capabilities and the sinks and it delivers encoded
  frames.
