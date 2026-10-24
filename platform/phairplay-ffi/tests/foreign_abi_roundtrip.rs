//! The proof that the *binding* works, not only the Rust code.
//!
//! WHAT: The test generates the foreign bindings of the bridge from the
//! built shared library, and it runs a foreign caller — a Python script
//! that loads the generated bindings and calls the library through the C
//! ABI — against the bridge. It also generates the bindings of Kotlin and
//! Swift and checks that they carry the whole contract.
//!
//! WHY: The Rust tests prove that the conversions are right. They cannot
//! prove that the *wire format* is right: the layout of the buffer, the
//! lifting of an optional value, the width of a number, the error that has
//! to arrive as a structured failure instead of a crash. Those are the
//! bugs that only a foreign caller finds — and the native shells are
//! foreign callers.
//!
//! EXAMPLE: `cargo test -p phairplay-ffi --test foreign_abi_roundtrip`

// WHY: the module holds the helpers of every test binary of the bridge.
// The items that this binary does not need are not needed here either.
#[allow(dead_code)]
mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::Must;

/// The directory of the build, found from the path of the test binary.
fn target_directory() -> PathBuf {
    let executable = std::env::current_exe().must("the path of the test binary must be known");
    executable
        .parent()
        .and_then(Path::parent)
        .must("the test binary must live in the build directory")
        .to_path_buf()
}

/// The shared library that carries the bridge.
fn shared_library(target: &Path) -> PathBuf {
    for name in [
        "libphairplay_ffi.so",
        "libphairplay_ffi.dylib",
        "phairplay_ffi.dll",
    ] {
        let candidate = target.join(name);
        if candidate.exists() {
            return candidate;
        }
    }
    panic!("the shared library of the bridge must be built next to the test binary");
}

/// The generator of the bindings.
fn bindgen(target: &Path) -> PathBuf {
    let candidate = target.join("uniffi-bindgen");
    assert!(
        candidate.exists(),
        "the generator of the bindings must be built next to the test binary: {}",
        candidate.display()
    );
    candidate
}

/// Generates the bindings of one language.
fn generate(bindgen: &Path, library: &Path, language: &str, out: &Path) {
    let status = Command::new(bindgen)
        .arg("generate")
        .arg("--library")
        .arg(library)
        .arg("--language")
        .arg(language)
        .arg("--out-dir")
        .arg(out)
        .status()
        .must("the generator of the bindings must be startable");
    assert!(
        status.success(),
        "the bindings for `{language}` could not be generated: {status}"
    );
}

#[test]
fn the_bindings_of_every_shell_can_be_generated() {
    let target = target_directory();
    let library = shared_library(&target);
    let bindgen = bindgen(&target);
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("bindings");

    for language in ["kotlin", "swift"] {
        let directory = out.join(language);
        std::fs::create_dir_all(&directory).must("the directory must be creatable");
        generate(&bindgen, &library, language, &directory);

        let generated: Vec<PathBuf> = std::fs::read_dir(&directory)
            .must("the directory must be readable")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect();
        assert!(
            !generated.is_empty(),
            "the bindings for `{language}` must not be empty"
        );
    }

    // The Kotlin bindings must carry the whole contract of the bridge.
    let kotlin = std::fs::read_to_string(
        out.join("kotlin")
            .join("uniffi")
            .join("phairplay_ffi")
            .join("phairplay_ffi.kt"),
    )
    .must("the Kotlin bindings must be readable");
    for declaration in [
        "data class FfiAppConfig",
        "data class FfiProtocolStatus",
        "data class FfiSessionSnapshot",
        "data class FfiStreamStats",
        "data class FfiDeviceIdentity",
        "sealed class FfiException",
        "interface ReceiverEventSink",
        "interface ReceiverInterface",
    ] {
        assert!(
            kotlin.contains(declaration),
            "the Kotlin bindings must contain `{declaration}`"
        );
    }

    // And so must the Swift ones.
    let swift = std::fs::read_to_string(out.join("swift").join("phairplay_ffi.swift"))
        .must("the Swift bindings must be readable");
    for declaration in [
        "public struct FfiAppConfig",
        "public struct FfiProtocolStatus",
        "public struct FfiSessionSnapshot",
        "public struct FfiStreamStats",
        "public struct FfiDeviceIdentity",
        "public enum FfiError",
        "public enum FfiReceiverEvent",
        "public protocol ReceiverEventSink",
        "public protocol ReceiverProtocol",
    ] {
        assert!(
            swift.contains(declaration),
            "the Swift bindings must contain `{declaration}`"
        );
    }
}

#[test]
fn a_foreign_caller_round_trips_through_the_c_abi() {
    let target = target_directory();
    let library = shared_library(&target);
    let bindgen = bindgen(&target);
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("python");

    std::fs::create_dir_all(&out).must("the directory must be creatable");
    generate(&bindgen, &library, "python", &out);

    // The generated module loads the shared library from its own directory.
    std::fs::copy(
        &library,
        out.join(library.file_name().must("the library has a name")),
    )
    .must("the shared library must be copyable");
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("foreign")
        .join("roundtrip.py");
    std::fs::copy(&script, out.join("roundtrip.py")).must("the script must be copyable");

    let python = python_interpreter();
    let Some(python) = python else {
        // WHY: the roundtrip through the C ABI needs an interpreter that
        // can load the generated bindings. The proof of the binding is the
        // script above; this test runs it when the interpreter is there.
        eprintln!("no python interpreter was found: the roundtrip was not run");
        return;
    };

    let output = Command::new(&python)
        .arg(out.join("roundtrip.py"))
        .current_dir(&out)
        .output()
        .must("the python interpreter must be startable");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the roundtrip through the C ABI failed:\n{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("the roundtrip through the C ABI succeeded"),
        "the roundtrip must report its success:\n{stdout}"
    );
    println!("{stdout}");
}

/// The Python interpreter of the machine, if there is one.
fn python_interpreter() -> Option<String> {
    for candidate in ["python3", "python"] {
        let available = Command::new(candidate)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if available {
            return Some(candidate.to_string());
        }
    }
    None
}
