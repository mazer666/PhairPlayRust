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
| `Tests/PhairPlayMacKitTests/` | the tests of the kit |

The kit knows nothing about the user interface: it imports nothing but
Foundation and the bindings of the bridge. It announces the changes of the
state through a closure, and the application decides how they are drawn.
That is what makes the kit testable without a window — and it is what
lets the whole logic of the shell be built and tested on a machine that is
not a Mac.

| File of the kit | What it holds |
|---|---|
| `ReceiverModel.swift` | the phases of a protocol, the four protocols, the card of a protocol, the state of the receiver |
| `ReceiverMapping.swift` | the translation of the objects of the bridge into the model of the shell |
| `ReceiverClient.swift` | the client that talks to the bridge and announces the changes of the state |
| `ReceiverFactory.swift` | the creation of the receiver and of the client |
| `ReceiverDefaults.swift` | the reader of the file with the defaults of the receiver |
| `ReceiverStrings.swift` | the reader of the string catalogue |
| `ReceiverStatus.swift` | the four states of the receiver as a whole |
| `ReceiverPhaseStyle.swift` | the word, the shape and the tone of every phase of a protocol |
| `ShellFocus.swift` | the ring of the focus and the moves inside the grid of the cards |
| `ShellKeyboard.swift` | the keys of the shell and the answer of the shell to a key |
| `ShellShortcut.swift` | the groups of the keys that the hint at the foot of the window names |
| `ShellFormatting.swift` | the sentences of the window, built out of the catalogue |
| `ShellTypography.swift` | the sizes of the text and the measures of the layout |
| `ShellTone.swift` | the colours of the states |

| File of the application | What it holds |
|---|---|
| `PhairPlayMacApp.swift` | the entry point of the application |
| `AppDelegate.swift` | the life cycle: the client, the store and the item in the menu bar |
| `DashboardView.swift` | the window: the head, the grid of the cards, the session and the hint |
| `StatusHeaderView.swift` | the head of the window with the state of the receiver |
| `ProtocolCardView.swift` | the card of one protocol and the switch that switches it |
| `FailureBannerView.swift` | the banner of a failure of the bridge |
| `NowPlayingView.swift` | the area of the session that is running |
| `FocusFrame.swift` | the visible frame that shows the focus |
| `KeyboardCatcher.swift` | the AppKit view that reads the keys of the window |
| `ReceiverStore.swift` | the state of the receiver, prepared for SwiftUI |
| `ShellTheme.swift` | the colours of the window |
| `StatusItemController.swift` | the item in the menu bar |

## The keyboard

The window is operated with the keyboard before it is operated with the
mouse. The keys are read by `KeyboardCatcher`, and what they mean is
decided by the kit — never by a view.

| Key | What it does |
|---|---|
| `Tab` | the focus goes to the next surface |
| `Shift`+`Tab` | the focus goes to the previous surface |
| the arrow keys | the focus moves inside the grid of the four cards |
| `Space` | the protocol of the card that holds the focus is switched on or off |
| `Esc` | the banner of a failure is closed |

The ring of the focus holds the button of the banner of a failure — when
there is one — and the cards of the protocols that the bridge reports. The
shell draws its own frame around the surface that holds the focus
(`FocusFrame`), because the frame of macOS is thin and dark and is hard to
see on a dark background.

## Building

The build needs a Swift toolchain of Swift 6 or later and the shared
library of the bridge.

```bash
# The bindings of the bridge are generated, never written by hand.
bash ../../scripts/generate-bindings.sh

# The build of the application and the tests of the kit.
swift build -c release
swift test
```

The `Package.swift` links the shared library of the bridge from
`target/debug` of the repository. The environment variable
`PHAIRPLAY_BUILD_DIR` points the build at another directory.

The target of the application belongs to the package only on macOS: the
window is written in SwiftUI and the item in the menu bar in AppKit, and
both frameworks exist only on a Mac. The kit and its tests are built and
tested on every system, so that the logic of the shell — the order of the
focus, the handling of the keys, the wording of the states and the sizes
of the text — is checked on every machine and not only on the machine of
the developer who happens to own one.

```bash
# On Linux, this builds the kit and runs the tests of the kit.
swift test

# On macOS, this builds the application as well.
swift build -c release
```

## The states of a protocol

The card of a protocol shows the state of its engine, and it tells the
state "switched off in the settings" apart from the state "the service is
stopped": the first one means that the protocol is not advertised at all,
the second one that it was stopped and can be started again.
