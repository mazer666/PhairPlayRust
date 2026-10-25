#!/usr/bin/env bash
#
# check-version-consistency.sh — the gate of the version of the workspace.
#
# WHAT: it compares the three places that carry the version of the workspace
# and it compares them with the version of the Android shell.
#
# WHY: the version lives in exactly three places — `VERSION`,
# `Cargo.toml` and `CHANGELOG.md` — and a release sets all of them in one
# change. A tree in which they differ is a tree that nobody can name: the
# tag says one version, the binary says another, and the notes of the
# release describe a third.
#
# WHY this gate exists: it did not. The release v0.2.0 was published with
# the file `VERSION` still carrying `0.1.0` — the version was never set.
# Nothing in the project noticed, because nothing compared the three
# places. A gate that exists only for the rules that somebody remembered to
# check is not a gate.
#
# The exit code is the verdict:
#
#   0  the version is consistent
#   1  the version is not consistent — the gate is red
#
# Usage:
#   bash scripts/check-version-consistency.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

FAILURES=0

say() {
    printf '%s\n' "$*"
}

fail() {
    printf '    %s\n' "$*" >&2
    FAILURES=$((FAILURES + 1))
}

# ── the four places ─────────────────────────────────────────────────────────

# The single line of the file VERSION.
version_file=""
if [ -f VERSION ]; then
    version_file="$(tr -d '[:space:]' < VERSION)"
fi

# The version of the workspace in Cargo.toml.
version_cargo=""
if [ -f Cargo.toml ]; then
    version_cargo="$(grep -m1 '^version[[:space:]]*=' Cargo.toml \
        | sed -E 's/^version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')"
fi

# The version of the Android shell.
version_android=""
if [ -f apps/android/app/build.gradle.kts ]; then
    version_android="$(grep -m1 'versionName[[:space:]]*=' apps/android/app/build.gradle.kts \
        | sed -E 's/.*versionName[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')"
fi

# The heading of the newest section of the changelog.
version_changelog=""
if [ -f CHANGELOG.md ]; then
    # WHY the first heading and not the "Unreleased": the section of the
    # newest release is the version that the tree carries.
    version_changelog="$(grep -m1 '^## \[[0-9]' CHANGELOG.md \
        | sed -E 's/^## \[([^]]+)\].*/\1/')"
fi

# ── the comparison ──────────────────────────────────────────────────────────

say "==> Checking the version of the workspace..."

if [ -z "$version_file" ]; then
    fail "VERSION does not exist or is empty"
fi
if [ -z "$version_cargo" ]; then
    fail "Cargo.toml carries no version of the workspace"
fi

say "    VERSION          : ${version_file:-<none>}"
say "    Cargo.toml       : ${version_cargo:-<none>}"
say "    Android shell    : ${version_android:-<none>}"
say "    CHANGELOG.md     : ${version_changelog:-<none>}"

if [ -n "$version_file" ] && [ -n "$version_cargo" ] && [ "$version_file" != "$version_cargo" ]; then
    fail "VERSION says ${version_file}, Cargo.toml says ${version_cargo}"
fi

if [ -n "$version_android" ] && [ -n "$version_file" ] && [ "$version_android" != "$version_file" ]; then
    fail "the Android shell says ${version_android}, VERSION says ${version_file}"
fi

# WHY the changelog is compared only when it names a release: the section of
# the tree that is not released yet is "Unreleased", and the version of a
# tree that is between two releases is the version of the last release.
if [ -n "$version_changelog" ] && [ -n "$version_file" ]; then
    if [ "$version_changelog" != "$version_file" ]; then
        fail "the newest section of the changelog is ${version_changelog}, VERSION says ${version_file}"
    fi
fi

# ── the verdict ─────────────────────────────────────────────────────────────

if [ "$FAILURES" -gt 0 ]; then
    printf '\nFAILED: the version of the workspace is not consistent.\n' >&2
    exit 1
fi

printf '\nOK: VERSION, Cargo.toml, the Android shell and the changelog carry the same version.\n'
exit 0
