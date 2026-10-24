# syntax=docker/dockerfile:1
#
# PhairPlayRust — the receiver as a container image.
#
# WHAT: a two-stage build: a Rust builder that compiles the headless
#       receiver, and a slim Debian image that runs it as a non-root user.
#
# WHY: the image has to be small and it has to contain nothing that the
#      receiver does not need. A Rust builder carries the compiler, the
#      sources of the standard library and the crate cache — several
#      gigabytes that have no business in the image that runs on a NAS. The
#      second stage therefore starts from `debian:bookworm-slim` and holds
#      the binary, the trust store of the system and the device profiles.
#
#      The image runs as its own unprivileged user. A receiver that listens
#      on the local network must not be able to write into the image, and it
#      must not run as root.
#
#      THE NETWORK IS THE PART THAT IS EASY TO GET WRONG: the receiver
#      announces itself with mDNS (UDP 5353, the multicast group
#      224.0.0.251) and with SSDP (UDP 1900, the multicast group
#      239.255.255.250). A Docker *bridge* network routes neither of the
#      two, and a sender in the same network never finds the receiver — the
#      container runs, the ports are published, and no phone shows the
#      device. The container therefore needs the network of the host:
#
#          docker run --net=host …
#
#      or, in a compose file:
#
#          network_mode: host
#
#      See deploy/docker/docker-compose.yml for the two variants and
#      deploy/README.md for the whole installation.
#
# EXAMPLE:
#
#   # build for the host:
#   docker buildx build -f deploy/docker/Dockerfile -t phairplayrust:latest .
#
#   # build for both architectures at once:
#   docker buildx build --platform linux/amd64,linux/arm64 \
#       -f deploy/docker/Dockerfile -t phairplayrust:latest .

# ── Stage 1: the builder ─────────────────────────────────────────────────────
FROM rust:1-bookworm AS builder

WORKDIR /src

# WHY: the manifest and the lock file are copied first and on their own.
# The layer of the dependencies is then reused as long as no dependency
# changes, and only a change of the sources invalidates the compile.
#
# WHY: `rust-toolchain.toml` is deliberately **not** copied. The image
# carries the latest stable toolchain of the `rust:1-bookworm` tag, which is
# the channel the repository pins (`channel = "stable"`), and copying the
# file would make rustup download a second toolchain at build time.
COPY Cargo.toml Cargo.lock ./
COPY core/ core/
COPY engines/ engines/
COPY formats/ formats/
COPY platform/ platform/
# WHY: only the web application is copied. The Android and the macOS shell
# are not members of the cargo workspace and are not part of this image.
COPY apps/web/ apps/web/

ENV CARGO_TERM_COLOR=never
ENV RUSTFLAGS="-C strip=symbols"

# WHY: `--locked` refuses a build against a lock file that does not match
# the manifests. The build of an image has to be reproducible.
RUN cargo build --release --locked -p phairplay-web --bin phairplay-web

# ── Stage 2: the runtime ─────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# WHY: the Cast control plane speaks TLS. The bundle of certificates is the
# trust store of the system and belongs to every image that speaks TLS.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm --recursive --force /var/lib/apt/lists/*

# WHY: the receiver runs as its own unprivileged user. There is no root in
# the running image: a program that listens on the local network has no
# business writing into the file system of the machine it runs on.
RUN useradd --system --create-home --home-dir /home/phairplay \
        --shell /usr/sbin/nologin phairplay \
    && mkdir --parents /var/lib/phairplayrust \
    && chown phairplay:phairplay /var/lib/phairplayrust

COPY --from=builder /src/target/release/phairplay-web /usr/local/bin/phairplay-web

# WHY: the device profiles travel with the image, so that a user can pick
# one without mounting anything:
#
#     docker run --net=host phairplayrust:latest --profile nas
#
# The profiles are owned by root and are read-only for the service user.
COPY config/profiles/ /etc/phairplayrust/profiles/
# WHY: the default configuration of the container is the profile of the
# generic device. The user overrides it with `--config` or `--profile`.
COPY config/profiles/generic.toml /etc/phairplayrust/config.toml

USER phairplay
WORKDIR /var/lib/phairplayrust

# WHY: the ports of the receiver. `EXPOSE` documents them and publishes
# nothing — the ports of the host are only open with `--net=host` or with
# an explicit `ports:` in a compose file.
#
#   7000/tcp   the RTSP control channel of AirPlay
#   8009/tcp   the TLS control channel of Google Cast
#   7236/tcp   the RTSP control plane of Miracast / Wi-Fi Display
#   6002/udp   the NTP timing service of AirPlay
#   1900/udp   the SSDP multicast group of DLNA
#   5353/udp   the mDNS multicast group of every protocol
#   7878/tcp   the dashboard — on the loopback interface of the container
EXPOSE 7000/tcp 8009/tcp 7236/tcp 6002/udp 1900/udp 5353/udp 7878/tcp

# WHY: the health check asks the own API of the receiver whether it still
# answers. The dashboard binds on the loopback interface of the container,
# so the check does not need the network of the host. A receiver that runs
# with `--no-dashboard` has no API to ask — such a container needs its own
# health command:
#
#     docker run --net=host --health-cmd "true" …
#
ENV PHAIRPLAY_WEB_PORT=7878
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl --fail --silent --show-error \
        "http://127.0.0.1:${PHAIRPLAY_WEB_PORT}/api/status" || exit 1

# WHY: the entry point is the binary itself, and the default arguments name
# the configuration file of the image. Every argument of the receiver can be
# appended to `docker run`:
#
#     docker run --net=host phairplayrust:latest --profile raspberry-pi
ENTRYPOINT ["/usr/local/bin/phairplay-web"]
CMD ["--config", "/etc/phairplayrust/config.toml"]
