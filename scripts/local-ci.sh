#!/usr/bin/env bash
#
# local-ci.sh — the gate of PhairPlayRust.
#
# WHAT: it runs every check that a change has to pass, in the order of the
# cost: the cheap structural checks first, the compiler and the tests last.
#
# WHY: the gate has to be runnable on the machine of a contributor without
# a network and without a GitHub runner, and it has to be the *same* gate
# that the continuous integration runs. A gate that exists only in the
# continuous integration is a gate that the contributor does not see before
# the push.
#
# THE EXIT CODE IS THE VERDICT:
#
#   0  the gate is green
#   1  a check found a defect — the gate is red
#   2  a check could not be run (NOT RUN) — the reason is named, and the
#      gate does **not** claim to be green
#
# A step that cannot be run in this environment is **NOT RUN** with the
# exact reason. It is never reported as a success (the rule of
# `docs/acceptance/ACCEPTANCE_TEMPLATE.md`, section 9).
#
# Usage:
#   bash scripts/local-ci.sh [--fast]
#   bash scripts/local-ci.sh --self-test
#
#   --fast       skip the tests of the workspace (the slow part). The gate is
#                then a *partial* gate and says so.
#   --self-test  prove that the gate is a gate: every rule is violated on
#                purpose, the gate has to go red, the violation is undone and
#                the gate has to go green again. The self-test changes the
#                working tree for a few seconds and restores every file it
#                touched — a trap makes sure of that.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# The tools of Rust live in `~/.cargo/bin`. The path of a non-interactive
# shell does not always carry it, and a gate that stops with "cargo: command
# not found" is a gate that was not run.
if [ -d "$HOME/.cargo/bin" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

FAST=0
SELF_TEST=0
for argument in "$@"; do
  case "$argument" in
    --fast) FAST=1 ;;
    --self-test) SELF_TEST=1 ;;
    *)
      printf 'ERROR: the argument "%s" is not known to this script.\n' "$argument" >&2
      printf 'Usage: bash scripts/local-ci.sh [--fast] [--self-test]\n' >&2
      exit 2
      ;;
  esac
done

# ── the helpers of the gate ──────────────────────────────────────────────────

heading() {
  printf '\n==> %s\n' "$*"
}

# Runs a step. The first argument is the name of the step, the second the
# command. A step that fails is a defect (exit code 1 of the step); a step
# that cannot be run is NOT RUN (exit code 2 of the step).
run_step() {
  local name="$1"
  shift
  heading "$name"
  "$@"
  local status=$?
  if [ "$status" -eq 0 ]; then
    printf '    OK: %s\n' "$name"
  elif [ "$status" -eq 2 ]; then
    printf '    NOT RUN: %s\n' "$name" >&2
    NOT_RUN=$((NOT_RUN + 1))
  else
    printf '    FAILED: %s (exit code %s)\n' "$name" "$status" >&2
    FAILURES=$((FAILURES + 1))
  fi
  return 0
}

# Runs a check script. A script that is not in the repository is a missing
# proof, not a defect of the product: the step says NOT RUN with the
# reason. A script that is there and that reports a violation is a defect:
# the gate goes red.
run_check_script() {
  local name="$1"
  local script="$2"
  if [ ! -f "$script" ]; then
    heading "$name"
    printf '    NOT RUN: %s is not in the repository. The check did not run.\n' "$script" >&2
    NOT_RUN=$((NOT_RUN + 1))
    return 0
  fi
  run_step "$name" bash "$script"
}

# ═══════════════════════════════════════════════════════════════════════════
# The self-test of the gate
# ═══════════════════════════════════════════════════════════════════════════
#
# WHY: a gate that has never refused anything is not a gate. The self-test
# breaks every rule on purpose, proves that the gate goes red, undoes the
# break and proves that the gate goes green again. The output of the self
# test is the evidence that the gate in the continuous integration means
# something.
#
# The self-test works on the working tree and restores every file it
# touched. A trap restores the files even when the script is interrupted.

# The directory the self-test keeps the original files in.
SELF_TEST_BACKUP=""
# The files the self-test changed.
SELF_TEST_TOUCHED=()

# Restores every file the self-test changed.
restore_files() {
  local file
  for file in "${SELF_TEST_TOUCHED[@]:-}"; do
    [ -n "$file" ] || continue
    if [ -f "$SELF_TEST_BACKUP/$(basename "$file")" ]; then
      cp -p "$SELF_TEST_BACKUP/$(basename "$file")" "$file" 2>/dev/null || true
    else
      rm -f "$file" 2>/dev/null || true
    fi
  done
  SELF_TEST_TOUCHED=()
}

# Remembers a file before it is changed.
remember() {
  local file="$1"
  mkdir -p "$SELF_TEST_BACKUP"
  cp -p "$file" "$SELF_TEST_BACKUP/$(basename "$file")" 2>/dev/null || true
  SELF_TEST_TOUCHED+=("$file")
}

# Runs a command and prints its exit code.
run_and_report() {
  local label="$1"
  shift
  "$@" >/dev/null 2>&1
  local status=$?
  printf '    %s -> exit code %s\n' "$label" "$status"
  return $status
}

self_test() {
  SELF_TEST_BACKUP="$(mktemp -d)"
  trap 'restore_files; rm -rf "$SELF_TEST_BACKUP"' EXIT INT TERM

  local failures=0
  local checked=0

  printf '==> The self-test of the gate of PhairPlayRust\n'
  printf '    Every rule is broken on purpose, the gate has to go red, the\n'
  printf '    break is undone and the gate has to go green again.\n\n'

  # ── 1. the lint: an `unwrap()` in the code ─────────────────────────────────
  printf -- '--- 1. the lint of the workspace: an unwrap() in the code\n'
  local victim="core/phairplay-core/src/identity.rs"
  remember "$victim"
  printf '\n%s\n' "/// A deliberate violation of the lint, added by the self-test." >> "$victim"
  printf '%s\n' "pub fn the_self_test_breaks_the_lint() -> u32 { Some(1).unwrap() }" >> "$victim"
  if run_and_report "clippy with an unwrap()" cargo clippy --workspace --all-targets -- -D warnings; then
    printf '    FAILED: the lint did not notice the unwrap().\n' >&2
    failures=$((failures + 1))
  else
    printf '    OK: the lint refused the unwrap().\n'
  fi
  restore_files
  checked=$((checked + 1))
  if run_and_report "clippy after the repair" cargo clippy --workspace --all-targets -- -D warnings; then
    printf '    OK: the gate is green again.\n'
  else
    printf '    FAILED: the gate is still red after the repair.\n' >&2
    failures=$((failures + 1))
  fi
  printf '\n'

  # ── 2. the length of the source files ──────────────────────────────────────
  printf -- '--- 2. the length of the source files: a file above the hard limit\n'
  local long_file="core/phairplay-core/src/self_test_long_file.rs"
  remember "$long_file"
  : > "$long_file"
  local line_number=0
  while [ "$line_number" -lt 600 ]; do
    printf '%s\n' "// a line of a file that is longer than the hard limit allows." >> "$long_file"
    line_number=$((line_number + 1))
  done
  if run_and_report "the file-size check with a long file" bash scripts/check-file-size.sh; then
    printf '    FAILED: the check did not notice the long file.\n' >&2
    failures=$((failures + 1))
  else
    printf '    OK: the check refused the long file.\n'
  fi
  restore_files
  checked=$((checked + 1))
  if run_and_report "the file-size check after the repair" bash scripts/check-file-size.sh; then
    printf '    OK: the gate is green again.\n'
  else
    printf '    FAILED: the gate is still red after the repair.\n' >&2
    failures=$((failures + 1))
  fi
  printf '\n'

  # ── 3. the licence headers ─────────────────────────────────────────────────
  printf -- '--- 3. the licence headers: a literal licence in a member manifest\n'
  local manifest="core/phairplay-core/Cargo.toml"
  remember "$manifest"
  sed -i 's|^license\.workspace = true$|license = "MIT"|' "$manifest"
  if run_and_report "the licence-header check with a literal licence" bash scripts/check-license-headers.sh; then
    printf '    FAILED: the check did not notice the literal licence.\n' >&2
    failures=$((failures + 1))
  else
    printf '    OK: the check refused the literal licence.\n'
  fi
  restore_files
  checked=$((checked + 1))
  if run_and_report "the licence-header check after the repair" bash scripts/check-license-headers.sh; then
    printf '    OK: the gate is green again.\n'
  else
    printf '    FAILED: the gate is still red after the repair.\n' >&2
    failures=$((failures + 1))
  fi
  printf '\n'

  # ── 4. the links of the documentation ──────────────────────────────────────
  printf -- '--- 4. the links of the documentation: a link to a file that does not exist\n'
  local document="docs/audit/self-test-document.md"
  remember "$document"
  printf '%s\n' "# The self-test" "" "[a link that does not resolve](./self-test-file-that-does-not-exist.md)" > "$document"
  if run_and_report "the link check with a broken link" bash scripts/check-docs-links.sh; then
    printf '    FAILED: the check did not notice the broken link.\n' >&2
    failures=$((failures + 1))
  else
    printf '    OK: the check refused the broken link.\n'
  fi
  restore_files
  checked=$((checked + 1))
  if run_and_report "the link check after the repair" bash scripts/check-docs-links.sh; then
    printf '    OK: the gate is green again.\n'
  else
    printf '    FAILED: the gate is still red after the repair.\n' >&2
    failures=$((failures + 1))
  fi
  printf '\n'

  # ── the verdict ────────────────────────────────────────────────────────────
  printf '==> The self-test of the gate\n'
  printf '    rules proven   : %s\n' "$checked"
  printf '    failed rules   : %s\n' "$failures"
  if [ "$failures" -gt 0 ]; then
    printf '\nFAILED: a rule of the gate did not refuse its violation.\n' >&2
    return 1
  fi
  printf '\nOK: every rule of the gate refused its violation and was green again after the repair.\n'
  return 0
}

# ── the gate itself ─────────────────────────────────────────────────────────

if [ "$SELF_TEST" -eq 1 ]; then
  self_test
  exit $?
fi

FAILURES=0
NOT_RUN=0

# ── 1. the structure of the repository ──────────────────────────────────────
run_check_script "the crate boundaries" scripts/check-crate-boundaries.sh
run_check_script "the length of the source files" scripts/check-file-size.sh
run_check_script "the licence headers" scripts/check-license-headers.sh
run_check_script "the links of the documentation" scripts/check-docs-links.sh
run_check_script "the version of the workspace" scripts/check-version-consistency.sh

# ── 2. the licences of the dependencies ─────────────────────────────────────
run_step "the licence audit (cargo-deny)" bash scripts/audit-licenses.sh

# ── 3. the compiler ─────────────────────────────────────────────────────────
run_step "the format of the sources" cargo fmt --all -- --check
run_step "the lint of the workspace" cargo clippy --workspace --all-targets -- -D warnings
run_step "the documentation of the workspace" cargo doc --workspace --no-deps

# ── 4. the tests ────────────────────────────────────────────────────────────
if [ "$FAST" -eq 1 ]; then
  heading "the tests of the workspace"
  printf '    NOT RUN: the tests were skipped (--fast). The gate is a partial gate.\n' >&2
  NOT_RUN=$((NOT_RUN + 1))
else
  # WHY: the round-trip test of the bridge loads the shared library and
  # calls the binding generator. `cargo test` builds neither — it builds the
  # test harness of the library, not its cdylib.
  run_step "the shared library of the bridge" cargo build -p phairplay-ffi
  run_step "the tests of the workspace" cargo test --workspace
fi

# ── the verdict ─────────────────────────────────────────────────────────────
printf '\n==> The gate of PhairPlayRust\n'
printf '    failed checks : %s\n' "$FAILURES"
printf '    NOT RUN       : %s\n' "$NOT_RUN"

if [ "$FAILURES" -gt 0 ]; then
  printf '\nFAILED: the gate of the project is red.\n' >&2
  exit 1
fi

if [ "$NOT_RUN" -gt 0 ]; then
  printf '\nNOT RUN: the gate did not run every check. The reasons are above.\n' >&2
  printf '        A check that did not run is never reported as a passed one.\n' >&2
  exit 2
fi

printf '\nOK: the gate of the project is green.\n'
exit 0
