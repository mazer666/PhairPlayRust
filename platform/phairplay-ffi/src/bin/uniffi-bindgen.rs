//! The generator of the foreign bindings.
//!
//! WHY: The bindings of Kotlin and Swift are generated, never written by
//! hand. A hand-edited binding drifts from the Rust source, and the drift
//! is only noticed when a phone crashes. This binary is the one command
//! that produces them.
//!
//! EXAMPLE:
//!
//! ```text
//! cargo run -p phairplay-ffi --bin uniffi-bindgen -- \
//!     generate --library target/debug/libphairplay_ffi.so \
//!     --language swift --out-dir ../apps/macos/Sources/PhairPlayFFI
//! ```

fn main() {
    uniffi::uniffi_bindgen_main()
}
