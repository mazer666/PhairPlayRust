# The installation of the receiver

<!--
WHAT: the deployment directory of PhairPlayRust — the three artifacts that
      put the headless receiver onto a device, and the two ways to run it.

WHY: the receiver is the product, and the product has to run on a device
      that has no screen: a Raspberry Pi, a NAS, a mini PC. The artifacts
      here answer the three questions of such a device: how do I install
      it, how do I start it again after a reboot, and how do I reach the
      dashboard without a monitor.

      The network is the part that is easy to get wrong, and it is stated
      first here for that reason: the receiver announces itself with mDNS
      and with SSDP. Both are multicast. A container in a bridge network
      publishes its ports and stays invisible — the sender asks the network
      "who is there?" and the answer never leaves the bridge.

EXAMPLE:

      deploy/install.sh --dry-run          # show the plan, change nothing
      deploy/install.sh --profile nas      # install it as a service
      docker compose -f deploy/docker/docker-compose.yml up -d phairplayrust
-->

---

## 1. The three artifacts

| Path | What it is | For which device |
|---|---|---|
| `deploy/systemd/phairplayrust.service` | the unit that runs the receiver as a service | every Linux with systemd: Pi, NAS, mini PC, desktop |
| `deploy/install.sh` | builds, installs and enables the unit | every Linux with systemd |
| `deploy/docker/Dockerfile` | the image of the receiver | every host with Docker: NAS, Pi, server, Proxmox |
| `deploy/docker/docker-compose.yml` | the two container variants | the same |

The device profiles that these artifacts install live in `config/profiles/`:
`raspberry-pi`, `nas`, `docker`, `desktop`, `generic`.

---

## 2. The network comes first

The receiver is found over multicast. It announces itself with

| Purpose | Protocol | Port | Group |
|---|---|---|---|
| every protocol | mDNS | UDP 5353 | 224.0.0.251 |
| DLNA / UPnP | SSDP | UDP 1900 | 239.255.255.250 |

and it answers the senders on these TCP ports:

| Protocol | Port | Note |
|---|---|---|
| AirPlay 2 | 7000 | the RTSP control channel |
| Google Cast | 8009 | the TLS control channel |
| Miracast / WFD | 7236 | switched off on the Pi and the NAS profile |
| Samsung Smart View | 8001 | |

**Consequences for a container:** a Docker *bridge* network routes neither
mDNS nor SSDP. The container runs, the ports are published, and no phone
shows the device. The container therefore needs the network of the host:

```bash
docker run --net=host …
```

**Consequences for a firewall:** the machine must not drop the multicast
groups 224.0.0.251 and 239.255.255.250, and the ports 7000, 8009 and 7236
have to be reachable for the senders.

---

## 3. The installation as a service (systemd)

### 3.1 What the script does

`deploy/install.sh` runs six steps and prints every command it runs:

1. builds the receiver (`cargo build --release --locked`)
2. creates the service user `phairplay` — an unprivileged, system account
3. installs the binary into `/opt/phairplayrust/bin/phairplay-web`
4. installs the configuration into `/etc/phairplayrust/config.toml`
5. installs the unit into `/etc/systemd/system/phairplayrust.service`
6. reloads systemd and enables the service

### 3.2 The dry run first

```bash
deploy/install.sh --dry-run
deploy/install.sh --dry-run --profile raspberry-pi
```

The dry run prints the complete plan and **changes nothing**. It is the way
to read what the script would do without running it as root.

### 3.3 The real installation

```bash
# a Raspberry Pi
deploy/install.sh --profile raspberry-pi

# a NAS
deploy/install.sh --profile nas

# a configuration of your own
deploy/install.sh --config ./my-receiver.toml
```

The script asks for `sudo` when it is not run as root. It never installs the
receiver as root and never runs it as root — the unit runs it as the
unprivileged user `phairplay`.

### 3.4 What the service does

```bash
systemctl status phairplayrust     # is it running?
journalctl -u phairplayrust -f    # the log, live
journalctl -u phairplayrust -n 50 # the last fifty lines
systemctl restart phairplayrust   # restart
systemctl stop phairplayrust      # stop — a clean stop, the sockets are released
```

The unit starts the receiver **after the network is up**
(`After=network-online.target`) and restarts it when it fails
(`Restart=on-failure`, `RestartSec=5`). A `systemctl stop` is not fought
back: the service stays stopped until it is started again.

The stop is graceful. The unit sends `SIGTERM`, the receiver stops the
engines, releases the sockets and leaves with the status 0.

### 3.5 The dashboard

The dashboard binds to the **loopback interface** of the machine — it is a
local control panel, not a service of the network. From another computer:

```bash
ssh -L 7878:127.0.0.1:7878 phairplay@<device>
# then open http://127.0.0.1:7878 in the browser of the other computer
```

To put the panel into the network on purpose:

```bash
phairplay-web --bind 0.0.0.0
```

The program warns about it on stderr. A panel in the network lets every
device of the local network read the configuration of the receiver and
switch its protocols off.

---

## 4. The installation as a container

### 4.1 The image

```bash
# for the architecture of the machine:
docker build -f deploy/docker/Dockerfile -t phairplayrust:latest .

# for both architectures at once (the Raspberry Pi is arm64, a NAS is often amd64):
docker buildx build --platform linux/amd64,linux/arm64 \
    -f deploy/docker/Dockerfile -t phairplayrust:latest .
```

The image is a two-stage build: a Rust builder compiles the receiver, a
`debian:bookworm-slim` runs it. **There is no root in the running image** —
the receiver runs as the unprivileged user `phairplay`.

The image carries the device profiles, so a profile can be chosen without
mounting anything:

```bash
docker run --net=host phairplayrust:latest --profile nas
```

### 4.2 The compose file

```bash
cd deploy/docker
cp ../../config/profiles/generic.toml ./phairplayrust.toml
docker compose up -d phairplay
```

| Variant | Network | Discovery | Use it for |
|---|---|---|---|
| `phairplay` | `network_mode: host` | **works** | everything that is supposed to *receive* |
| `phairplay-bridge` | bridge | **does not work** | the dashboard alone, or a sender that is configured by hand |

The bridge variant starts and serves its ports — but no sender finds the
receiver, because the announcements stay inside the bridge. It has to be
asked for explicitly:

```bash
docker compose --profile bridge up -d phairplay-bridge
```

### 4.3 The health check

The image asks its own API whether the receiver still answers:

```bash
docker inspect --format '{{.State.Health.Status}}' phairplayrust
```

A container that runs with `--no-dashboard` has no API to ask and needs its
own health command:

```bash
docker run --net=host --health-cmd "true" …
```

---

## 5. The ports of the machine

| Port | Protocol | Direction |
|---|---|---|
| 7000/tcp | AirPlay 2 | in |
| 8009/tcp | Google Cast | in |
| 7236/tcp | Miracast | in (off on the Pi and the NAS profile) |
| 8001/tcp | Samsung Smart View | in |
| 6002/udp | AirPlay timing | in |
| 1900/udp | SSDP | in, multicast |
| 5353/udp | mDNS | in, multicast |
| 7878/tcp | the dashboard | in, loopback only by default |

---

## 6. What is *not* installed here

* **No hardware decoding.** The receiver parses the stream and hands the
  *encoded* frames to the platform. Decoding is the job of the platform
  (MediaCodec on Android, VideoToolbox on Apple). On a headless Linux device
  nothing decodes, so nothing is shown — the receiver serves the protocol.
* **No DRM.** Protected streams are refused with a reason.
* **No cloud.** Nothing is sent anywhere. The receiver only answers the
  senders in the local network.

The full guide — with the device list, the honest limits of every device and
the installation per device — is in
[`docs/guides/OTHER-DEVICES.md`](../docs/guides/OTHER-DEVICES.md)
and its English twin
[`docs/guides/OTHER-DEVICES.en.md`](../docs/guides/OTHER-DEVICES.en.md).
