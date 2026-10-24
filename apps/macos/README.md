# The macOS shell of PhairPlayRust

WHAT: the macOS application of the receiver — SwiftUI for the window,
AppKit for the item in the menu bar, and the UniFFI bindings of the bridge
of `platform/phairplay-ffi`.

WHY: The shell is the native face of the receiver on the computer. It
shows the same four protocols with the same states as the dashboard of
`apps/web` and as the Android shell, it starts and stops the engines, and
it keeps running when the window is closed. It never talks to the
internet: the only thing it opens is a socket to a sender in the local
network.

## The layout

| Path | What it is |
|---|---|
| `Sources/PhairPlayFFI/` | the generated bindings — **never edited by hand** |
| `Sources/PhairPlayFFIHeaders/` | the C header of the bridge and the module map of the module `phairplay_ffiFFI` |
| `Sources/PhairPlayMacKit/` | the logic of the shell: the model, the translation of the objects of the bridge, the client, the reader of the default configuration, the texts |
| `Sources/PhairPlayMacApp/` | the application: the SwiftUI window and the AppKit item in the menu bar |
| `Sources/PhairPlayMacKit/Resources/` | the string catalogue (English and German) and the mirror of the default configuration |
| `Resources/Info.plist` | the description of the application bundle, with the permission of the local network |
| `Tests/PhairPlayMacKitTests/` | the tests of the model and of the reader of the defaults |

The kit knows nothing about the user interface. It announces the changes
of the state through a closure, and the application decides how they are
drawn. That is what makes the kit testable without a window — and it is
what lets the whole logic of the shell be built and tested on a machine
that is not a Mac.

## Building

The build needs a Swift toolchain of Swift 6 or later and the shared
library of the bridge.

```bash
# The bindings of the bridge are generated, never written by hand.
bash ../../scripts/generate-bindings.sh

# The build of the application and the tests of the kit.
swift build -c release
swift test

# The application bundle.
./make-app-bundle.sh
```

The `Package.swift` links the shared library of the bridge from
`target/debug` of the repository. The environment variable
`PHAIRPLAY_BUILD_DIR` points the build at another directory.

## The states of a protocol

The card of a protocol shows the state of its engine, and it tells the
state "switched off in the settings" apart from the state "the service is
stopped": the first one means that the protocol is not advertised at all,
the second one that it was stopped and can be started again.
