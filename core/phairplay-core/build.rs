//! Build script of the core crate.
//!
//! WHY: `TARGET` and the rustc version are not exposed to `env!` by Cargo.
//! The build info is shown in the settings screen of a shipped app, so it
//! must be compiled into the binary — a runtime `rustc --version` call would
//! not exist on a device.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=PH_TARGET={target}");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=PH_RUSTC_VERSION={version}");
}
