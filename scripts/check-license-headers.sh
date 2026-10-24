#!/usr/bin/env bash
#
# check-license-headers.sh — the licence-header gate of PhairPlayRust.
#
# WHAT: it checks that every file of the workspace carries the licence it
# is allowed to carry, and that the files that are derived from a
# third-party work carry the attribution of that work.
#
# WHY: the whole workspace is published under the AGPL-3.0-or-later
# licence. A file that carries a different licence header is a licence
# statement about a file that does not exist — and a licence statement
# that is wrong is worse than one that is missing, because it is believed.
# The other direction is the derived work: the native Rust port of the ALAC
# reference decoder of Apple is a derived work under the Apache License
# 2.0, and the Apache License demands that the origin and the licence are
# mentioned. Both facts have to be *checked*, not remembered.
#
# THE RULES:
#
#   R1  Every member manifest of the workspace inherits the licence from
#       the workspace (`license.workspace = true`). A literal licence
#       string in a member manifest is an error (FR-001).
#   R2  A source file that carries a licence header carries one of the
#       licences the project is allowed to use: AGPL-3.0-or-later for the
#       own work, Apache-2.0 for the derived ALAC port.
#   R3  Every file that `THIRD_PARTY.md` lists as derived carries an
#       attribution header that names the origin and the licence — and so
#       does every other implementation file of the derived crate, because
#       the whole crate is a port.
#   R4  A file **outside** the derived crate does not carry the Apache
#       header: that would claim a third-party licence for work that is
#       not derived.
#   R5  `REUSE.toml` names the licence of the package.
#
# Usage:
#   bash scripts/check-license-headers.sh [--quiet]
#
# Exit code 0 = green, 1 = at least one violation.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
if [ "${1:-}" = "--quiet" ]; then
  QUIET=1
fi

say() {
  if [ "$QUIET" -eq 0 ]; then
    printf '%s\n' "$*"
  fi
}

VIOLATIONS=0
CHECKED_MANIFESTS=0
CHECKED_SOURCES=0
CHECKED_DERIVED=0
ERRORS=()

error() {
  VIOLATIONS=$((VIOLATIONS + 1))
  ERRORS+=("$*")
}

# The licences the project is allowed to use in a file header.
OWN_LICENCE="AGPL-3.0-or-later"
DERIVED_LICENCE="Apache-2.0"

# ── R1: the member manifests ────────────────────────────────────────────────
say "==> Checking the licence of the member manifests..."

while IFS= read -r manifest; do
  [ -n "$manifest" ] || continue
  # The root manifest is the owner of the licence of the workspace; it
  # carries the licence itself and inherits nothing.
  if [ "$manifest" = "./Cargo.toml" ]; then
    continue
  fi
  CHECKED_MANIFESTS=$((CHECKED_MANIFESTS + 1))
  if ! grep -qE '^[[:space:]]*license\.workspace[[:space:]]*=[[:space:]]*true[[:space:]]*$' "$manifest"; then
    error "R1: ${manifest} does not inherit the licence from the workspace (license.workspace = true)"
  fi
  if grep -qE '^[[:space:]]*license[[:space:]]*=[[:space:]]*"' "$manifest"; then
    error "R1: ${manifest} carries a literal licence string instead of inheriting it"
  fi
done < <(
  find . \
    \( -name target -o -name .git -o -name archive -o -name node_modules \) \
    -prune -o -name Cargo.toml -type f -print | sort
)

# The root manifest owns the licence of the workspace.
if ! grep -qE '^[[:space:]]*license[[:space:]]*=[[:space:]]*"AGPL-3\.0-or-later"[[:space:]]*$' Cargo.toml; then
  error "R1: Cargo.toml does not declare the workspace licence AGPL-3.0-or-later"
fi

# ── the derived files ───────────────────────────────────────────────────────
#
# The list is read out of the inventory, so that the inventory and the
# check cannot drift apart: a file that is added to the inventory becomes
# a file that has to carry the attribution, without a second edit.
DERIVED_FILES=()
while IFS= read -r path; do
  [ -n "$path" ] || continue
  DERIVED_FILES+=("$path")
done < <(
  grep -oE '^\|[[:space:]]*`[^`]+\.rs`' THIRD_PARTY.md 2>/dev/null \
    | sed -E 's/^\|[[:space:]]*`([^`]+)`.*/\1/' | sort -u
)

is_derived() {
  local candidate="$1"
  local known
  for known in "${DERIVED_FILES[@]}"; do
    if [ "$known" = "$candidate" ]; then
      return 0
    fi
  done
  return 1
}

# The crate that is the derived work. Every implementation file of it
# carries the attribution of the reference implementation.
DERIVED_CRATE="formats/alac/src"

# ── R2, R3 and R4: the headers of the source files ──────────────────────────
say "==> Checking the licence headers of the source files..."

while IFS= read -r file; do
  [ -n "$file" ] || continue
  # The paths of the inventory do not carry the "./" of the search.
  relative="${file#./}"
  CHECKED_SOURCES=$((CHECKED_SOURCES + 1))
  # The header is the beginning of the file: the first forty lines carry
  # the attribution of a derived file and the licence of the work.
  head="$(head -n 40 "$file")"

  spdx="$(printf '%s\n' "$head" | grep -oE 'SPDX-License-Identifier:[[:space:]]*"?[A-Za-z0-9.+-]+' | head -n 1 | sed -E 's/.*:[[:space:]]*"?//')"
  apache="$(printf '%s\n' "$head" | grep -ciE 'licensed under the Apache License|Apache License, Version 2\.0|SPDX-License-Identifier:[[:space:]]*"?Apache-2\.0')"
  agpl="$(printf '%s\n' "$head" | grep -ciE 'AGPL-3\.0-or-later|GNU Affero General Public License')"

  if [ -n "$spdx" ]; then
    case "$spdx" in
      "$OWN_LICENCE"|"$DERIVED_LICENCE")
        ;;
      *)
        error "R2: ${relative} carries the licence identifier '${spdx}', which the project does not use"
        ;;
    esac
  fi

  # A licence statement that is neither the own licence nor the licence of
  # a derived work is a wrong statement about the file.
  other="$(printf '%s\n' "$head" | grep -oiE 'SPDX-License-Identifier:[[:space:]]*"?[A-Za-z0-9.+-]+' \
    | sed -E 's/.*:[[:space:]]*"?//' \
    | grep -vE "^(${OWN_LICENCE}|${DERIVED_LICENCE})$" | head -n 1)"
  if [ -n "$other" ]; then
    error "R2: ${relative} carries the licence identifier '${other}', which the project does not use"
  fi

  # A test file of the derived crate is new work and carries no attribution.
  is_test_file=0
  case "$relative" in
    *_tests.rs|*/tests.rs|*/tests/*) is_test_file=1 ;;
  esac

  if is_derived "$relative"; then
    CHECKED_DERIVED=$((CHECKED_DERIVED + 1))
    if [ "$apache" -eq 0 ]; then
      error "R3: ${relative} is listed as derived in THIRD_PARTY.md but carries no Apache-2.0 attribution"
    fi
    if ! printf '%s\n' "$head" | grep -qiE 'Apple'; then
      error "R3: ${relative} is listed as derived in THIRD_PARTY.md but does not name the origin (Apple)"
    fi
  elif [ "$is_test_file" -eq 0 ] && [ "${relative#"$DERIVED_CRATE"/}" != "$relative" ]; then
    # An implementation file of the derived crate that the inventory does
    # not name yet: the attribution is still mandatory, because the crate
    # as a whole is a port.
    CHECKED_DERIVED=$((CHECKED_DERIVED + 1))
    if [ "$apache" -eq 0 ]; then
      error "R3: ${relative} belongs to the derived crate but carries no Apache-2.0 attribution"
    fi
    if ! printf '%s\n' "$head" | grep -qiE 'Apple'; then
      error "R3: ${relative} belongs to the derived crate but does not name the origin (Apple)"
    fi
  elif [ "$apache" -gt 0 ] && [ "$agpl" -eq 0 ]; then
    error "R4: ${relative} carries an Apache-2.0 header but is not part of the derived crate"
  fi
done < <(
  find . \
    \( -name target -o -name .git -o -name archive -o -name node_modules \) \
    -prune -o -name '*.rs' -type f -print | sort
)

# ── R5: the REUSE metadata ──────────────────────────────────────────────────
say "==> Checking the REUSE metadata..."

if [ ! -f REUSE.toml ]; then
  error "R5: REUSE.toml is missing"
else
  if ! grep -qE 'SPDX-License-Identifier[[:space:]]*=[[:space:]]*"AGPL-3\.0-or-later"' REUSE.toml; then
    error "R5: REUSE.toml does not declare the licence AGPL-3.0-or-later"
  fi
fi

# ── the result ──────────────────────────────────────────────────────────────
if [ "${#ERRORS[@]}" -gt 0 ]; then
  printf '\n!!! %s licence violation(s) found:\n' "${#ERRORS[@]}" >&2
  for entry in "${ERRORS[@]}"; do
    printf '    %s\n' "$entry" >&2
  done
fi

say ""
say "Member manifests checked : ${CHECKED_MANIFESTS}"
say "Source files checked     : ${CHECKED_SOURCES}"
say "Derived files checked    : ${CHECKED_DERIVED}"
say "Violations               : ${VIOLATIONS}"

if [ "$VIOLATIONS" -gt 0 ]; then
  say ""
  say "FAILED: the licence headers of the repository are not consistent."
  exit 1
fi

say ""
say "OK: every licence header of the repository is correct."
exit 0
