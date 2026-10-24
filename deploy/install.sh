#!/usr/bin/env bash
#
# install.sh — install the headless receiver of PhairPlayRust as a service.
#
# WHAT: it builds the receiver, installs it under a prefix, installs the
#       configuration and the systemd unit, and enables the service.
#
# WHY: the steps of an installation are the same on every device — a
#      Raspberry Pi, a NAS, a mini PC — and every one of them is a step that
#      can be done wrong. The script does the steps in the order in which
#      they depend on each other, with the paths that the systemd unit
#      expects, and it prints every command it runs. A user who does not
#      want to run the script can read the list of the commands in the
#      output of `--dry-run` and copy them.
#
#      The service runs as its own unprivileged user. The script never
#      installs the receiver as root and never runs the receiver as root.
#
# EXAMPLE:
#
#   # show what would happen, without changing anything:
#   deploy/install.sh --dry-run
#
#   # install the Raspberry Pi profile as a service:
#   deploy/install.sh --profile raspberry-pi
#
#   # install without building again, with another prefix:
#   deploy/install.sh --no-build --prefix /usr/local
#
# Exit code 0 = the receiver is installed and the unit is enabled.

set -euo pipefail

# ── The defaults of the installation ─────────────────────────────────────────
# WHY: the paths are the paths of deploy/systemd/phairplayrust.service. The
# unit and the script have to agree, otherwise the service starts a binary
# that is not there.
PREFIX_DEFAULT="/opt/phairplayrust"
CONFIG_DIR_DEFAULT="/etc/phairplayrust"
UNIT_PATH_DEFAULT="/etc/systemd/system/phairplayrust.service"
USER_DEFAULT="phairplay"
STATE_DIR_DEFAULT="/var/lib/phairplayrust"
SERVICE_NAME="phairplayrust.service"

PREFIX="$PREFIX_DEFAULT"
CONFIG_DIR="$CONFIG_DIR_DEFAULT"
UNIT_PATH="$UNIT_PATH_DEFAULT"
SERVICE_USER="$USER_DEFAULT"
STATE_DIR="$STATE_DIR_DEFAULT"
CONFIG_SOURCE=""
NO_BUILD=0
NO_ENABLE=0
DRY_RUN=0

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── The output of the script ─────────────────────────────────────────────────
say() {
    printf '%s\n' "$*"
}

say_step() {
    printf '\n== %s\n' "$*"
}

# Runs a command — or only prints it, when the run is a dry run.
#
# WHY: the dry run has to show *exactly* what the script would do, including
# the arguments of the privileged commands. Printing the command line and
# running it are therefore the same code path.
run() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '  DRY-RUN  %s\n' "$*"
        return 0
    fi
    printf '  RUN      %s\n' "$*"
    "$@"
}

# The same, for a command that is written as a string.
run_sh() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '  DRY-RUN  %s\n' "$1"
        return 0
    fi
    printf '  RUN      %s\n' "$1"
    sh -c "$1"
}

usage() {
    cat <<'EOF'
install.sh — install the headless receiver of PhairPlayRust as a service.

Usage:
  deploy/install.sh [options]

What it does:
  1. builds the receiver (cargo build --release --locked)
  2. creates the service user, if it does not exist yet
  3. installs the binary under the prefix
  4. installs the configuration file
  5. installs the systemd unit
  6. reloads systemd and enables the service

Options:
  --prefix <dir>       the directory the receiver is installed in
                       (default: /opt/phairplayrust)
  --config-dir <dir>   the directory the configuration is installed in
                       (default: /etc/phairplayrust)
  --unit <file>        the path the systemd unit is installed to
                       (default: /etc/systemd/system/phairplayrust.service)
  --user <name>        the unprivileged user the service runs as
                       (default: phairplay)
  --config <file>      the configuration file to install
                       (default: config/profiles/generic.toml)
  --profile <name>     a device profile of config/profiles to install, for
                       example raspberry-pi, nas, docker, desktop, generic
  --no-build           do not build the binary, install the one that is built
  --no-enable          install everything, but do not enable or start the unit
  --dry-run            print every command and change nothing
  -h, --help           print this text and leave

Examples:
  deploy/install.sh --dry-run
  deploy/install.sh --profile raspberry-pi
  deploy/install.sh --config ./my-receiver.toml --prefix /usr/local
EOF
}

# ── The arguments ───────────────────────────────────────────────────────────
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            [ "$#" -ge 2 ] || { say "ERROR: --prefix needs a directory."; exit 2; }
            PREFIX="$2"
            shift 2
            ;;
        --config-dir)
            [ "$#" -ge 2 ] || { say "ERROR: --config-dir needs a directory."; exit 2; }
            CONFIG_DIR="$2"
            shift 2
            ;;
        --unit)
            [ "$#" -ge 2 ] || { say "ERROR: --unit needs a file."; exit 2; }
            UNIT_PATH="$2"
            shift 2
            ;;
        --user)
            [ "$#" -ge 2 ] || { say "ERROR: --user needs a name."; exit 2; }
            SERVICE_USER="$2"
            shift 2
            ;;
        --config)
            [ "$#" -ge 2 ] || { say "ERROR: --config needs a file."; exit 2; }
            CONFIG_SOURCE="$2"
            shift 2
            ;;
        --profile)
            [ "$#" -ge 2 ] || { say "ERROR: --profile needs a name."; exit 2; }
            CONFIG_SOURCE="config/profiles/$2.toml"
            shift 2
            ;;
        --no-build)
            NO_BUILD=1
            shift
            ;;
        --no-enable)
            NO_ENABLE=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            say "ERROR: the argument \"$1\" is not known."
            say "Try 'deploy/install.sh --help'."
            exit 2
            ;;
    esac
done

# The configuration that is installed: an explicit file, a profile of
# config/profiles, or the generic profile.
if [ -z "$CONFIG_SOURCE" ]; then
    CONFIG_SOURCE="config/profiles/generic.toml"
fi
case "$CONFIG_SOURCE" in
    /*) ;;
    *) CONFIG_SOURCE="$REPO_ROOT/$CONFIG_SOURCE" ;;
esac

BINARY_SOURCE="$REPO_ROOT/target/release/phairplay-web"
BINARY_TARGET="$PREFIX/bin/phairplay-web"
CONFIG_TARGET="$CONFIG_DIR/config.toml"
UNIT_SOURCE="$REPO_ROOT/deploy/systemd/phairplayrust.service"

# ── The privileges ───────────────────────────────────────────────────────────
# WHY: the steps that touch /opt, /etc and systemd need root. The script
# asks for it with sudo and does not run the receiver as root.
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        SUDO="sudo"
    elif [ "$DRY_RUN" -eq 1 ]; then
        say "  WARNING: sudo is not installed — a real installation would stop here."
    else
        say "ERROR: the installation needs root, and sudo is not installed."
        say "Run the script as root, or install sudo."
        exit 1
    fi
fi

# ── The checks before the first change ───────────────────────────────────────
say_step "The installation plan"
say "  repository     : $REPO_ROOT"
say "  binary         : $BINARY_SOURCE -> $BINARY_TARGET"
say "  configuration  : $CONFIG_SOURCE -> $CONFIG_TARGET"
say "  unit           : $UNIT_SOURCE -> $UNIT_PATH"
say "  service user   : $SERVICE_USER"
say "  state directory: $STATE_DIR"
if [ "$DRY_RUN" -eq 1 ]; then
    say "  mode           : DRY RUN — nothing is changed"
fi

# A missing prerequisite is an error — but only in a real run. A dry run has
# to show the whole plan even on a machine that lacks one of the tools, and
# it has to say where the real run would stop.
#
# WHY: the dry run is the plan of the installation. A plan that stops at the
# first missing tool is not a plan.
require_tool() {
    local what="$1"
    local tool="$2"
    local hint="$3"
    if command -v "$tool" >/dev/null 2>&1; then
        return 0
    fi
    if [ "$DRY_RUN" -eq 1 ]; then
        say "  WARNING: $what is not installed — a real installation would stop here."
        say "           $hint"
        return 0
    fi
    say "ERROR: $what is not installed."
    say "$hint"
    exit 1
}

require_file() {
    local what="$1"
    local path="$2"
    local hint="$3"
    if [ -f "$path" ]; then
        return 0
    fi
    if [ "$DRY_RUN" -eq 1 ]; then
        say "  WARNING: $what ($path) does not exist — a real installation would stop here."
        say "           $hint"
        return 0
    fi
    say "ERROR: $what ($path) does not exist."
    say "$hint"
    exit 1
}

if [ "$NO_BUILD" -eq 0 ]; then
    require_tool "cargo" cargo "Install the Rust toolchain (https://rustup.rs)."
else
    require_file "the binary" "$BINARY_SOURCE" \
        "Build it first, or drop the --no-build option."
fi

require_file "the configuration" "$CONFIG_SOURCE" \
    "The device profiles live in config/profiles/."

require_file "the unit" "$UNIT_SOURCE" \
    "The unit lives in deploy/systemd/."

if [ "$NO_ENABLE" -eq 0 ]; then
    require_tool "systemctl" systemctl \
        "This is not a systemd machine. Use the container image (deploy/docker/) or start the receiver by hand."
fi

# ── 1. The build ─────────────────────────────────────────────────────────────
if [ "$NO_BUILD" -eq 0 ]; then
    say_step "1. Building the receiver"
    run cargo build --release --locked -p phairplay-web --bin phairplay-web \
        --manifest-path "$REPO_ROOT/Cargo.toml"
else
    say_step "1. Building the receiver — skipped (--no-build)"
fi

# ── 2. The service user ─────────────────────────────────────────────────────
say_step "2. The service user"
if id "$SERVICE_USER" >/dev/null 2>&1; then
    say "  the user $SERVICE_USER exists already"
else
    run $SUDO useradd --system --home-dir "$STATE_DIR" \
        --shell /usr/sbin/nologin "$SERVICE_USER"
fi

# ── 3. The binary ────────────────────────────────────────────────────────────
say_step "3. The binary"
run $SUDO install -D -m 0755 "$BINARY_SOURCE" "$BINARY_TARGET"

# ── 4. The configuration ─────────────────────────────────────────────────────
say_step "4. The configuration"
run $SUDO install -D -m 0640 -o "$SERVICE_USER" -g "$SERVICE_USER" \
    "$CONFIG_SOURCE" "$CONFIG_TARGET"

# ── 5. The unit ──────────────────────────────────────────────────────────────
say_step "5. The systemd unit"
run $SUDO install -D -m 0644 "$UNIT_SOURCE" "$UNIT_PATH"

# ── 6. The service ───────────────────────────────────────────────────────────
say_step "6. The service"
if [ "$NO_ENABLE" -eq 1 ]; then
    say "  skipped (--no-enable)"
else
    run $SUDO systemctl daemon-reload
    run $SUDO systemctl enable --now "$SERVICE_NAME"
fi

# ── What the user does next ──────────────────────────────────────────────────
say_step "Done"
if [ "$DRY_RUN" -eq 1 ]; then
    say "This was a dry run: nothing has been changed."
    say "Run the script without --dry-run to do it."
else
    say "The receiver is installed as $SERVICE_NAME."
    say ""
    say "  the logs       : journalctl -u phairplayrust -f"
    say "  the status     : systemctl status phairplayrust"
    say "  the stop       : systemctl stop phairplayrust"
    say "  the dashboard  : http://127.0.0.1:7878 — on the loopback interface"
    say "                   of the machine. From another computer, use an SSH"
    say "                   port forwarding:"
    say "                     ssh -L 7878:127.0.0.1:7878 $SERVICE_USER@<device>"
    say ""
    say "The receiver announces itself with mDNS (UDP 5353) and with SSDP"
    say "(UDP 1900). Both are multicast: the ports 7000, 8009 and 7236 have"
    say "to be open for the senders, and the firewall of the machine must"
    say "not drop the multicast groups 224.0.0.251 and 239.255.255.250."
    say "A container needs the network of the host (--net=host) for the same"
    say "reason — see deploy/README.md."
fi

exit 0
