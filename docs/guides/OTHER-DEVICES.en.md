# PhairPlayRust on other devices

> **Status:** living. This guide answers two questions: on which devices the
> PhairPlayRust receiver runs, and how to get it there.
>
> **Editions:** [`OTHER-DEVICES.en.md`](OTHER-DEVICES.en.md) (English, this
> file) and [`OTHER-DEVICES.md`](OTHER-DEVICES.md) (German). Both are
> complete; neither is a translation with gaps.
>
> **The numbers** — the time to the first picture, the frame rate, the
> resident memory, the CPU load, the queue budgets — live in
> [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml).
> This guide does not repeat them, it links to them. The numbers of the
> device classes live in the profiles under
> [`../../config/profiles/`](../../config/profiles/README.md), the ports in
> [`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).
>
> **The rule:** format, house style and the evidence rule live in
> [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md). A command
> that could not be executed in the environment of this document is marked
> **NOT RUN** with the reason.

---

## 1. The short answer

The core of PhairPlayRust is a Rust program without a platform API: no
window system, no driver, no codec library, no dependency on a particular
piece of hardware. It runs everywhere a Linux binary runs — on a Raspberry
Pi, in a container, on a NAS, in a Proxmox VM, on a mini PC, on a desktop.

The dependency on the device is **not** the receiver — it is the decoder.
The core parses the bitstream, reassembles the RTP packets and hands the
**encoded** frames to the platform; the interface for that is the
`VideoSink` in
[`../../core/phairplay-core/src/media.rs`](../../core/phairplay-core/src/media.rs).
The device does the decoding: MediaCodec on Android, VideoToolbox on Apple,
a browser on the web.

From this follows the honest statement that shapes the rest of this guide:

> On a device without a screen the receiver runs **as a service**. It
> accepts sessions, parses the stream and measures it. It shows **no
> picture** and plays **no sound**, because the headless build registers no
> media sink — the frames are accepted and discarded. The protocols work,
> the picture surface does not.

Whoever wants a picture takes the native shells (Android, macOS) or a device
that has a decoder attached. Whoever wants a device in the network takes one
of the installations in §4.

---

## 2. Which devices are in question

| Device | Profile | AirPlay 2, Google Cast, DLNA | Miracast | Picture |
|---|---|---|---|---|
| Raspberry Pi 3B, 4 or 5 | `raspberry-pi` | yes, over the ordinary network | no — the radio of the board cannot do Wi-Fi Direct | no |
| Orange Pi | `raspberry-pi` | yes, over the ordinary network | only with a USB adapter that can do Wi-Fi Direct | no |
| Docker container | `docker` | yes — but only with the network of the host | no — a container has no radio | no |
| Synology, QNAP, TrueNAS | `nas` | yes, over the ordinary network | no — a NAS has no radio | no |
| Proxmox VM | `nas` | yes, if the bridge forwards the multicast | no — a VM has no radio | no |
| Mini PC | `nas` or `generic` | yes, over the ordinary network | only with an adapter that can do Wi-Fi Direct | no |
| Linux desktop | `desktop` | yes | yes, if the adapter can do Wi-Fi Direct | no |
| WSL under Windows | `generic` — for development only | no — the multicast does not leave WSL | no | no |

The *profile* column names the file that describes the device class:
[`../../config/profiles/`](../../config/profiles/README.md). A profile is a
complete configuration; the receiver reads it with `--profile <name>` or with
`--config <file>`.

---

## 3. Why the core runs on every device

| Property | Where it comes from |
|---|---|
| no window system, no driver, no codec library | the core consists of the crates under [`../../core/phairplay-core/`](../../core/phairplay-core/src/lib.rs) and [`../../engines/`](../../engines/) — they speak TCP and UDP sockets and nothing else |
| no cgo, no system library | the crates build against the Rust standard library and the crates of the workspace; the only C interface of the product is the UniFFI bridge, and only the native shells use it |
| the decoder belongs to the device | the core hands encoded frames to the platform; the headless application registers no sink, the frames are discarded |
| the bridge to the native shells is optional | [`../../platform/phairplay-ffi/`](../../platform/phairplay-ffi/src/lib.rs) carries the Rust API **and** the UniFFI exports; the headless build uses the Rust API — the supervisor — and needs neither the C ABI nor the generated bindings |

The core is therefore **the same on every device**. What changes is the
profile: the device class decides which protocols are switched on and how
much memory and CPU the receiver may use.

---

## 4. The installation per device

### 4.0 The ground work — the build

The receiver is built from the repository. A device that builds it itself
needs a C compiler (the linker) and the Rust toolchain:

```bash
# on a Debian family (Raspberry Pi OS, Debian, Ubuntu, TrueNAS SCALE):
sudo apt install --yes build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

Whoever does not want to build on the device builds on a fast machine and
copies the result. The script [`../../deploy/install.sh`](../../deploy/install.sh)
accepts the result with `--no-build`; it expects the binary under
`target/release/phairplay-web` in the tree.

```bash
# on the fast machine:
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
cargo build --release --locked -p phairplay-web --bin phairplay-web
```

The binary is linked against the C library of the build machine. On an old
system — a NAS with an old kernel — the build on the device or the
container is therefore the safe way.

> **NOT RUN** in this environment: `rustup` was not installed here, because
> the environment has no network and no `apt`. The commands are the ones of
> the Rust website; the build itself was executed here with `cargo build
> --offline -p phairplay-web --bin phairplay-web` (a debug build) and is
> recorded in
> [`../audit/other-devices-guide.md`](../audit/other-devices-guide.md).

### 4.1 Raspberry Pi (3B, 4, 5) and Orange Pi

The Pi is the common case: a computer without a screen that is supposed to
stand in the network.

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile raspberry-pi
```

The script does, in this order: it builds the receiver, it creates the
unprivileged user `phairplay`, it installs the binary under
`/opt/phairplayrust/bin/`, the configuration into
`/etc/phairplayrust/config.toml`, the unit into
`/etc/systemd/system/phairplayrust.service`, and it enables and starts the
service.

After that:

```bash
# is the service running?
systemctl status phairplayrust

# what does the start report say?
journalctl -u phairplayrust -f
```

The start report names every protocol and, when one could not start, the
reason. A protocol that does not start takes nothing away from the others —
the receiver runs with the ones that work.

The report also names a protocol for which this build brings no engine. A
line like *Samsung Smart View not started — no engine implementation is
registered for the protocol `smartview`* means: the scheme is known to the
core, the engine is not part of this build. That is not a defect of the
device — the device only decides the radio question (§7.4).

If the machine has a firewall, the ports of the receiver have to be open:

```bash
sudo ufw allow 7000/tcp   # the RTSP control channel of AirPlay
sudo ufw allow 6002/udp   # the NTP timing service of AirPlay
sudo ufw allow 8009/tcp   # the TLS control channel of Google Cast
sudo ufw allow 7236/tcp   # the RTSP control plane of Miracast — only when Miracast runs
sudo ufw allow 1900/udp   # SSDP — the discovery of DLNA
sudo ufw allow 5353/udp   # mDNS — the discovery of AirPlay and of Google Cast
```

Miracast stays off on the Pi: the radio of the board cannot do Wi-Fi
Direct. §7.4 explains how the user checks that themselves.

> **NOT RUN** in this environment: the environment has no systemd and no
> `ufw`. `deploy/install.sh --dry-run --profile raspberry-pi` was executed
> and is recorded; the `systemctl` and `ufw` commands come from
> [`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service)
> and from the list of the ports in
> [`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).

### 4.2 Docker container

A container shares the kernel of its host, but not its network. That is
exactly why the network of the host is a requirement and not a nicety:

```bash
cd deploy/docker
cp ../../config/profiles/generic.toml ./phairplayrust.toml
docker compose up -d phairplay
```

`deploy/docker/docker-compose.yml` carries `network_mode: host` for the
service `phairplay`. That is mandatory: the receiver announces itself with
mDNS (UDP 5353) and with SSDP (UDP 1900), and both are multicast. A bridge
forwards neither — the container runs, the ports are open, and no phone
finds the receiver. The variant `phairplay-bridge` in the same file is the
counter-proof: it runs, it publishes the ports, and the announcement stays
inside the bridge.

Whoever wants to match the configuration of the container to the device takes
the matching profile file:

```bash
cp ../../config/profiles/raspberry-pi.toml ./phairplayrust.toml
```

Checking:

```bash
docker logs -f phairplayrust
curl --fail --silent --show-error http://127.0.0.1:7878/api/status
```

Without Compose, as a single container:

```bash
docker buildx build -f deploy/docker/Dockerfile -t phairplayrust:latest .
docker run --net=host --name phairplayrust phairplayrust:latest --profile raspberry-pi
```

The image carries the profiles (`/etc/phairplayrust/profiles`), so `--profile`
works in the container without any further preparation.

> **NOT RUN** in this environment: there is no Docker and no
> `docker compose` here. The commands are written down verbatim in
> [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml)
> and in [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile).

### 4.3 NAS — Synology, QNAP, TrueNAS

A NAS is a server without a screen: the container is the way the vendors
themselves offer — Container Manager (Synology DSM 7.2), Container Station
(QNAP QTS), Apps (TrueNAS SCALE). The names of the menus differ between the
versions; what is needed is the place where the vendor starts containers.
The compose file of §4.2 is brought onto the NAS and started there:

```bash
cd deploy/docker
cp ../../config/profiles/nas.toml ./phairplayrust.toml
sudo docker compose up -d phairplay
```

The image has to match the architecture of the NAS — NAS comes in x86_64 and
in ARM:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -f deploy/docker/Dockerfile -t phairplayrust:latest .
```

The container has to use the network of the NAS. `network_mode: host` means
exactly that. If the container manager of the NAS does not offer the host
network mode, the receiver runs natively on the NAS instead — that needs a
complete Linux userland though, and the BusyBox shell of the NAS vendors is
not one. In case of doubt the container is the right way.

A NAS has no radio: Miracast is switched off in the `nas` profile, and a NAS
without a screen has no decoder — the receiver serves the protocol there,
not the picture surface.

> **NOT RUN** in this environment: there is no Docker, no NAS and no
> `buildx` here. The commands are written down verbatim in
> [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml)
> and in [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile).

### 4.4 Proxmox VM

A **VM**, not an LXC container. An LXC container shares the network of the
host exactly like a Docker container, and the multicast stays inside exactly
the same way.

```bash
# in the VM, as the normal user:
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile nas
```

Why the profile `nas` and not `generic`: a VM has no radio, and `generic`
switches Miracast on. Whoever wants to switch Miracast off in their own
configuration writes it into the file:

```toml
[protocols.miracast]
enabled = false
```

The bridge of the hypervisor is the usual suspect when the sender does not
see the receiver:

```bash
cat /sys/class/net/vmbr0/bridge/multicast_snooping
```

A bridge that snoops the multicast only forwards a group to the ports that
joined it. The VM has joined, the physical port of the bridge often has not
— the announcement then stays inside the bridge. The way out is a bridge of
its own for the VM without snooping, or the receiver on the physical
machine.

The address matters even more: the receiver announces the address that the
sender can reach. A VM with a DHCP lease gets a different one after a
reboot — the sender does not find it any more. So set a fixed address or a
DHCP reservation.

> **NOT RUN** in this environment: there is neither Proxmox nor a bridge of
> that name here. The commands are standard tools of Linux; the profile
> `nas` and its refusal of Miracast are in
> [`../../config/profiles/nas.toml`](../../config/profiles/nas.toml).

### 4.5 Mini PC

A small box with an Ethernet port: the network works, the discovery works,
and it usually has no radio.

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile nas
```

`nas` is the base here, because it is the profile of a server without a
screen and it leaves Miracast switched off. Whoever wants to keep the
default of `generic` takes `--profile generic` and switches Miracast off in
the configuration.

### 4.6 Linux desktop

On the desktop the surface is local: the panel runs in the browser of the
same machine.

```bash
# from the repository, without an installation:
cargo run -p phairplay-web -- --profile desktop

# or as a service:
deploy/install.sh --profile desktop
```

The profile `desktop` switches every protocol on — Miracast presumes an
adapter with Wi-Fi Direct, which not every desktop has. Whoever has none
switches Miracast off in the configuration.

When Miracast runs, the firewall has to release the port of the protocol as
well: `7236/tcp`. The list of all the ports is in
[`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).

### 4.7 WSL under Windows

WSL is a **development environment, not a receiver**. Neither WSL 1 nor
WSL 2 forwards the multicast of mDNS and SSDP to the local network — the
phone does not find the receiver, whatever the configuration.

What WSL can do: build, test, use the panel and the API locally. WSL 2
forwards the localhost of the guest to the host by default, so the panel is
reachable from the browser of Windows under `http://127.0.0.1:7878`. The
receiver for the house is then the machine next to it.

> **NOT RUN** in this environment: there is no WSL here. The forwarding of
> the localhost is a default of the WSL configuration (`localhostForwarding`
> in `.wslconfig`); whoever changed it checks it.

---

## 5. The operation

### 5.1 As a service (systemd)

The unit lives in
[`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service).

| What | Command |
|---|---|
| State | `systemctl status phairplayrust` |
| Follow the log | `journalctl -u phairplayrust -f` |
| Restart | `systemctl restart phairplayrust` |
| Stop | `systemctl stop phairplayrust` |
| Do not start at boot any more | `systemctl disable phairplayrust` |

The service runs as the unprivileged user `phairplay`, not as root. It
binds no privileged port and needs no capability (`CapabilityBoundingSet=`
is empty). The unit starts only when the network is up
(`After=network-online.target`) — a receiver that starts before the network
sends its announcement into the void and stays invisible until the next
restart.

The log level is set in the unit (`RUST_LOG=info`). The unit reads an
additional file of variables when it exists:
`/etc/phairplayrust/phairplayrust.env`.

### 5.2 In the container

| What | Command |
|---|---|
| Follow the log | `docker logs -f phairplayrust` |
| Rebuild the image and start | `docker compose up -d --build phairplay` |
| Stop | `docker stop phairplayrust` |
| State of the API | `curl --fail --silent --show-error http://127.0.0.1:7878/api/status` |

### 5.3 The update

After a new release or after a `git pull`:

```bash
cd PhairPlayRust
git pull
deploy/install.sh --profile raspberry-pi
sudo systemctl restart phairplayrust
```

The restart is a step of its own and may not be left out: `systemctl enable
--now` does not restart a service that is already running. Without the
restart the service keeps running with the old binary.

In the container:

```bash
cd deploy/docker
docker compose up -d --build phairplay
```

---

## 6. The surface on other devices

### 6.1 The dashboard is a local panel

The dashboard is a **local control panel**, not a network service: it binds
to the loopback interface (`127.0.0.1`) and to nothing else. The default
port is `7878`; the receiver announces the address it listens on when it
starts.

From the device itself:

```bash
# open in the browser of the device:
#   http://127.0.0.1:7878
```

From another machine, without putting the panel into the network — the
forwarding of an SSH connection:

```bash
# the address is an example — replace it by the address of the device:
ssh -L 7878:127.0.0.1:7878 phairplay@192.168.1.5
```

After that the panel is reachable in the browser of the other machine under
`http://127.0.0.1:7878`.

Whoever wants the panel in the network on purpose binds it to an address:

```bash
phairplay-web --profile raspberry-pi --bind 0.0.0.0
```

The receiver then warns on stderr: every device in the network can open the
panel, read its configuration and switch its protocols off. The default is
and remains the loopback interface.

### 6.2 The remote control

The panel is operated with the keyboard and with a remote control — that is
the way a Raspberry Pi without a screen is steered.

| Key | Effect |
|---|---|
| arrow left, right, up, down | moves the focus between the protocol cards |
| enter or the space key | switches the receiver under the focus |
| escape | closes the message of a problem |
| home / end | jumps to the first or the last card of the group |
| tab | leaves the group of the cards |

The cards are **one** stop of the tab key, not one stop per card: one jump,
then the arrow keys. At the edge of the group the arrow keys go on: they
leave the group and put the focus on the element next to it. They do not stop
at an edge.

The panel shows the state of the protocols, the live sessions and the
counters. It shows **no picture** — see §7.1.

### 6.3 The API for scripts

For a script on the device:

```bash
curl --silent http://127.0.0.1:7878/api/status
curl --silent http://127.0.0.1:7878/api/sessions
curl --silent http://127.0.0.1:7878/api/metrics
```

And switching a protocol by hand:

```bash
curl --silent --request POST http://127.0.0.1:7878/api/protocols/airplay/disable
curl --silent --request POST http://127.0.0.1:7878/api/protocols/airplay/enable
```

A protocol that is switched off that way is off — it is no longer announced
and binds no socket any more. For good, the configuration decides:
[`CONFIGURATION.md`](CONFIGURATION.md) explains which file holds which value.

---

## 7. The limits

### 7.1 No hardware decoding without a platform

The receiver **does not decode**. It parses the bitstream, reassembles the
RTP packets and hands the encoded frames to the platform. Whether a picture
comes out of that is decided by the device:

- **Android** brings MediaCodec, **macOS** VideoToolbox, a browser on the
  web its own decoder. The native shells attach the sink.
- **The headless build on Linux** attaches no sink. The frames are accepted
  and discarded. The panel shows the state, not the picture.

Hardware decoding that the platform does not deliver does not exist: a
device without a decoder shows no picture, however good the network is.

### 7.2 No DRM content

| Scheme | What holds |
|---|---|
| FairPlay Streaming (Apple TV+, Netflix, Disney+) | not decryptable — no open-source receiver can do that without a licence from Apple (`EXC-001`) |
| in-app audio of Apple Music | protected on every AirPlay path (`EXC-002`) |
| HDCP 2.x | negotiated inside the WFD handshake and enforced by the hardware; the receiver does not implement the key exchange (`EXC-003`) |
| Widevine and PlayReady | belong to the DRM framework of the device, not to the receiver (`EXC-004`) |

**Smart View is not a DRM scheme.** It is Samsung's own open feature on
Galaxy devices: it needs neither HDCP nor a decryption. The message format
of the Smart View channel is proprietary and not publicly documented,
though; the engine implements the announcement and the handshake, not the
content of the channel
([`../../engines/smartview/`](../../engines/smartview/src/lib.rs), `EXC-011`).

**The engine is not registered in this build.** `phairplay-web` registers
four engines — AirPlay, Miracast, Google Cast, DLNA. The start therefore
reports Smart View as *not started*, with the reason `no engine
implementation is registered for the protocol 'smartview'`. A running
receiver does not answer the search of the MultiScreen Framework yet; the
milestone M5 in [`../../ROADMAP.md`](../../ROADMAP.md) carries that state.

### 7.3 No cloud operation

The receiver is a device in the local network. It relays no stream over the
internet, it offers no remote access, it has no account, no telemetry and
no advertising (`EXC-005`, `EXC-006`). The panel binds to the loopback
interface, because a panel in the network would be a door into the receiver.

### 7.4 Miracast needs Wi-Fi Direct

Miracast needs a radio link that only the device itself can form: Wi-Fi
Direct (P2P). The radio of the Raspberry Pi (the Broadcom chip with the
`brcmfmac` driver) offers no P2P mode — that is why Miracast is switched
off in the profile `raspberry-pi`. A receiver that announces `_miracast._tcp`
without the radio appears in the device list of a Windows phone and fails on
every attempt.

The user checks that themselves on the device:

```bash
iw phy phy0 info
```

The list *Supported interface modes* has to contain `* P2P-client` or
`* P2P-GO`. If they are missing, the board cannot form a Wi-Fi Direct group
and Miracast does not work on that board — a USB adapter with a P2P radio is
the only way.

On a desktop or a mini PC with a current adapter the P2P modes are usually
there; the check is the same.

### 7.5 The multicast has to get through

The discovery of AirPlay and of Google Cast runs over mDNS, the discovery of
DLNA over SSDP — both are multicast. A network that filters multicast makes
the receiver invisible without breaking it: AP isolation in the WLAN, a
guest network, a bridge with snooping, a container without the network of
the host. The table *symptom → cause → check* is in
[`TROUBLESHOOTING.md`](TROUBLESHOOTING.md).

---

## 8. The checklist for the user

| # | Check | Command | Expectation |
|---|---|---|---|
| 1 | The service runs | `systemctl status phairplayrust` | `active (running)` |
| 2 | The start report | `journalctl -u phairplayrust -f` | every protocol is named; every one that did not start names its reason |
| 3 | The state of the API | `curl --silent http://127.0.0.1:7878/api/status` | every protocol that is switched on stands on `advertising` |
| 4 | The discovery | a phone in the same network | the receiver appears in the device list |
| 5 | The radio | `iw phy phy0 info` | `P2P-client` or `P2P-GO` — only for Miracast |
| 6 | The firewall | `sudo ufw status` | the ports of the protocols are open |

The steps 1 to 3 answer the question *does the receiver run*. The steps 4 to
6 answer the question *does a sender find it*.

---

## 9. The sources

| Topic | File |
|---|---|
| The device profiles | [`../../config/profiles/README.md`](../../config/profiles/README.md) |
| The numbers of the product | [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml) |
| The installation as a service | [`../../deploy/install.sh`](../../deploy/install.sh) |
| The guide to the installation | [`../../deploy/README.md`](../../deploy/README.md) |
| The unit | [`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service) |
| The image and the compose file | [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile), [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml) |
| The command line | [`../../apps/web/src/cli.rs`](../../apps/web/src/cli.rs) |
| The composition root of the receiver | [`../../apps/web/src/receiver.rs`](../../apps/web/src/receiver.rs) |
| The configuration in detail | [`CONFIGURATION.md`](CONFIGURATION.md) |
| Symptom, cause, check | [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md) |
| The check against a real sender | [`DEVICE_TEST_PLAN.md`](DEVICE_TEST_PLAN.md) |
| The exclusions of the product | [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) |

---

*The German edition of this guide is
[`OTHER-DEVICES.md`](OTHER-DEVICES.md).*
