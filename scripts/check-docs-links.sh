#!/usr/bin/env bash
#
# check-docs-links.sh — the documentation gate of PhairPlayRust.
#
# It does two things:
#
#   1. It verifies that every internal Markdown link of the repository
#      resolves to a file that exists. A broken link in a document is a
#      defect: the reader follows a promise that the repository does not
#      keep.
#
#   2. It verifies that the numeric budgets of
#      config/performance-budgets.toml still agree with the typed mirror in
#      core/phairplay-core/src/config.rs. The documentation owns the value;
#      the Rust constant is the mirror. A difference means that one of the
#      two was changed without the other — the "single source of truth" rule
#      of docs/DOCUMENTATION_POLICY.md is broken.
#
# Usage:
#   bash scripts/check-docs-links.sh [--quiet]
#
# Exit code 0 = green, 1 = at least one broken link or one budget drift.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
if [ "${1:-}" = "--quiet" ]; then
  QUIET=1
fi

BROKEN_LINKS=0
CHECKED_LINKS=0
CHECKED_FILES=0
LINK_ERRORS=()
MIRROR_ERRORS=()
MIRROR_WARNINGS=()
MIRROR_CHECKED=0

# ── helpers ──────────────────────────────────────────────────────────────────

say() {
  if [ "$QUIET" -eq 0 ]; then
    printf '%s\n' "$*"
  fi
}

# Normalises an absolute path: removes ".", resolves "..", collapses the
# slashes. The input is expected to be absolute, the output is absolute.
normalize_path() {
  local input="$1"
  local out=""
  local part
  local IFS='/'
  local -a parts=()
  read -ra parts <<< "$input"
  for part in "${parts[@]}"; do
    case "$part" in
      ""|".") continue ;;
      "..") out="${out%/*}" ;;
      *)
        if [ -z "$out" ]; then
          out="$part"
        else
          out="${out}/${part}"
        fi
        ;;
    esac
  done
  printf '/%s' "$out"
}

# Decodes the few percent escapes that a Markdown link may carry.
decode_escapes() {
  local text="$1"
  text="${text//%20/ }"
  text="${text//%28/(}"
  text="${text//%29/)}"
  printf '%s' "$text"
}

# Reads a numeric value out of the TOML file: key "name" -> the value of
# "name = <value>" in the section that is currently open.
toml_value() {
  local file="$1"
  local section="$2"
  local key="$3"
  awk -v section="$section" -v key="$key" '
    /^[[:space:]]*\[/ {
      current = $0
      gsub(/[[:space:]\[\]]/, "", current)
      next
    }
    current == section {
      line = $0
      sub(/#.*$/, "", line)
      n = index(line, "=")
      if (n == 0) next
      name = substr(line, 1, n - 1)
      value = substr(line, n + 1)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      if (name == key) { print value; exit }
    }
  ' "$file"
}

# Reads a numeric value out of the typed Rust model: the field name and the
# expression on the right of the colon in the default implementation.
rust_value() {
  local file="$1"
  local field="$2"
  grep -E "^[[:space:]]+${field}:" "$file" | head -n 1 | sed -E 's/^[^:]+:[[:space:]]*//; s/,$//; s/[[:space:]]+//g'
}

# Evaluates a small integer expression such as "64*1024*1024" or "30_000".
evaluate_expression() {
  local expression="$1"
  expression="${expression//_/}"
  expression="${expression// /}"
  if ! [[ "$expression" =~ ^[0-9+*()\-]+$ ]]; then
    printf ''
    return 1
  fi
  printf '%s' "$((expression))"
}

# ── 1. the link check ────────────────────────────────────────────────────────

say "==> Checking the internal links of the documentation..."

while IFS= read -r file; do
  [ -n "$file" ] || continue
  CHECKED_FILES=$((CHECKED_FILES + 1))
  directory="$(dirname "$file")"

  while IFS= read -r raw; do
    [ -n "$raw" ] || continue
    target="${raw#](}"
    target="${target%)}"

    # A target in angle brackets may contain spaces.
    if [[ "$target" == "<"*">" ]]; then
      target="${target:1:${#target}-2}"
    else
      # Everything after the first blank is a title, not part of the path.
      target="${target%% *}"
    fi

    [ -n "$target" ] || continue

    # An anchor inside the same document.
    [[ "$target" == \#* ]] && continue

    # An external resource.
    case "$target" in
      http://*|https://*|ftp://*|mailto:*|tel:*|data:*|irc:*|news:*|xmpp:*|sms:*)
        continue
        ;;
    esac

    # The fragment and the query are not part of the path.
    target="${target%%#*}"
    target="${target%%\?*}"
    [ -n "$target" ] || continue

    target="$(decode_escapes "$target")"

    CHECKED_LINKS=$((CHECKED_LINKS + 1))

    if [[ "$target" == /* ]]; then
      resolved="${REPO_ROOT}${target}"
    else
      resolved="${REPO_ROOT}/${directory#./}/${target}"
    fi
    resolved="$(normalize_path "$resolved")"

    if [ ! -e "$resolved" ]; then
      BROKEN_LINKS=$((BROKEN_LINKS + 1))
      LINK_ERRORS+=("${file}: ${target}")
    fi
  done < <(grep -oE '\]\([^)]*\)' "$file" 2>/dev/null || true)
done < <(find . \( -name target -o -name .git -o -name archive -o -name node_modules \) -prune -o -name '*.md' -print | sort)

# The reference-style definitions of a Markdown file: "[id]: path".
while IFS= read -r file; do
  [ -n "$file" ] || continue
  directory="$(dirname "$file")"
  while IFS= read -r raw; do
    [ -n "$raw" ] || continue
    target="${raw#*:}"
    target="${target#"${target%%[![:space:]]*}"}"
    [ -n "$target" ] || continue
    [[ "$target" == \#* ]] && continue
    case "$target" in
      http://*|https://*|ftp://*|mailto:*|tel:*|data:*) continue ;;
    esac
    target="${target%%#*}"
    target="${target%%\?*}"
    [ -n "$target" ] || continue
    target="$(decode_escapes "$target")"
    CHECKED_LINKS=$((CHECKED_LINKS + 1))
    if [[ "$target" == /* ]]; then
      resolved="${REPO_ROOT}${target}"
    else
      resolved="${REPO_ROOT}/${directory#./}/${target}"
    fi
    resolved="$(normalize_path "$resolved")"
    if [ ! -e "$resolved" ]; then
      BROKEN_LINKS=$((BROKEN_LINKS + 1))
      LINK_ERRORS+=("${file}: ${target} (reference definition)")
    fi
  done < <(grep -oE '^\[[^]]+\]:[[:space:]]*[^[:space:]]+' "$file" 2>/dev/null || true)
done < <(find . \( -name target -o -name .git -o -name archive -o -name node_modules \) -prune -o -name '*.md' -print | sort)

if [ "${#LINK_ERRORS[@]}" -gt 0 ]; then
  printf '\n!!! %s broken link(s) found:\n' "${#LINK_ERRORS[@]}" >&2
  for error in "${LINK_ERRORS[@]}"; do
    printf '    %s\n' "$error" >&2
  done
fi

# ── 2. the mirror check ──────────────────────────────────────────────────────

say "==> Checking the documented budgets against the typed model..."

BUDGETS="config/performance-budgets.toml"
MODEL="core/phairplay-core/src/config.rs"

# The mapping: "<section>|<toml key>|<rust field>".
MAPPING=(
  "latency|first_frame_ms|first_frame_latency_ms"
  "latency|idle_timeout_ms|idle_timeout_ms"
  "throughput|minimum_fps|minimum_fps"
  "resources|max_memory_mb|max_memory_mb"
  "resources|max_cpu_percent|max_cpu_percent"
  "queues|video_queue_frames|video_queue_frames"
  "queues|video_queue_bytes|video_queue_bytes"
  "queues|audio_queue_packets|audio_queue_packets"
  "limits|max_message_bytes|max_message_bytes"
  "limits|max_header_count|max_header_count"
  "limits|max_header_bytes|max_header_bytes"
  "limits|max_pairing_attempts|max_pairing_attempts"
  "limits|max_artwork_bytes|max_artwork_bytes"
  "limits|max_photo_bytes|max_photo_bytes"
  "limits|max_media_bytes|max_media_bytes"
)

if [ ! -f "$BUDGETS" ] || [ ! -f "$MODEL" ]; then
  MIRROR_ERRORS+=("the budget file or the typed model is missing")
else
  for entry in "${MAPPING[@]}"; do
    section="${entry%%|*}"
    rest="${entry#*|}"
    key="${rest%%|*}"
    field="${rest#*|}"

    documented="$(toml_value "$BUDGETS" "$section" "$key")"
    if [ -z "$documented" ]; then
      MIRROR_ERRORS+=("${BUDGETS}: the key '${section}.${key}' does not exist")
      continue
    fi

    expression="$(rust_value "$MODEL" "$field")"
    if [ -z "$expression" ]; then
      MIRROR_WARNINGS+=("${MODEL}: the field '${field}' was not found — the mapping table of this script may be outdated")
      continue
    fi

    implemented="$(evaluate_expression "$expression")"
    if [ -z "$implemented" ]; then
      MIRROR_WARNINGS+=("${MODEL}: the expression of '${field}' ('${expression}') could not be evaluated")
      continue
    fi

    MIRROR_CHECKED=$((MIRROR_CHECKED + 1))
    if [ "$documented" != "$implemented" ]; then
      MIRROR_ERRORS+=("${section}.${key}: documented ${documented}, typed model ${implemented} (${field})")
    fi
  done
fi

if [ "${#MIRROR_WARNINGS[@]}" -gt 0 ]; then
  printf '\n--- warnings ---\n' >&2
  for warning in "${MIRROR_WARNINGS[@]}"; do
    printf '    %s\n' "$warning" >&2
  done
fi

if [ "${#MIRROR_ERRORS[@]}" -gt 0 ]; then
  printf '\n!!! %s budget drift(s) found:\n' "${#MIRROR_ERRORS[@]}" >&2
  for error in "${MIRROR_ERRORS[@]}"; do
    printf '    %s\n' "$error" >&2
  done
fi

# ── the result ───────────────────────────────────────────────────────────────

say ""
say "Markdown files scanned : ${CHECKED_FILES}"
say "Internal links checked : ${CHECKED_LINKS}"
say "Broken links           : ${BROKEN_LINKS}"
say "Budgets compared       : ${MIRROR_CHECKED}"

if [ "$BROKEN_LINKS" -gt 0 ] || [ "${#MIRROR_ERRORS[@]}" -gt 0 ]; then
  say ""
  say "FAILED: the documentation is not consistent with the repository."
  exit 1
fi

say ""
say "OK: every internal link resolves and every documented budget matches the typed model."
exit 0
