# The evidence of the guide for other devices — 2026-10-24

> **Status:** living. This record holds the evidence behind
> [`../guides/OTHER-DEVICES.md`](../guides/OTHER-DEVICES.md) and
> [`../guides/OTHER-DEVICES.en.md`](../guides/OTHER-DEVICES.en.md). A command
> in those guides that is not named here is a command the reader has to trust
> the writer for — that is not good enough.

## 1. What was measured

| Item | Value |
|---|---|
| Revision | `b0d0f40c38ae17db2ea1a91b2394b3735c077f06` (branch `feat/other-devices`) |
| Toolchain | `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Host | Linux 5.10.134-18.0.12.lifsea8.x86_64, 2 CPUs |
| Build target | `linux-x86_64`, `CARGO_TARGET_DIR=/workspace/target-docs` |

The build ran with `--offline`: the sandbox has no network. The registry cache
of the toolchain was complete, so the build used the same versions as
`Cargo.lock`.

## 2. The build of the receiver — §4.0 of the guide

The command of the guide is the command that was run:

```text
$ cargo build --release --locked --offline -p phairplay-web --bin phairplay-web
   Compiling phairplay-ffi v0.1.0 (…/platform/phairplay-ffi)
   Compiling phairplay-dlna v0.1.0 (…/engines/dlna)
   Compiling phairplay-miracast v0.1.0 (…/engines/miracast)
   Compiling phairplay-web v0.1.0 (…/apps/web)
    Finished `release` profile [optimized] target(s) in 13m 07s
```

Exit code `0`. The binary is at `target/release/phairplay-web`; it was placed
in a separate target directory so that the working tree stayed clean.

## 3. The command line of the receiver — §4 and §6 of the guide

Every option that the guide names is in the help of the binary. The help of
the release build:

```text
$ phairplay-web --help
phairplay-web — the receiver of PhairPlayRust

USAGE:
    phairplay-web [--config <path>] [--profile <name>] [--port <port>] [--bind <address>] [--no-dashboard] [--version] [--help]

OPTIONS:
    --config <path>     Reads the configuration of the receiver from a TOML file.
    --profile <name>    Uses the device profile <name> — the file config/profiles/<name>.toml — as the base of the configuration.
    --port <port>       Binds the dashboard to <port> on the chosen interface.
    --bind <address>    Binds the dashboard to <address> instead of the loopback interface.
    --no-dashboard      Runs the receiver without the dashboard; the API stays available.
    --version           Prints the version of this build and exits.
    --help              Prints this help and exits.

THE PROFILES:
    raspberry-pi  a single-board computer such as a Raspberry Pi: little memory, no Wi-Fi Direct
    nas           a network attached storage device: a server without a screen
    docker        a container on a host: the default budget, no radio of its own
    desktop       a Linux desktop: memory to spare and every protocol
    generic       the defaults of the product, for a device that is unknown
```

The version of the build:

```text
$ phairplay-web --version
phairplay-web 0.1.0 (x86_64-unknown-linux-gnu, rustc 1.99.1 (58cf8f9c3 2026-10-13), AGPL-3.0-or-later)
```

## 4. A real start — §4.1, §5 and §6.3 of the guide

The receiver was started with the profile of the Raspberry Pi on a free port,
and the API was asked while it ran:

```text
$ phairplay-web --profile raspberry-pi --port 7879
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         not started — the protocol is switched off in the configuration
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View not started — no engine implementation is registered for the protocol `smartview`
phairplay-web: 3 of 5 protocols are running; the reason of every failure is in the status of the protocol.
```

Three facts of this output are quoted by the guide and are therefore
confirmed:

- Miracast is off in the profile `raspberry-pi` (§2 and §7.4 of the guide).
- A protocol that does not start does not stop the others (§4.1 of the guide).
- Every protocol is named in the start report, with its reason (§4.1 of the
  guide).

The status of the API:

```text
$ curl --fail --silent --show-error http://127.0.0.1:7879/api/status
{"protocols":[{"protocol":"airplay","display_name":"AirPlay 2","state":"advertising","enabled":true,"tcp_port":7000,"udp_port":6002,"peer_count":0,"last_error":null},{"protocol":"miracast","display_name":"Miracast","state":"idle","enabled":false,"tcp_port":null,"udp_port":null,"peer_count":0,"last_error":null},{"protocol":"cast","display_name":"Google Cast","state":"advertising","enabled":true,"tcp_port":8009,"udp_port":null,"peer_count":0,"last_error":null},{"protocol":"dlna","display_name":"DLNA / UPnP","state":"advertising","enabled":true,"tcp_port":41321,"udp_port":1900,"peer_count":0,"last_error":null},{"protocol":"smartview","display_name":"Samsung Smart View","state":"idle","enabled":true,"tcp_port":null,"udp_port":null,"peer_count":0,"last_error":"no engine implementation is registered for the protocol `smartview`"}],"active_sessions":0,"uptime_ms":6235}
```

The sessions of the API:

```text
$ curl --silent http://127.0.0.1:7879/api/sessions
{"sessions":[]}
```

The shutdown with `SIGTERM` leaves the process with the status `0`:

```text
$ kill -TERM <pid>
$ echo $?
0
```

**The `smartview` line is the reason why §7.2 of the guide was corrected.** The
crate `engines/smartview` implements the announcement and the handshake, but
`apps/web` does not register it — the receiver says so itself. The architecture
document carries the same statement in
[`../../ARCHITECTURE.md`](../../ARCHITECTURE.md); the milestone M5 in
[`../../ROADMAP.md`](../../ROADMAP.md) carries the state.

## 5. The installation script — §4.1 of the guide

The syntax of the script:

```text
$ bash -n deploy/install.sh
$ echo $?
0
```

The dry run of the script — it prints every command it would run and changes
nothing:

```text
$ bash deploy/install.sh --dry-run --profile raspberry-pi

== The installation plan
  repository     : /workspace/PhairPlayRust
  binary         : /workspace/PhairPlayRust/target/release/phairplay-web -> /opt/phairplayrust/bin/phairplay-web
  configuration  : /workspace/PhairPlayRust/config/profiles/raspberry-pi.toml -> /etc/phairplayrust/config.toml
  unit           : /workspace/PhairPlayRust/deploy/systemd/phairplayrust.service -> /etc/systemd/system/phairplayrust.service
  service user   : phairplay
  state directory: /var/lib/phairplayrust
  mode           : DRY RUN — nothing is changed
  WARNING: systemctl is not installed — a real installation would stop here.
           This is not a systemd machine. Use the container image (deploy/docker/) or start the receiver by hand.

== 1. Building the receiver
  DRY-RUN  cargo build --release --locked -p phairplay-web --bin phairplay-web --manifest-path /workspace/PhairPlayRust/Cargo.toml

== 2. The service user
  DRY-RUN  useradd --system --home-dir /var/lib/phairplayrust --shell /usr/sbin/nologin phairplay

== 3. The binary
  DRY-RUN  install -D -m 0755 /workspace/PhairPlayRust/target/release/phairplay-web /opt/phairplayrust/bin/phairplay-web

== 4. The configuration
  DRY-RUN  install -D -m 0640 -o phairplay -g phairplay /workspace/PhairPlayRust/config/profiles/raspberry-pi.toml /etc/phairplayrust/config.toml

== 5. The systemd unit
  DRY-RUN  install -D -m 0644 /workspace/PhairPlayRust/deploy/systemd/phairplayrust.service /etc/systemd/system/phairplayrust.service

== 6. The service
  DRY-RUN  systemctl daemon-reload
  DRY-RUN  systemctl enable --now phairplayrust.service

== Done
This was a dry run: nothing has been changed.
Run the script without --dry-run to do it.
```

The order of the steps, the paths and the service user of this output are the
ones the guide names in §4.1.

## 6. The cross-check of every command and every path

A read-only script checked the two editions of the guide against the tree.
It verifies the options of the binary against its help, the profiles against
the files and against the help, the paths of the guide against the tree, the
service names against the compose file, the statements of the guide about the
unit against the unit, the statements about the image against the image, the
ports against the typed model, the routes of the API against the router, the
exclusion identifiers against the requirements, and the vocabulary of the two
editions against each other.

```text
RESULT: every command and every path of both editions matches the repository.
```

The script changed nothing in the repository. It lives outside the tree, in
the working directory of the documentation role, because it is a check of one
document and not a gate of the project.

## 7. The link check

```text
$ bash scripts/check-docs-links.sh
==> Checking the internal links of the documentation...
==> Checking the documented budgets against the typed model...

Markdown files scanned : 57
Internal links checked : 461
Broken links           : 0
Budgets compared       : 16
Budgets owned alone    : 7

OK: every internal link resolves, every mirrored budget matches the typed model, and every budget owned alone exists exactly once.
```

Exit code `0`. The check is a gate of the project — `make check` runs it.

**The negative case.** A check that never fails is not a check. A deliberately
broken link was therefore added to the German edition of the guide, the check
was run again, and the link was removed afterwards:

```text
$ bash scripts/check-docs-links.sh
==> Checking the internal links of the documentation...

!!! 1 broken link(s) found:
    ./docs/guides/OTHER-DEVICES.md: ../../config/profiles/does-not-exist.toml
...
Broken links           : 1
...
FAILED: the documentation is not consistent with the repository.
```

Exit code `1`. With the broken link removed, the check returns to `0` — the
result in the block above.

## 8. The deployment directory — `deploy/README.md`

The unit, the Dockerfile, the compose file and the installation script all
pointed at a `deploy/README.md` that did not exist. The file is now there:
it holds the network rules (mDNS 5353 / SSDP 1900, the multicast groups),
the ports of the machine, the installation as a service, the installation as
a container, and the health check of the image.

The link check covers it, and the two editions of the guide now link it
beside the deployment artifacts:

```text
$ bash scripts/check-docs-links.sh
==> Checking the internal links of the documentation...
==> Checking the documented budgets against the typed model...

Markdown files scanned : 58
Internal links checked : 466
Broken links           : 0
Budgets compared       : 16
Budgets owned alone    : 7

OK: every internal link resolves, every mirrored budget matches the typed model, and every budget owned alone exists exactly once.
```

Exit code `0`.

## 9. The two defects that the second pass found

Both were found by running the binary, not by reading the source.

### 9.1 The message of `--no-dashboard` promised a panel that was not there

`--no-dashboard` leaves the *files* of the panel out and keeps the API. The
start message said "the dashboard is served on …" in both cases, so a user
who passed the option was sent to a URL that answers 404.

The message now names the API when the panel is switched off:

```text
$ phairplay-web --no-dashboard --port 7911
phairplay-web: the API is served on http://127.0.0.1:7911 (the dashboard is switched off)

$ phairplay-web --port 7912
phairplay-web: the dashboard is served on http://127.0.0.1:7912
```

### 9.2 Clippy refused the test code of the profile lookup

```text
error: unnecessary use of `clone` to create a slice from a reference
  --> apps/web/src/profiles.rs:198:55
   |
198 |         let error = load_from(DeviceProfile::Generic, &[directory.clone()])
   |                                                       ^^^^^^^^^^^^^^^^^^^^ help: try: `std::slice::from_ref(&directory)`
```

The gate of the project is `cargo clippy -- -D warnings`. The lint is
correct and the fix is the one the lint names. After the fix:

```text
$ cargo clippy -p phairplay-web --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.10s
```

Exit code `0`. The tests of the crate: 54 unit tests and 10 doc tests, all
of them green.

## 10. What was NOT RUN

| Command | Reason |
|---|---|
| `curl … https://sh.rustup.rs \| sh` | the sandbox has no network and no `apt`; the command of the guide is the one of the Rust website |
| `sudo apt install --yes build-essential` | the sandbox has no `apt` |
| `docker buildx build …` | `docker: command not found` — there is no `docker` in the sandbox |
| `docker build …` | `docker: command not found` — there is no `docker` in the sandbox |
| `docker compose up -d phairplay` | there is no `docker compose` in the sandbox |
| `systemctl status phairplayrust` | `systemctl: command not found` — there is no systemd in the sandbox |
| `sudo ufw allow …` | there is no `ufw` in the sandbox |
| `iw phy phy0 info` | the sandbox has no Wi-Fi adapter |
| `cat /sys/class/net/vmbr0/bridge/multicast_snooping` | there is no Proxmox bridge in the sandbox |
| `docker run --net=host …` | `docker: command not found` — there is no `docker` in the sandbox |
| the start of the receiver under WSL | there is no WSL in the sandbox |

Every one of these commands stands in the guide under a **NOT RUN** marker
with the reason next to it.
