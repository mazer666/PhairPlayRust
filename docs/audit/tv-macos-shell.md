# The macOS shell of PhairPlayRust — audit

> **Status:** the logic of the shell is **green** (101 tests, 0 failures). The
> window itself — the SwiftUI part — is **NOT RUN**: this machine has no
> macOS SDK, so SwiftUI cannot be compiled here. Every statement below
> carries the command that produced it.

## 1. What was measured

| Item | Value |
|---|---|
| Command | `swift test` (in `apps/macos`) |
| Revision | `7cd6fe0413c0229bc49d3bfe3f86a1df51e817ec` (branch `feat/other-devices`, working tree) |
| Toolchain | `Swift version 6.0.3 (swift-6.0.3-RELEASE)`, target `x86_64-unknown-linux-gnu` |
| Host | `Linux 5.10.134-18.0.12.lifsea8.x86_64 x86_64` |
| Build | clean — `rm -rf .build && swift test` (2 m 54 s) |
| Raw output | [`tv-macos-shell-swift-test.log`](tv-macos-shell-swift-test.log) |

The suite was run twice: once incrementally and once from a clean build
directory. Both runs report the same result. The log in this directory is
the clean run.

### The result

```text
Test Suite 'All tests' passed at 2026-10-24 08:56:47.576
	 Executed 101 tests, with 0 failures (0 unexpected) in 0.534 (0.534) seconds
```

| Suite | Tests | What it proves |
|---|---|---|
| `ReceiverDefaultsTests` | 7 | the reader of the defaults file accepts the file of the repository and refuses a broken one |
| `ReceiverModelTests` | 17 | the translation of the objects of the bridge into the model of the shell |
| `ReceiverStringsTests` | 6 | the reader of the catalogue answers the text, answers the key when a text is missing and fills the placeholders from left to right |
| `ShellFocusTests` | 12 | the ring of the focus: the order of the tab key, the moves of the arrow keys, the edge of the grid |
| `ShellKeyboardTests` | 10 | the mapping of the key codes of macOS onto the keys of the shell, and the answer of the shell to every key |
| `ShellShortcutTests` | 8 | every key of the shell belongs to exactly one group of shortcuts, and every group has a name and a sentence |
| `ShellStatusTests` | 12 | the four states of the receiver, with a word, a shape and a tone each |
| `ShellToneTests` | 7 | every tone is a colour of six digits, no two tones share a colour, every phase and every state has a tone |
| `ShellFormattingTests` | 13 | the sentences of the window come out of the catalogue; the text is big enough to be read from a distance |
| `ShellCatalogueTests` | 6 | the English and the German catalogue carry the same keys; every key that the sources use exists in both; no view writes a text of its own |
| `ShellKitPurityTests` | 3 | the kit imports nothing but Foundation and the bindings; no file of the shell is above 400 lines |

## 2. The files

### The kit — `Sources/PhairPlayMacKit/` (Foundation only, builds on Linux)

| File | Lines | What it is |
|---|---|---|
| `ReceiverModel.swift` | 310 | the phases of a protocol, the four protocols, the card of a protocol, the state of the receiver |
| `ReceiverMapping.swift` | 235 | the translation of the DTOs of the bridge into the model of the shell |
| `ReceiverClient.swift` | 343 | the client that talks to the bridge and announces the changes of the state |
| `ReceiverDefaults.swift` | 226 | the reader of the file with the defaults of the receiver |
| `ReceiverFactory.swift` | 67 | the creation of the receiver and of the client |
| `ReceiverStrings.swift` | 49 | the reader of the string catalogue |
| `ReceiverStatus.swift` | 117 | the four states of the receiver as a whole — **new in this change** |
| `ReceiverPhaseStyle.swift` | 84 | the word, the shape and the tone of every phase — **new in this change** |
| `ShellFocus.swift` | 195 | the ring of the focus and the moves inside the grid of the cards — **new in this change** |
| `ShellKeyboard.swift` | 214 | the keys of the shell, the key codes of macOS and the answer of the shell to a key — **new in this change** |
| `ShellShortcut.swift` | 132 | the groups of the keys that the hint names — **new in this change** |
| `ShellFormatting.swift` | 177 | the sentences of the window, built out of the catalogue — **new in this change** |
| `ShellTypography.swift` | 103 | the sizes of the text and the measures of the layout — **new in this change** |
| `ShellTone.swift` | 65 | the colours of the states — **new in this change** |
| `Resources/en.lproj/Localizable.strings` | 90 | the English texts |
| `Resources/de.lproj/Localizable.strings` | 88 | the German texts |
| `Resources/receiver-defaults.properties` | — | the mirror of the defaults of the core |

### The application — `Sources/PhairPlayMacApp/` (SwiftUI + AppKit, macOS only)

| File | Lines | What it is |
|---|---|---|
| `DashboardView.swift` | 248 | the window: the head, the grid of the cards, the banner, the session and the hint — **rebuilt in this change** |
| `ProtocolCardView.swift` | 163 | the card of one protocol and the switch that switches it — **rebuilt in this change** |
| `FailureBannerView.swift` | 107 | the banner of a failure and its drawn button — **new in this change** |
| `NowPlayingView.swift` | 110 | the area of the session that is running |
| `StatusItemController.swift` | 111 | the item in the menu bar |
| `KeyboardCatcher.swift` | 92 | the AppKit view that reads the keys of the window — **new in this change** |
| `FocusFrame.swift` | 61 | the visible frame that shows the focus — **new in this change** |
| `ReceiverStore.swift` | 62 | the state of the receiver, prepared for SwiftUI — **new in this change** |
| `ShellTheme.swift` | 60 | the colours of the window — **new in this change** |
| `StatusHeaderView.swift` | 59 | the head of the window — **new in this change** |
| `AppDelegate.swift` | 63 | the life cycle: the client, the store and the item in the menu bar |
| `PhairPlayMacApp.swift` | 34 | the entry point of the application |

### The tests — `Tests/PhairPlayMacKitTests/`

| File | Lines | What it proves |
|---|---|---|
| `ShellCatalogueTests.swift` | 276 | the parity of the two catalogues, the completeness of the keys, the absence of literals in the views — **new** |
| `ShellKitPurityTests.swift` | 125 | the kit stays free of SwiftUI; no file is above 400 lines — **new** |
| `ShellShortcutTests.swift` | 91 | the groups of the keys and their texts — **new** |
| `ShellToneTests.swift` | 68 | the colours of the states — **new** |
| `ShellFocusTests.swift` | 126 | the ring of the focus — **new** |
| `ShellFormattingTests.swift` | 123 | the sentences and the sizes — **new** |
| `ShellKeyboardTests.swift` | 132 | the keys and their answers — **new** |
| `ShellStatusTests.swift` | 112 | the states of the receiver — **new** |
| `ReceiverStringsTests.swift` | 61 | the reader of the catalogue — **new** |
| `ReceiverModelTests.swift` | 235 | the model of the shell |
| `ReceiverDefaultsTests.swift` | 111 | the reader of the defaults |

## 3. The target structure of `Package.swift`

`Package.swift` builds two different things under two different conditions,
and the split is the reason the logic of the shell is testable at all.

| Target | Condition | Imports | Built on Linux |
|---|---|---|---|
| `phairplay_ffiFFI` | always | the C header of the bridge | yes |
| `PhairPlayFFI` | always | the generated bindings | yes |
| `PhairPlayMacKit` | always | **Foundation only** | yes |
| `PhairPlayMacKitTests` | always | XCTest | yes |
| `PhairPlayMacApp` | `#if os(macOS)` | SwiftUI, AppKit, Combine | **no** |

The proof that the application target is not part of the package on this
machine:

```text
$ swift package describe --type json
phairplay_ffiFFI         library      Sources/PhairPlayFFIHeaders
PhairPlayMacKitTests     test         Tests/PhairPlayMacKitTests
PhairPlayMacKit          library      Sources/PhairPlayMacKit
PhairPlayFFI             library      Sources/PhairPlayFFI
```

The rule that keeps the kit free of the frameworks of the user interface is
not a convention but a **test** — `ShellKitPurityTests` reads every file of
the kit and fails if one of them imports `SwiftUI`, `AppKit`, `Combine`,
`UIKit` or `WebKit`:

```swift
func testTheKitImportsNothingButFoundationAndTheBindings() {
    for file in ShellKitPurityTests.sources(of: "PhairPlayMacKit") {
        for module in ShellKitPurityTests.imports(of: file) {
            XCTAssertTrue(
                ShellKitPurityTests.allowedImports.contains(module),
                "the file \(file.lastPathComponent) imports \(module)"
            )
        }
    }
}
```

## 4. The keyboard

The shell reads the keys itself. macOS moves the focus with the tab key only
when the full keyboard access is switched on, and it offers no key at all
for the space bar in a window that is not a form — `KeyboardCatcher` is
therefore an AppKit view that becomes the first responder of the window and
hands the keys over to `ShellKeyboard`, which is a pure function in the
kit.

| Key | What it does | Where it is decided | Where it is tested |
|---|---|---|---|
| `Tab` | the focus goes to the next surface | `ShellKeyboard.handle` | `ShellKeyboardTests.testTheTabKeyMovesTheFocusToTheNextCard` |
| `Shift`+`Tab` | the focus goes to the previous surface | `ShellKeyboard.handle` | `ShellKeyboardTests.testTheShiftTabKeyMovesTheFocusToThePreviousCard` |
| the arrow keys | the focus moves inside the grid of the four cards | `FocusRing.move` | `ShellFocusTests.testTheArrowKeysMoveInsideTheGrid` |
| `Space` | the protocol of the card that holds the focus is switched | `ShellKeyboard.handle` | `ShellKeyboardTests.testTheSpaceKeySwitchesTheFocusedProtocol` |
| `Esc` | the banner of a failure is closed | `ShellKeyboard.handle` | `ShellKeyboardTests.testTheEscapeKeyClosesTheBanner` |

The key codes of the virtual keyboard of macOS are mapped in the kit
(`ShellKeyMap`), not in the application, because they are the only thing
that AppKit hands over that is the same on every keyboard layout.

The hint at the foot of the window is **not** a literal: it is built out of
the groups of shortcuts of the kit, and the groups carry the keys. A key
that the shell gains later shows up in the hint without a second place to
change — and `ShellShortcutTests.testEveryKeyOfTheShellBelongsToExactlyOneGroup`
fails the moment a key is added without a group.

## 5. The focus frame

The window draws its **own** frame instead of leaning on the ring of macOS,
because the ring of the system is thin and dark and is nearly invisible on
a dark background. The frame is three points wide (`ShellTypography.focusRingWidth`),
in the colour `4DA3FF` (`ShellTheme.focus`), with a soft light around it.

* `FocusFrame.swift` — the modifier, plus `View.focusFrame(_:cornerRadius:)`.
* `DashboardView.swift` — the `@FocusState private var focus: FocusTarget?`
  and the `.focused($focus, equals: …)` binding on every surface of the ring.
* `ProtocolCardView.swift` and `FailureBannerView.swift` — the frame itself.
* `WithoutFocusEffect` — the ring of the system is switched off on macOS 14
  and later, so that only one frame is visible.

Three defects were found and fixed in this change:

1. **The banner of a failure could never hold the focus.** The ring of the
   focus holds `.dismissFailure` when there is a failure, but the banner
   carried no `.focusable()` and no `.focused(…)` binding — the escape key
   and the tab key had nothing to walk to. The banner now carries the same
   binding as the cards.
2. **The button of the banner took the focus away from the grid.** A button
   of the system takes the focus of the window when it is clicked, and the
   focus of this window belongs to the surfaces of the ring. The button is
   now drawn (`DismissButton`), like the switch of a card, and it keeps the
   traits of a button for the screen reader.
3. **The card that holds the focus could be thrown away.** The grid was a
   `LazyVGrid`, and a lazy grid throws a card away when it leaves the
   window — the user would press the space bar and nothing would happen.
   The grid is built out of plain rows now, with the same number of
   columns as the ring of the focus (`FocusRing.columns`), so that the
   arrow keys move the focus exactly where the eye expects it.

## 6. The texts

Every text the user can read comes out of the string catalogue of the kit
(`en.lproj` and `de.lproj`). The rules are enforced by tests, not by
convention:

* `ShellCatalogueTests.testTheEnglishAndTheGermanCatalogueCarryTheSameKeys`
  — both catalogues carry the same keys (73 keys each at the time of this
  audit).
* `ShellCatalogueTests.testEveryKeyThatTheSourcesUseExistsInBothCatalogues`
  — the test scans the sources of the kit and of the application for every
  `ReceiverStrings.text("…")` and looks the key up in both catalogues.
* `ShellCatalogueTests.testNoViewWritesATextOfItsOwn` — the test scans the
  sources of the application for a `Text("…")` with a literal in it.

The texts that this change added:

| Key | English | German |
|---|---|---|
| `shortcut.tab.title` / `.detail` | Tab / next card | Tab / nächste Karte |
| `shortcut.shiftTab.title` / `.detail` | Shift+Tab / previous card | Shift+Tab / vorherige Karte |
| `shortcut.arrows.title` / `.detail` | Arrow keys / move in the grid | Pfeiltasten / im Raster bewegen |
| `shortcut.space.title` / `.detail` | Space / switch the protocol | Leertaste / Protokoll umschalten |
| `shortcut.escape.title` / `.detail` | Esc / close this message | Esc / Meldung schließen |

The sentence `dashboard.keyboard` was removed: the hint is now built out of
the groups above, so that it cannot name a key that the shell does not
answer to.

`ReceiverStatus` needs texts of its own (`status.running`,
`status.starting`, `status.stopped`, `status.error` and their
`.detail`), and they were already in both catalogues — the test
`ShellStatusTests.testTheWordOfAStateIsNoKey` proves that none of the four
states falls back to its own key. `ShellTypography` needs no texts: it holds
sizes and measures, and `ShellFormattingTests.testTheTextIsBigEnoughToBeReadFromADistance`
keeps the promise that the text is readable from a distance.

## 7. What is NOT RUN — and why

| What | State | Exact reason |
|---|---|---|
| The build of `PhairPlayMacApp` | **NOT RUN** | SwiftUI is not available under Linux. There is no macOS SDK on this machine, and the frameworks `SwiftUI`, `AppKit` and `Combine` exist only on macOS. |
| The type check of the views | **NOT RUN** | The same reason: the compiler cannot resolve `import SwiftUI` without the SDK of macOS. |
| The window on a screen | **NOT RUN** | The same reason: there is no window server and no display on this machine. |
| The syntax of the views | **RUN — green** | `swiftc -parse` only parses; it does not resolve the imports. It proves that the sources of the application are syntactically correct — nothing more. |
| The tests of the kit | **RUN — green** | 101 tests, 0 failures (section 1). |
| The application bundle | **NOT RUN** | The README of the shell pointed at a `make-app-bundle.sh` that does not exist in the repository. The reference was removed instead of inventing a script that cannot be verified here. Packaging the application is an open point (section 9). |

The syntax check of the application:

```text
$ swiftc -parse Sources/PhairPlayMacApp/*.swift
exit code: 0
```

**What this check is not:** it is not a type check. A wrong argument name, a
wrong modifier or a wrong enum case in the views would still be found only
on a Mac.

### The instructions for a Mac

```bash
cd apps/macos

# 1. The bindings of the bridge (they are generated, never written by hand).
bash ../../scripts/generate-bindings.sh

# 2. The shared library of the bridge, if it is not there yet.
cd ../..
cargo build -p phairplay-ffi
#    The package links the library from target/debug; PHAIRPLAY_BUILD_DIR
#    points the build at another directory.

# 3. The tests of the kit — they run on a Mac as well.
cd apps/macos && swift test

# 4. The application.
swift build -c release
swift run PhairPlayMacApp
```

The first build of the application on a Mac needs the Swift 6 toolchain
(Xcode 16 or newer) and a macOS 13 or newer system.

## 8. What the change does not touch

* The bridge (`platform/phairplay-ffi`) and the generated bindings are
  untouched — `Sources/PhairPlayFFI/phairplay_ffi.swift` is not edited by
  hand.
* The Rust side of the product is untouched.
* The shell of Android (`apps/android`) and the dashboard of the web
  (`apps/web`) are untouched by this change. The **states** are the same on
  the three platforms — the five slots "disabled", "transition",
  "advertising", "streaming" and "error" — but the **colours** are not: each
  platform carries its own palette (open point 5).

## 9. The open points — a decision of the owner is needed

1. **The bundle of the application.** The README pointed at a
   `make-app-bundle.sh` that does not exist. Either the script is written
   (it can only be verified on a Mac) or the README keeps describing the
   plain `swift build`. **Decision needed.**
2. **The behaviour of the space bar on a card that is switched off.** The
   kit toggles the protocol of the card that holds the focus; the card is
   drawn with the switch of the system replaced by a drawn one. If the
   product wants the space bar to open the card of a protocol that is
   switched off instead of switching it on, the rule has to be written into
   `ShellKeyboard` and covered by a test.
3. **The return key.** The shell answers to `Tab`, the arrow keys, the space
   bar and the escape key. A return key that activates the surface that
   holds the focus is the usual expectation of a native application; it is
   not implemented, because the key map of the kit is a contract and adding
   a key means a new group of shortcuts and a new text in both catalogues.
   **Decision needed.**
4. **The size of the window.** The minimum of the window is 760 × 620
   points. On a screen with a smaller resolution the content scrolls. The
   grid of the cards is not lazy any more, so the focus cannot get lost —
   but a card that is out of the view does not show its frame. A
   `ScrollViewReader` that scrolls to the card that holds the focus would
   close that gap; it is not implemented, because it cannot be verified
   without a screen.
5. **One palette for the three platforms — or three palettes.** The five
   states are the same everywhere, the colours are not. The measured
   values:

   | Slot | macOS (`ShellTone`) | Android TV (`PhairPlayColors`) | Web (`theme.css`) |
   |---|---|---|---|
   | disabled / idle | `#5B6672` | `#9AA5B1` | `--text-muted` `#a7aec0` |
   | transition | `#8A6D1F` | `#FBBF24` | `--warning` `#fbbf24` |
   | advertising | `#1F8A5B` | `#4ADE80` | `--accent` `#a78bfa` |
   | streaming | `#0F7C8A` | `#22D3EE` | `--success` `#3ecf8e` |
   | failure | `#B3261E` | `#FB7185` | `--danger` `#ff7a7a` |
   | frame of the focus | `#4DA3FF` | `#60A5FA` | `--focus-ring` `#c4b5fd` |

   The doc comments of `ShellTone` and of `ShellTheme` claimed that the
   numbers of the kit **are** the colours of the Android shell. They are
   not, and the claim was corrected in this change — the kit holds the
   slots, each platform holds its own colours. **Decision needed:** one
   palette for the three platforms, or three palettes on purpose.
6. **The dark appearance.** The surfaces of the window use the colours of
   the system (`controlBackgroundColor`, `windowBackgroundColor`), so the
   shell follows the appearance of macOS. The tone of a state is chosen for
   a dark room — a muted grey for a state that does nothing, a red only for
   a real failure — and a muted grey on a light background has a lower
   contrast than the same grey on a dark one. **Decision needed:** force
   the dark appearance (the shell of the television uses a dark surface
   everywhere), or check the contrast of every tone against a light
   background as well.
