#!/usr/bin/env bash
#
# generate-bindings.sh — the generator of the foreign bindings of the bridge.
#
# It produces the Kotlin bindings of the Android shell and the Swift bindings
# of the macOS shell from the Rust sources of `platform/phairplay-ffi`.
#
# WHY: The bindings are never written by hand (FR-073). A hand-edited
# binding drifts from the Rust source, and the drift is only noticed when a
# phone crashes. This script is the one command that produces them, so that
# a regeneration produces no diff in a clean tree.
#
# WHAT it does, step by step:
#
#   1. It builds the shared library of the bridge and the `uniffi-bindgen`
#      binary of the workspace.
#   2. It generates the Swift bindings and moves the C header and the
#      module map into the include directory of the SwiftPM C target, where
#      SwiftPM expects them.
#   3. It generates the Kotlin bindings into the package directory that the
#      Gradle build of the Android shell compiles.
#
# Usage:
#   bash scripts/generate-bindings.sh [--offline] [--keep]
#
#   --offline  Builds the Rust side with `--offline`; the sandbox has no
#              network access to the registry of crates.
#   --keep     Keeps the bindings that are already there instead of
#              removing the directories first. Useful to inspect what a
#              regeneration would change.
#
# Exit code 0 = the bindings were generated, 1 = they were not.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OFFLINE=0
KEEP=0
for argument in "$@"; do
    case "$argument" in
        --offline) OFFLINE=1 ;;
        --keep) KEEP=1 ;;
        -h|--help)
            sed -n '2,30p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown argument: $argument" >&2
            echo "usage: bash scripts/generate-bindings.sh [--offline] [--keep]" >&2
            exit 1
            ;;
    esac
done

# The directory of the build of the workspace.
TARGET_DIR="$REPO_ROOT/target/debug"

# The shared library of the bridge. The name of the file depends on the
# platform: Linux has the `.so`, macOS the `.dylib`, Windows the `.dll`.
find_library() {
    for name in \
        "$TARGET_DIR/libphairplay_ffi.so" \
        "$TARGET_DIR/libphairplay_ffi.dylib" \
        "$TARGET_DIR/phairplay_ffi.dll"; do
        if [ -f "$name" ]; then
            printf '%s\n' "$name"
            return 0
        fi
    done
    return 1
}

echo "==> Building the bridge and the binding generator…"
BUILD_FLAGS=()
if [ "$OFFLINE" -eq 1 ]; then
    BUILD_FLAGS+=(--offline)
fi
if ! cargo build "${BUILD_FLAGS[@]}" -p phairplay-ffi --lib --bins; then
    echo "the bridge could not be built — the bindings were not generated" >&2
    exit 1
fi

DYLIB_PATH="$(find_library)" || {
    echo "the shared library of the bridge was not found in $TARGET_DIR" >&2
    exit 1
}
echo "    library: $DYLIB_PATH"

BINDGEN="$TARGET_DIR/uniffi-bindgen"
if [ ! -x "$BINDGEN" ]; then
    echo "the binding generator was not built: $BINDGEN" >&2
    exit 1
fi

# The targets of the shells.
SWIFT_OUT_DIR="$REPO_ROOT/apps/macos/Sources/PhairPlayFFI"
SWIFT_C_INCLUDE_DIR="$REPO_ROOT/apps/macos/Sources/PhairPlayFFIHeaders/include"
SWIFT_C_TARGET_DIR="$REPO_ROOT/apps/macos/Sources/PhairPlayFFIHeaders"
KOTLIN_OUT_DIR="$REPO_ROOT/apps/android/app/src/main/java"

if [ "$KEEP" -eq 0 ]; then
    rm -rf "$SWIFT_OUT_DIR" "$SWIFT_C_TARGET_DIR" "$KOTLIN_OUT_DIR/uniffi"
fi
mkdir -p "$SWIFT_OUT_DIR" "$SWIFT_C_INCLUDE_DIR" "$SWIFT_C_TARGET_DIR" "$KOTLIN_OUT_DIR"

echo "==> Generating the Swift bindings…"
"$BINDGEN" generate \
    --library "$DYLIB_PATH" \
    --language swift \
    --no-format \
    --out-dir "$SWIFT_OUT_DIR" || {
    echo "the Swift bindings could not be generated" >&2
    exit 1
}

# SwiftPM expects the C header of a C target in an `include` directory
# together with a module map that gives the module its name.
if [ -f "$SWIFT_OUT_DIR/phairplay_ffiFFI.h" ]; then
    mv "$SWIFT_OUT_DIR/phairplay_ffiFFI.h" "$SWIFT_C_INCLUDE_DIR/phairplay_ffiFFI.h"
fi
if [ -f "$SWIFT_OUT_DIR/phairplay_ffiFFI.modulemap" ]; then
    mv "$SWIFT_OUT_DIR/phairplay_ffiFFI.modulemap" "$SWIFT_C_INCLUDE_DIR/module.modulemap"
fi

# SwiftPM refuses a C target without a translation unit. The file is empty
# on purpose: the header of the bridge carries the declarations, and the
# translation unit only exists so that the target is valid.
cat > "$SWIFT_C_TARGET_DIR/shim.c" <<'SHIM'
/* Intentionally empty C translation unit for the SwiftPM C target that wraps
   phairplay_ffiFFI.h. The generated header carries the declarations of the
   bridge; this file exists only because SwiftPM requires a source file for
   every C target. */
SHIM

echo "==> Generating the Kotlin bindings…"
"$BINDGEN" generate \
    --library "$DYLIB_PATH" \
    --language kotlin \
    --no-format \
    --out-dir "$KOTLIN_OUT_DIR" || {
    echo "the Kotlin bindings could not be generated" >&2
    exit 1
}

# The generator must not leave an empty Kotlin package behind.
if [ -d "$KOTLIN_OUT_DIR/uniffi/phairplay_ffi" ] && [ -z "$(ls -A "$KOTLIN_OUT_DIR/uniffi/phairplay_ffi")" ]; then
    rmdir "$KOTLIN_OUT_DIR/uniffi/phairplay_ffi"
fi

echo "✓ Swift bindings: apps/macos/Sources/{PhairPlayFFI,PhairPlayFFIHeaders}"
echo "✓ Kotlin bindings: apps/android/app/src/main/java/uniffi/phairplay_ffi"
