#!/usr/bin/env bash
#
# build-release-archive.sh — the archive of a release of the headless receiver.
#
# WHAT: it packs the binary of the receiver with the files that a device
# needs: the installer, the unit of the service, the profiles of the
# devices, the licences and the readme.
#
# WHY: a binary without its configuration is a program that starts and does
# nothing, and a binary without its licences is a binary that nobody may
# use. The archive therefore carries the whole set of what a device needs —
# and the checksums, so that a user can tell whether the archive is the one
# that the release published.
#
# WHY the version of the archive is the version of the workspace: the file
# `VERSION` is the single owner of the number, and the archive is named
# after it.
#
# Usage:
#   bash scripts/build-release-archive.sh <target> <output directory>
#
#   <target>             the Rust target, for example x86_64-unknown-linux-gnu
#   <output directory>   where the archive is written
#
# Exit code 0 = the archive is built, 1 = it is not.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TARGET="${1:-}"
OUTPUT="${2:-}"

if [ -z "$TARGET" ] || [ -z "$OUTPUT" ]; then
    printf 'ERROR: the target and the output directory are needed.\n' >&2
    printf 'Usage: bash scripts/build-release-archive.sh <target> <output directory>\n' >&2
    exit 2
fi

VERSION="$(tr -d '[:space:]' < VERSION)"

# The name of the architecture, for the name of the file. The name of the
# binary differs on Windows: the program carries the suffix .exe.
# WHY the platform is part of the name: two archives of the same version
# and the same architecture — one for Linux and one for Windows — must not
# carry the same name, or the second one overwrites the first.
case "$TARGET" in
    x86_64-unknown-linux-gnu) ARCH="x86_64"; PLATFORM="linux"; BINARY_NAME="phairplay-web" ;;
    aarch64-unknown-linux-gnu) ARCH="aarch64"; PLATFORM="linux"; BINARY_NAME="phairplay-web" ;;
    x86_64-pc-windows-gnu) ARCH="x86_64"; PLATFORM="windows"; BINARY_NAME="phairplay-web.exe" ;;
    *) ARCH="$TARGET"; PLATFORM="unknown"; BINARY_NAME="phairplay-web" ;;
esac

BINARY="$REPO_ROOT/target/$TARGET/release/$BINARY_NAME"

if [ ! -f "$BINARY" ]; then
    printf 'ERROR: the binary %s does not exist.\n' "$BINARY" >&2
    printf '       Build it with: cargo build --release -p phairplay-web --target %s\n' "$TARGET" >&2
    exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
ROOT="$STAGE/phairplayrust-$VERSION-$PLATFORM-$ARCH"

mkdir -p "$ROOT/bin" "$ROOT/deploy" "$ROOT/config" "$ROOT/licenses"

install -m 0755 "$BINARY" "$ROOT/bin/$BINARY_NAME"
cp -R deploy/. "$ROOT/deploy/"
cp -R config/. "$ROOT/config/"
cp LICENSE THIRD_PARTY.md "$ROOT/licenses/"
cp -R LICENSES/. "$ROOT/licenses/"
cp README.md "$ROOT/README.md"

# The readme of the archive: what it is and how it is installed.
sed "s|BINARY_NAME|$BINARY_NAME|g" > "$ROOT/INSTALL.md" <<'TEXT'
# PhairPlayRust — the headless receiver

This archive carries the receiver of PhairPlayRust and the files that a
device needs: the installer, the unit of the service, the profiles of the
classes of devices, the configuration and the licences.

## Install

```bash
tar xzf phairplayrust-<version>-<arch>.tar.gz
cd phairplayrust-<version>-<arch>
sudo ./deploy/install.sh --no-build --prefix /opt/phairplayrust
```

The installer needs the binary at `target/release/phairplay-web` of the
checkout, or the binary of this archive — the script accepts both, and
`--no-build` says that the binary is already there.

## Run without installing

```bash
./bin/BINARY_NAME --profile generic
```

On Windows the binary is `phairplay-web.exe`.

The dashboard is served on the loopback interface. The protocols announce
the receiver over mDNS and over SSDP — **both are multicast**, and a
container in a bridge network publishes its ports and stays invisible.

## The profiles

`config/profiles/` carries the classes of devices: `raspberry-pi`, `nas`,
`docker`, `desktop` and `generic`. The profile of the device is the
starting point of the configuration; a file of your own is laid over it.

## The licences

The receiver is AGPL-3.0-or-later. `licenses/` carries the licence of the
project, the licences of the dependencies and the inventory of them in
`THIRD_PARTY.md`.
TEXT

# The archive itself. The name carries the version and the architecture.
NAME="phairplayrust-$VERSION-$PLATFORM-$ARCH"
ARCHIVE="$OUTPUT/$NAME.tar.gz"
mkdir -p "$OUTPUT"
tar czf "$ARCHIVE" -C "$STAGE" "$NAME"

# The checksums: the archive and the checksum file travel together.
( cd "$OUTPUT" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256" )

printf 'the archive   : %s\n' "$ARCHIVE"
printf 'the checksum  : %s\n' "$OUTPUT/$NAME.tar.gz.sha256"
printf 'the binary    : %s\n' "$BINARY"
printf 'the version   : %s\n' "$VERSION"
printf 'the size      : %s\n' "$(du -h "$ARCHIVE" | cut -f1)"
exit 0
