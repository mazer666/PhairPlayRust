#!/usr/bin/env bash
#
# audit-licenses.sh — the licence gate of PhairPlayRust.
#
# WHAT: it runs the pinned cargo-deny against `deny.toml` and reports what
# the tool found.
#
# WHY: the licence of the components that the receiver links is part of the
# trust that the users place in it. The gate makes an unknown or an
# incompatible licence a build failure instead of a legal accident.
#
# WHY THE VERSION OF THE TOOL IS PINNED: cargo-deny reads the same
# configuration differently from one version to the next. The spellings of
# the GNU licences are the example: `AGPL-3.0-or-later` is the licence that
# the crates of this workspace carry, and a cargo-deny that does not know
# that spelling refuses to load the configuration at all — or, worse, loads
# it and reports the crates of the project itself as a licence violation.
# A configuration that depends on the version of the tool is a
# configuration that differs from machine to machine, so the version is
# part of the configuration and a machine with another version says
# **NOT RUN** with the reason.
#
# THE EXIT CODES ARE THE CONTRACT:
#
#   0  the gate is green — cargo-deny read `deny.toml` and found nothing
#   1  the gate is red   — cargo-deny found a violation
#   2  NOT RUN           — cargo-deny is missing, or it has another version
#
# A NOT RUN is never reported as a green gate. The caller of this script
# has to treat the exit code 2 as "the proof did not happen".
#
# Usage:
#   bash scripts/audit-licenses.sh [--check-only]
#
#   --check-only   only verify that the pinned cargo-deny is installed and
#                  that it reads the configuration; do not run the audit.
#                  The continuous integration uses it to turn a missing tool
#                  into a readable message before the long run starts.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# ── THE PIN ─────────────────────────────────────────────────────────────────
#
# The version of cargo-deny that reads `deny.toml` of this project.
# `deny.toml` states the same version in its header; the two must agree.
CARGO_DENY_REQUIRED_VERSION="0.20.2"

CHECK_ONLY=0
if [ "${1:-}" = "--check-only" ]; then
  CHECK_ONLY=1
fi

say() { printf '%s\n' "$*"; }
fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

if [ ! -f "$REPO_ROOT/deny.toml" ]; then
  fail "ERROR: deny.toml is missing — the licence policy of the project is not configured."
fi

# ── the tool ────────────────────────────────────────────────────────────────
if ! command -v cargo-deny >/dev/null 2>&1 && ! cargo deny --version >/dev/null 2>&1; then
  say "NOT RUN: cargo-deny is not installed on this machine."
  say "         The licence configuration of the project was NOT checked in this run."
  say "         Install the pinned version with:"
  say "           cargo install cargo-deny --version ${CARGO_DENY_REQUIRED_VERSION} --locked"
  exit 2
fi

INSTALLED_VERSION="$(cargo deny --version 2>/dev/null | sed -E 's/^[^0-9]*([0-9]+\.[0-9]+\.[0-9]+).*/\1/' | head -n 1)"
if [ -z "$INSTALLED_VERSION" ]; then
  say "NOT RUN: the version of cargo-deny could not be read on this machine."
  say "         The licence configuration of the project was NOT checked in this run."
  exit 2
fi

if [ "$INSTALLED_VERSION" != "$CARGO_DENY_REQUIRED_VERSION" ]; then
  say "NOT RUN: this machine carries cargo-deny ${INSTALLED_VERSION}, the project pins ${CARGO_DENY_REQUIRED_VERSION}."
  say "         The spellings of the GNU licences in deny.toml differ between the versions of the tool,"
  say "         so a run with another version would not read the configuration of the project."
  say "         The licence configuration of the project was NOT checked in this run."
  say "         Install the pinned version with:"
  say "           cargo install cargo-deny --version ${CARGO_DENY_REQUIRED_VERSION} --locked"
  exit 2
fi

say "==> cargo-deny ${INSTALLED_VERSION} (the pinned version) reads deny.toml."

if [ "$CHECK_ONLY" -eq 1 ]; then
  say "OK: the pinned cargo-deny is installed. The audit itself was not run (--check-only)."
  exit 0
fi

# ── the audit ───────────────────────────────────────────────────────────────
#
# The three sections that are configured are run: the licences, the bans and
# the sources. The **advisories** are not part of this check: they need the
# database of the advisories, and the tool that reads that database is
# `cargo audit` — it is the job `audit` of the continuous integration, and
# a check that needs a database that another job owns is a check that
# fails for the wrong reason.
#
# The crates of the workspace are fetched first: on a machine that was built
# from scratch the registry is empty, and the audit would then report every
# crate as "unknown" and fail — a failure that says nothing about the
# licences of the project.
#
# WHY `--allow unnecessary-skip`: the crates of the workspace are skipped by
# the check of the wildcards, because they depend on each other by path and
# a path carries no version. The warning of the tool says that the skip is
# unnecessary — it only knows that the crate has a single version, not that
# the skip is what suppresses the wildcard.
if ! cargo fetch --locked >/dev/null 2>&1; then
  say "NOT RUN: the crates of the workspace could not be fetched (no network, or the lock file is out of date)."
  say "         The licence configuration of the project was NOT checked in this run."
  exit 2
fi

set +e
cargo deny --all-features check --allow unnecessary-skip licenses bans sources 2>&1
DENY_EXIT=$?
set -e

if [ "$DENY_EXIT" -eq 0 ]; then
  say ""
  say "OK: cargo-deny found no violation of the licence policy of the project."
  exit 0
fi

say ""
say "ERROR: cargo-deny found a violation of the licence policy of the project." >&2
exit 1
