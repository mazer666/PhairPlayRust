#!/usr/bin/env bash
#
# check-crate-boundaries.sh — the module boundary gate of PhairPlayRust.
#
# WHAT: it reads the manifests of the workspace and checks that the
# dependency direction between the crates is the one the build contract
# demands. A violation is a violation of the architecture, not a style
# question: a `formats/*` crate that depends on an `engines/*` crate means
# that a wire format knows a protocol, and the next engineer will copy
# that mistake.
#
# WHY: the direction cannot be left to a code review. Reviews look at the
# diff, not at the whole graph, and a dependency that nobody notices is
# added in a hurry. A script that reads every manifest answers the
# question "what does this crate depend on?" in one second and fails the
# build when the answer is wrong.
#
# THE RULE (docs/design/BUILD-CONTRACT.md, section 1)
#
#   The workspace has five layers. A crate may only depend on the layers
#   that are listed next to it:
#
#     apps/*      ->  platform/*, core/*
#     platform/*  ->  core/*
#     core/*      ->  (nothing)
#     engines/*   ->  core/*, formats/*
#     formats/*   ->  (nothing)
#
#   Two rules deserve their own name:
#
#     * `core/*` depends on no other crate of the workspace. The core is
#       the shared vocabulary; a vocabulary that needs a vocabulary is not
#       a vocabulary any more.
#     * `formats/*` depends on no other crate of the workspace. A format
#       is a pure transformation of bytes into bytes — no I/O, no
#       sockets, no platform calls.
#
#   `engines/*` may use `formats/*`: an AirPlay receiver needs an H.264
#   parser, and the H.264 parser must not know that AirPlay exists.
#
# HOW: the script parses the manifests directly. It does not need a
# network, a Cargo registry or a build. Every dependency that carries a
# `path = "..."` is resolved against the directory of its crate; a path
# that points into the workspace is a workspace-internal dependency and is
# checked against the matrix above. A path that points outside the
# workspace is a third-party crate and is none of this script's business.
#
#   The check covers `[dependencies]`, `[dev-dependencies]`,
#   `[build-dependencies]` and their `[target....]` variants. A test that
#   reaches from a format crate into an engine crate is the same
#   violation with a smaller blast radius.
#
#   The script also refuses a `path` entry in `[workspace.dependencies]`:
#   such an entry hands a workspace-internal crate to *every* member and
#   would make the matrix unenforceable.
#
# Usage:
#   bash scripts/check-crate-boundaries.sh [--quiet] [--list]
#
#   --quiet  only print the verdict and the violations.
#   --list   print the dependency table of the workspace.
#
# Exit code 0 = green, 1 = at least one boundary violation.
#
# Related: docs/adr/ADR-0001-cargo-workspace-and-module-boundaries.md

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

QUIET=0
LIST=0
SELF_TEST=0
for argument in "$@"; do
  case "$argument" in
    --quiet) QUIET=1 ;;
    --list) LIST=1 ;;
    --self-test) SELF_TEST=1 ;;
    -h|--help)
      sed -n '2,60p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      printf 'unknown argument: %s\n' "$argument" >&2
      exit 2
      ;;
  esac
done

VIOLATIONS=0
ROWS=0

say() {
  if [ "$QUIET" -eq 0 ]; then
    printf '%s\n' "$*"
  fi
}

fail() {
  VIOLATIONS=$((VIOLATIONS + 1))
  printf '!!! %s\n' "$*" >&2
}

# ── helpers ──────────────────────────────────────────────────────────────────

# The layer a crate directory belongs to, or the empty string when the
# directory is not part of one of the five layers.
layer_of() {
  case "$1" in
    apps/*) printf 'apps' ;;
    platform/*) printf 'platform' ;;
    core/*) printf 'core' ;;
    engines/*) printf 'engines' ;;
    formats/*) printf 'formats' ;;
    *) printf '' ;;
  esac
}

# Whether a dependency from one layer to another is allowed.
#
#   The first argument is the layer of the crate that declares the
#   dependency, the second the layer of the crate that is depended upon.
#   The function answers "yes" or "no".
dependency_allowed() {
  local from="$1"
  local to="$2"
  case "$from" in
    apps)
      case "$to" in
        platform|core) printf 'yes' ;;
        *) printf 'no' ;;
      esac
      ;;
    platform)
      case "$to" in
        core) printf 'yes' ;;
        *) printf 'no' ;;
      esac
      ;;
    core)
      printf 'no'
      ;;
    engines)
      case "$to" in
        core|formats) printf 'yes' ;;
        *) printf 'no' ;;
      esac
      ;;
    formats)
      printf 'no'
      ;;
    *)
      printf 'no'
      ;;
  esac
}

# The reason a dependency is forbidden. The gate has to explain itself:
# a red build without a sentence is a build that people learn to ignore.
forbidden_reason() {
  local from="$1"
  local to="$2"
  case "$from" in
    core)
      printf 'the core is the shared vocabulary and depends on no other crate of the workspace'
      ;;
    formats)
      printf 'a format crate is pure: bytes into bytes, no protocol, no core'
      ;;
    engines)
      case "$to" in
        platform) printf 'an engine may not depend on the platform bridge' ;;
        apps) printf 'an engine may not depend on an application' ;;
        *) printf 'an engine may only depend on the core and on the format crates' ;;
      esac
      ;;
    platform)
      case "$to" in
        engines) printf 'the platform bridge may not depend on a protocol engine' ;;
        formats) printf 'the platform bridge may not depend on a format crate' ;;
        apps) printf 'the platform bridge may not depend on an application' ;;
        *) printf 'the platform bridge may only depend on the core' ;;
      esac
      ;;
    apps)
      case "$to" in
        engines) printf 'an application may not depend on a protocol engine' ;;
        formats) printf 'an application may not depend on a format crate' ;;
        *) printf 'an application may only depend on the platform bridge and on the core' ;;
      esac
      ;;
    *)
      printf 'unknown layer'
      ;;
  esac
}

# The members of the workspace, one directory per line, relative to the
# root of the repository.
workspace_members() {
  awk '
    /^\[workspace\][ \t]*$/ { in_workspace = 1; next }
    /^\[/ { in_workspace = 0 }
    in_workspace && /^members[ \t]*=/ { grab = 1 }
    grab {
      line = $0
      while (match(line, /"[^"]*"/)) {
        print substr(line, RSTART + 1, RLENGTH - 2)
        line = substr(line, RSTART + RLENGTH)
      }
      if (index($0, "]") > 0) { grab = 0 }
    }
  ' Cargo.toml
}

# The path dependencies of one manifest, one relative path per line. Only
# the sections that carry dependencies are read, so that the `path` of a
# `[[bin]]` is not mistaken for a dependency.
manifest_path_dependencies() {
  local manifest="$1"
  awk '
    /^\[/ { section = $0 }
    section ~ /dependencies/ && match($0, /path[ \t]*=[ \t]*"[^"]*"/) {
      value = substr($0, RSTART, RLENGTH)
      sub(/^path[ \t]*=[ \t]*"/, "", value)
      sub(/"$/, "", value)
      print value
    }
  ' "$manifest"
}

# The name of a crate, read out of its manifest.
crate_name() {
  local manifest="$1"
  awk '
    /^\[package\]/ { in_package = 1; next }
    /^\[/ { in_package = 0 }
    in_package && /^name[ \t]*=/ {
      line = $0
      sub(/^name[ \t]*=[ \t]*/, "", line)
      gsub(/^[ \t]*"|"[ \t]*$/, "", line)
      print line
      exit
    }
  ' "$manifest"
}

# The absolute path of a directory, without a trailing slash.
normalize_dir() {
  local path="$1"
  local out=""
  local part
  local IFS='/'
  local -a parts=()
  read -ra parts <<< "$path"
  for part in "${parts[@]}"; do
    case "$part" in
      ""|".") continue ;;
      "..") out="${out%/*}" ;;
      *) out="${out}/${part}" ;;
    esac
  done
  printf '%s' "$out"
}

# ── the self test ────────────────────────────────────────────────────────────

# The self test proves that the gate really fails. It copies the manifests
# of the workspace into a temporary directory, injects a violation there and
# checks that the gate answers with exit code 1. The repository itself is
# never modified: the copy is what carries the violation.
self_test() {
  local sandbox
  sandbox="$(mktemp -d)"
  local failures=0

  # The copy: the root manifest, every member manifest and the gate itself.
  mkdir -p "${sandbox}/scripts"
  cp "${REPO_ROOT}/Cargo.toml" "${sandbox}/Cargo.toml"
  cp "${BASH_SOURCE[0]}" "${sandbox}/scripts/check-crate-boundaries.sh"
  while IFS= read -r member; do
    [ -n "$member" ] || continue
    mkdir -p "${sandbox}/${member}"
    cp "${REPO_ROOT}/${member}/Cargo.toml" "${sandbox}/${member}/Cargo.toml"
  done <<< "$(workspace_members)"

  printf '==> Self test of the crate boundary gate\n'
  printf '    sandbox: %s\n\n' "$sandbox"

  # The clean copy must be green.
  if bash "${sandbox}/scripts/check-crate-boundaries.sh" --quiet >/dev/null 2>&1; then
    printf '    [ ok ] the unmodified copy is green\n'
  else
    printf '    [FAIL] the unmodified copy is already red\n'
    failures=$((failures + 1))
  fi

  # Each case: the manifest to break, the anchor, the line to insert.
  run_case() {
    local label="$1"
    local manifest="$2"
    local anchor="$3"
    local line="$4"
    local copy
    copy="$(mktemp -d)"
    cp -r "${sandbox}/." "${copy}/"
    python3 - "${copy}/${manifest}" "$anchor" "$line" <<'PY'
import pathlib
import sys

target = pathlib.Path(sys.argv[1])
anchor = sys.argv[2]
line = sys.argv[3]
text = target.read_text()
if anchor not in text:
    raise SystemExit(f"anchor not found in {target}")
target.write_text(text.replace(anchor, anchor + "\n" + line, 1))
PY
    if [ $? -ne 0 ]; then
      printf '    [FAIL] %s: the manifest could not be broken\n' "$label"
      failures=$((failures + 1))
      rm -rf "$copy"
      return
    fi
    local output
    output="$(bash "${copy}/scripts/check-crate-boundaries.sh" 2>&1)"
    local status=$?
    rm -rf "$copy"
    if [ "$status" -eq 1 ]; then
      printf '    [ ok ] %s: the gate answered with exit code 1\n' "$label"
      printf '%s\n' "$output" | sed 's/^/           | /'
    else
      printf '    [FAIL] %s: the gate answered with exit code %s, expected 1\n' "$label" "$status"
      printf '%s\n' "$output" | sed 's/^/           | /'
      failures=$((failures + 1))
    fi
  }

  run_case "formats -> engines" \
    "formats/plist/Cargo.toml" \
    "thiserror = { workspace = true }" \
    'phairplay-airplay = { path = "../../engines/airplay" }'

  run_case "core -> formats" \
    "core/phairplay-core/Cargo.toml" \
    "serde = { workspace = true }" \
    'phairplay-plist = { path = "../../formats/plist" }'

  run_case "engines -> platform" \
    "engines/airplay/Cargo.toml" \
    'phairplay-core = { path = "../../core/phairplay-core" }' \
    'phairplay-ffi = { path = "../../platform/phairplay-ffi" }'

  run_case "platform -> engines" \
    "platform/phairplay-ffi/Cargo.toml" \
    "phairplay-core = { path = \"../../core/phairplay-core\" }" \
    'phairplay-airplay = { path = "../../engines/airplay" }'

  rm -rf "$sandbox"

  printf '\n'
  if [ "$failures" -gt 0 ]; then
    printf 'FAILED: %s case(s) of the self test did not behave as expected.\n' "$failures"
    return 1
  fi
  printf 'OK: the gate fails on every violation and passes on the clean tree.\n'
  return 0
}

if [ "$SELF_TEST" -eq 1 ]; then
  self_test
  exit $?
fi

# ── the check ────────────────────────────────────────────────────────────────

say "==> Checking the crate boundaries of the workspace..."

MEMBERS="$(workspace_members)"
if [ -z "$MEMBERS" ]; then
  printf '!!! the workspace member list of Cargo.toml could not be read\n' >&2
  exit 1
fi

# The layer of every member, and the directory of every member.
declare -a MEMBER_DIRS=()
declare -a MEMBER_LAYERS=()
declare -a MEMBER_NAMES=()

while IFS= read -r member; do
  [ -n "$member" ] || continue
  member="${member%/}"
  manifest="${REPO_ROOT}/${member}/Cargo.toml"
  if [ ! -f "$manifest" ]; then
    fail "${member}: the workspace member has no Cargo.toml"
    continue
  fi
  layer="$(layer_of "$member")"
  if [ -z "$layer" ]; then
    fail "${member}: the directory is not part of apps/, platform/, core/, engines/ or formats/"
    continue
  fi
  MEMBER_DIRS+=("$member")
  MEMBER_LAYERS+=("$layer")
  MEMBER_NAMES+=("$(crate_name "$manifest")")
  ROWS=$((ROWS + 1))
done <<< "$MEMBERS"

# A crate that lives in one of the five layer directories but is not a
# member of the workspace would not be checked by this gate. Such a crate
# is a hole in the boundary, so it is a finding.
while IFS= read -r manifest; do
  directory="${manifest%/Cargo.toml}"
  relative="${directory#${REPO_ROOT}/}"
  known=0
  for member in "${MEMBER_DIRS[@]:-}"; do
    [ "$member" = "$relative" ] && known=1 && break
  done
  if [ "$known" -eq 0 ]; then
    fail "${relative}: the crate is not a member of the workspace and would escape the gate"
  fi
done < <(find apps platform core engines formats -name Cargo.toml -type f 2>/dev/null | sort)

# The dependency table of the workspace.
declare -a TABLE=()

for index in "${!MEMBER_DIRS[@]}"; do
  member="${MEMBER_DIRS[$index]}"
  layer="${MEMBER_LAYERS[$index]}"
  name="${MEMBER_NAMES[$index]}"
  manifest="${REPO_ROOT}/${member}/Cargo.toml"

  while IFS= read -r relative_path; do
    [ -n "$relative_path" ] || continue
    target="$(normalize_dir "${REPO_ROOT}/${member}/${relative_path}")"
    # A dependency that points outside the workspace is a third-party
    # crate. Not this script's business.
    case "$target" in
      "${REPO_ROOT}/"*) ;;
      *)
        continue
        ;;
    esac
    target_relative="${target#${REPO_ROOT}/}"

    target_layer="$(layer_of "$target_relative")"
    if [ -z "$target_layer" ]; then
      fail "${name} (${layer}) depends on ${target_relative}, which is not part of a layer of the workspace"
      continue
    fi

    allowed="$(dependency_allowed "$layer" "$target_layer")"
    if [ "$allowed" != "yes" ]; then
      fail "${name} (${layer}) depends on ${target_relative} (${target_layer}) — $(forbidden_reason "$layer" "$target_layer")"
      continue
    fi

    TABLE+=("${name}|${layer}|${target_relative}|${target_layer}")
  done < <(manifest_path_dependencies "$manifest")
done

# A path dependency in [workspace.dependencies] hands a workspace-internal
# crate to every member of the workspace and would make the matrix above
# unenforceable.
while IFS= read -r relative_path; do
  [ -n "$relative_path" ] || continue
  fail "[workspace.dependencies] declares the path '${relative_path}' — a workspace dependency must not point into the workspace"
done < <(awk '
  /^\[workspace\.dependencies\]/ { section = $0; next }
  /^\[/ { section = $0 }
  section ~ /^\[workspace\.dependencies\]/ && /path[ \t]*=/ {
    line = $0
    sub(/^.*path[ \t]*=[ \t]*/, "", line)
    gsub(/^[ \t]*"|"[ \t]*$/, "", line)
    print line
  }
' Cargo.toml)

# ── the report ───────────────────────────────────────────────────────────────

if [ "$LIST" -eq 1 ]; then
  say ""
  say "Workspace crates:"
  for index in "${!MEMBER_DIRS[@]}"; do
    say "  ${MEMBER_LAYERS[$index]}: ${MEMBER_DIRS[$index]}  (${MEMBER_NAMES[$index]})"
  done
  say ""
  say "Allowed internal dependencies:"
  if [ "${#TABLE[@]}" -eq 0 ]; then
    say "  (none)"
  else
    for row in "${TABLE[@]}"; do
      IFS='|' read -r from from_layer to to_layer <<< "$row"
      say "  ${from_layer}: ${from}  ->  ${to_layer}: ${to}"
    done
  fi
fi

say ""
say "Crates checked      : ${ROWS}"
say "Internal edges      : ${#TABLE[@]}"
say "Violations          : ${VIOLATIONS}"

if [ "$VIOLATIONS" -gt 0 ]; then
  say ""
  say "FAILED: a crate depends on a layer it may not depend on."
  exit 1
fi

say ""
say "OK: every dependency points the way the build contract demands."
exit 0
