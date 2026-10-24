// swift-tools-version: 6.0
// The package of the macOS shell of PhairPlayRust.
//
// WHAT: the package that builds the shell: the wrapper of the generated
// bindings, the logic of the shell and the application itself.
//
// WHY: The bindings are generated, and the module of the C header has to
// have the name that the generated Swift file imports (`phairplay_ffiFFI`).
// The target of the header therefore carries that name, and the target of
// the generated Swift code carries the name of the product.
//
// The language mode is Swift 5 for every target. The strict concurrency
// of Swift 6 would ask for an actor around the whole state of the shell;
// the state of the shell lives on the main queue, which is the pattern of
// the AppKit lifecycle, and the language mode of Swift 5 says exactly
// that without the ceremony.
//
// EXAMPLE: `swift build -c release` builds the application.

import Foundation
import PackageDescription

// The root of the repository and the directory of the build of the Rust
// side. The environment variable lets a build outside the repository point
// at another build directory.
let environment = ProcessInfo.processInfo.environment
let repositoryRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .path
let buildDirectory = environment["PHAIRPLAY_BUILD_DIR"] ?? "\(repositoryRoot)/target/debug"

// The flags that link the shared library of the bridge.
//
// WHY: The run path is part of the binary, so that the application finds
// the library without an environment variable. The path of the build
// machine is the only way to do that without a bundle of its own; an
// installation of the product replaces it with the path of the
// installation.
let ffiLinkerFlags: [String] = [
    "-L\(buildDirectory)",
    "-lphairplay_ffi",
    "-Xlinker",
    "-rpath",
    "-Xlinker",
    buildDirectory,
]

let package = Package(
    name: "PhairPlayMac",
    // The language of the catalogue of the shell. The package carries a
    // German catalogue as well; the language of the system decides which
    // one is used.
    defaultLocalization: "en",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "PhairPlayMacKit",
            targets: ["PhairPlayMacKit"]
        ),
        .executable(
            name: "PhairPlayMacApp",
            targets: ["PhairPlayMacApp"]
        ),
    ],
    targets: [
        // The C header of the bridge. The module map inside the directory
        // gives the module the name that the generated Swift file imports.
        .target(
            name: "phairplay_ffiFFI",
            path: "Sources/PhairPlayFFIHeaders",
            publicHeadersPath: "include"
        ),
        // The generated Swift bindings. The file is never edited by hand.
        .target(
            name: "PhairPlayFFI",
            dependencies: ["phairplay_ffiFFI"],
            path: "Sources/PhairPlayFFI",
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ],
            linkerSettings: [
                .unsafeFlags(ffiLinkerFlags)
            ]
        ),
        // The logic of the shell: the model, the translation of the objects
        // of the bridge, the client and the texts.
        .target(
            name: "PhairPlayMacKit",
            dependencies: ["PhairPlayFFI"],
            path: "Sources/PhairPlayMacKit",
            resources: [
                .process("Resources/en.lproj"),
                .process("Resources/de.lproj"),
                .copy("Resources/receiver-defaults.properties"),
            ],
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        ),
        // The application: SwiftUI for the window, AppKit for the item in
        // the menu bar and for the life cycle of the application.
        .executableTarget(
            name: "PhairPlayMacApp",
            dependencies: ["PhairPlayMacKit"],
            path: "Sources/PhairPlayMacApp",
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        ),
        .testTarget(
            name: "PhairPlayMacKitTests",
            dependencies: ["PhairPlayMacKit"],
            path: "Tests/PhairPlayMacKitTests",
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        ),
    ]
)
