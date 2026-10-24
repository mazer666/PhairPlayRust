# The Android TV shell of PhairPlayRust — audit

> **Status:** the Kotlin of the shell **compiles** (`:app:compileDebugKotlin`,
> green) and the **tests of the shell are green** (39 tests, 0 failures,
> 12 of them for the order of the focus). The **APK is NOT RUN** in this
> sandbox: the cross-build of the native bridge needs the NDK of the
> version that the Android Gradle plugin wants, and it is not installed
> here — the exact message is in section 9. The task
> `:app:compileGoogletvDebugKotlin` of the assignment **does not exist**,
> because the module declares no product flavour — the exact message is in
> section 3.
>
> Every statement below carries the command that produced it.

## 1. What was measured

| Item | Value |
|---|---|
| Command | `./gradlew :app:compileDebugKotlin --offline --rerun-tasks` (in `apps/android`) |
| Tests | `./gradlew :app:testDebugUnitTest --offline --rerun-tasks` |
| Lint | `./gradlew :app:lintDebug` |
| Revision | `7cd6fe0413c0229bc49d3bfe3f86a1df51e817ec` (branch `feat/other-devices`, working tree) |
| Toolchain | Gradle 8.13, Kotlin (Android Gradle plugin), `compileSdk 35`, `minSdk 33` |
| Compose | BOM `2024.10.01` (compose-ui 1.7.5), Material 3 |
| SDK | `/workspace/.home/tools/android-sdk` (see `apps/android/local.properties`) |
| JDK | `/workspace/.home/tools/jdk-17.0.21+9` (`JAVA_HOME` has to be set — `java` is not on the `PATH` of this machine) |
| Raw output | [`tv-android-shell-compile.log`](tv-android-shell-compile.log), [`tv-android-shell-test.log`](tv-android-shell-test.log), [`tv-android-shell-lint.log`](tv-android-shell-lint.log), [`tv-android-shell-googletv-task.log`](tv-android-shell-googletv-task.log) |

> **Note about the machine:** the sandbox has two processors and three
> gigabytes of memory, and several agents build at the same time. One test
> run was aborted in the middle by the operating system
> (`peer closed connection without sending complete message body`) and a
> second one failed with a stale output directory
> (`java.nio.file.NoSuchFileException: .../mergeDebugResources/merged.dir/values-bn`).
> Both are the symptoms of the memory pressure of the machine and not of
> the source: the run that is documented here is the run after both
> incidents, with a fresh compilation of the module.

## 2. The result of the build

```text
$ cd apps/android && ./gradlew :app:compileDebugKotlin --offline --rerun-tasks
...
> Task :app:compileDebugKotlin
BUILD SUCCESSFUL in 2m 59s
15 actionable tasks: 15 executed
```

The compile of the module is green — including the manifest merge, the
merge of the resources and the three locales:

```text
> Task :app:processDebugMainManifest
> Task :app:mergeDebugResources
> Task :app:processDebugResources
> Task :app:compileDebugKotlin
```

## 3. The task of the assignment `:app:compileGoogletvDebugKotlin`

**NOT RUN — the task does not exist in this repository.** The module
`apps/android/app` declares no `productFlavors`, so Gradle has no variant
named `googletv`:

```text
$ cd apps/android && ./gradlew :app:compileGoogletvDebugKotlin --offline
FAILURE: Build failed with an exception.

* What went wrong:
Cannot locate tasks that match ':app:compileGoogletvDebugKotlin' as task 'compileGoogletvDebugKotlin' not found in project ':app'.
```

The variant that exists — and that carries exactly the same Kotlin — is
`debug`; it is the variant that was built (section 2). The shell is one
application for the telephone and for the television: the manifest carries
`LEANBACK_LAUNCHER` **and** `LAUNCHER`, and `android.hardware.touchscreen`
is `required="false"`. A separate `googletv` flavour would be a product
decision (a second application id, a second release pipeline); it is not
part of this change and it is listed in section 12 as an open point.

## 4. The tests of the shell

```text
$ cd apps/android && ./gradlew :app:testDebugUnitTest --offline --rerun-tasks
> Task :app:testDebugUnitTest
BUILD SUCCESSFUL in 2m 15s
24 actionable tasks: 24 executed
```

| Suite | Tests | What it proves |
|---|---|---|
| `org.phairplay.receiver.ui.TvFocusGraphTest` | 12 | **the order of the focus** — every move of the D-pad, the edges of the grid, the banner, the grid without cards |
| `org.phairplay.receiver.model.ReceiverMappingTest` | 9 | the translation of the DTOs of the bridge into the model of the shell |
| `org.phairplay.receiver.bridge.ReceiverDefaultsTest` | 7 | the reader of the defaults file |
| `org.phairplay.receiver.model.ProtocolPhaseTest` | 7 | the phases of a protocol and their wording |
| `org.phairplay.receiver.model.FormattingTest` | 4 | the formatting of the durations and of the ports |
| **total** | **39** | **0 failures, 0 errors, 0 skipped** |

## 4a. The lint of the module

```text
$ cd apps/android && ./gradlew :app:lintDebug
BUILD SUCCESSFUL in 4m 22s
28 actionable tasks: 10 executed, 18 up-to-date
```

The lint of the module is **green**: no error, and the only warnings that
are left are in files outside the shell — the pinned versions of the
libraries in `build.gradle.kts` (a deliberate decision of the project),
the ABI of the Chrome OS (the application is built for ARM64 only) and
the folder `mipmap-anydpi-v26` (a leftover of an older `minSdk`). **Not a
single warning is in the layer of the interface** — the check of the
order of the parameters of a composable, the check of the translations
and the check of the unused resources were all repaired.

The lint had reported two errors before this change — both of them in
`ReceiverService.kt`, both of them about the permission
`POST_NOTIFICATIONS`, which the shell deliberately does not ask for: the
notification of a foreground service does not need it. The check is now
silenced on purpose, with the reason written next to it.

## 5. Before and after

| Aspect | Before | After | Why it is better on a television |
|---|---|---|---|
| The four protocols | a `LazyColumn` — the fourth card was below the edge of the screen | a grid of two rows of two, `ProtocolGrid.kt`, the height of the rows is the height that is left over | all four protocols are visible at once, without scrolling and without losing the place of the focus |
| The focus | nowhere to be seen | a frame of 4 dp in the accent colour, a growth of 3 % and a veil that brightens the card | the user sees from three metres where the focus is, also on a panel with a bad contrast |
| The order of the focus | the order of the composition | `TvFocusGraph.move()` — a pure function with 12 tests | "right" always leads to the same card; the order can be learned in one try |
| The switch of a protocol | a `Switch` of 48 dp in the corner of the card, a small target | the **card** is the target: `focusable()` + `onKeyEvent` — "OK", `Enter` and `Space` switch the protocol | a small target cannot be hit with a remote control |
| The state of a protocol | `bodyMedium` (14 sp) in a corner | a label of 22 sp (20 sp on a small screen) with a sign of its own | the state is read in one glance from three metres |
| The four states | a colour | a **word** (`Disabled` / `Advertising` / `Streaming` / `Error`) **and** a **shape** (a circle with a bar, a ring with a dot, a filled triangle, a triangle with a warning mark) | a colour weakness does not hide the state |
| The head of the dashboard | the name of the device and a small status | the name of the device, **one big status** ("The receiver is running" / "is not running"), the **help of the keys** ("Use the arrow keys to choose, press OK to switch") | the two first questions of the user are answered in the largest type of the screen |
| The running session | one line between the head and the list | `NowPlayingPanel` — its own surface above the grid, with the name of the sender, the codec and the duration | the reason the user opens the application is the most important thing on the screen |
| The application on the television | a telephone application | `LEANBACK_LAUNCHER` + a banner of 320 × 180 dp + `android.software.leanback` | the application appears in the list of the television and looks like a television application |
| The texts of the shell | — | every text in `res/values/strings.xml`, `values-de` and `values-fr`; no literal in the UI layer | the shell can be translated and reviewed without reading Kotlin |

## 6. The order of the focus — concretely

The grid always holds the protocols in the order of the core: **AirPlay
(0), Miracast (1), Google Cast (2), DLNA (3)**. The columns are the axis
of the horizontal keys, the rows are the axis of the vertical keys.

```text
             +---------------------------+---------------------------+
             |  0 AirPlay               |  1 Miracast              |
             +---------------------------+---------------------------+
             |  2 Google Cast           |  3 DLNA                  |
             +---------------------------+---------------------------+

        header (device name, status, help of the keys)
           |  up / down
        [ banner ]   (only while the bridge reports a failure)
           |  up / down
        card 0  <->  card 1        left / right inside a row
           |            |
        card 2  <->  card 3        up / down inside the column
```

Every key, from every place:

| From | Key | To |
|---|---|---|
| header | `DOWN` | the banner of the failure, if there is one — otherwise card 0 (AirPlay) |
| header | `UP` / `LEFT` / `RIGHT` | the header stays where it is (the key is eaten) |
| banner | `UP` | the header |
| banner | `DOWN` | card 0 (AirPlay) |
| banner | `LEFT` / `RIGHT` | the banner stays where it is |
| card 0 (AirPlay) | `RIGHT` | card 1 (Miracast) |
| card 0 | `LEFT` | card 0 stays (the edge of the row) |
| card 0 | `DOWN` | card 2 (Google Cast) |
| card 0 | `UP` | the banner, if there is one — otherwise the header |
| card 1 (Miracast) | `LEFT` | card 0 (AirPlay) |
| card 1 | `RIGHT` | card 1 stays (the edge of the row) |
| card 1 | `DOWN` | card 3 (DLNA) |
| card 1 | `UP` | the header |
| card 2 (Google Cast) | `RIGHT` | card 3 (DLNA) |
| card 2 | `LEFT` | card 2 stays |
| card 2 | `UP` | card 0 (AirPlay) |
| card 2 | `DOWN` | card 2 stays (the lower row has no row below it) |
| card 3 (DLNA) | `LEFT` | card 2 (Google Cast) |
| card 3 | `RIGHT` | card 3 stays |
| card 3 | `UP` | card 1 (Miracast) |
| card 3 | `DOWN` | card 3 stays |
| card 0 / 1 / 2 / 3 | `OK` (`DPAD_CENTER`), `Enter`, `Space` | **the protocol is switched on or off** — the card is the switch |
| card, banner | `BACK` | the header (the `BackHandler` of the screen) |
| header | `BACK` | the system leaves the application (the `BackHandler` is disabled on the header) |
| the banner of the failure | `OK` on the button | the banner is dismissed, the focus returns to the header |

The rule is not a guess: `TvFocusGraph.move()` is a pure function and
`TvFocusGraphTest` proves every line of the table above with 12 tests
(section 4).

## 7. The files

### The shell — `apps/android/app/src/main/java/org/phairplay/receiver/ui/`

| File | Lines | What it is |
|---|---|---|
| `ReceiverScreen.kt` | 233 | the dashboard: the head, the banner, the panel of the session and the grid — **the owner of the focus graph** |
| `ReceiverHeader.kt` | 198 | the head: the name of the device, the big status, the help of the keys; a focus point of its own |
| `ProtocolGrid.kt` | 105 | the grid of the four protocols in two rows of two; it never scrolls |
| `ProtocolCard.kt` | 348 | the card of one protocol: the name, the state with its sign, the ports, the switch; the card is the button |
| `TvFocus.kt` | 201 | the places of the dashboard, the four directions and the rule of the movement; the requesters of the focus |
| `TvFocusFrame.kt` | 69 | the frame of the focus (4 dp), the growth (3 %) and the veil — one modifier for every focusable surface |
| `NowPlayingPanel.kt` | 160 | the panel of the running session: the sender, the codec, the duration |
| `FailureBanner.kt` | 131 | the banner of a failure of the bridge and its button |
| `StatusGlyph.kt` | 135 | the sign of the state — the shape that a colour weakness does not hide |
| `StateLabels.kt` | 90 | the word and the colour of every phase of a protocol |
| `PlayMark.kt` | 42 | the triangle of a running stream |
| `WaitingPanel.kt` | 43 | the sentence while the bridge has not answered yet |
| `Theme.kt` | 66 | the type of the shell: the smallest text is 16 sp, the state of a card is 22 sp |
| `PhairPlayColors.kt` | 78 | the colours and their measured contrasts |

### The resources and the manifest

| File | Lines | What it is |
|---|---|---|
| `res/values/strings.xml` | 91 | every text of the shell |
| `res/values-de/strings.xml` | 86 | the German texts |
| `res/values-fr/strings.xml` | 90 | the French texts |
| `res/values/colors.xml` | 20 | the colours of the window and of the banner |
| `res/values/themes.xml` | 22 | the theme of the platform: a dark window, no action bar |
| `res/drawable/app_banner.xml` | 24 | the banner of the Leanback launcher: a layer list of 320 × 180 dp |
| `res/drawable/tv_banner_logo.xml` | 34 | the sign of the shell inside the banner |
| `AndroidManifest.xml` | 136 | `LEANBACK_LAUNCHER`, the banner, the `uses-feature` of the television — and **no** location, microphone or contact permission |

### The tests

| File | Lines | What it is |
|---|---|---|
| `app/src/test/java/org/phairplay/receiver/ui/TvFocusGraphTest.kt` | 154 | the 12 tests of the order of the focus |

### The files outside the UI layer that had to be repaired

The module did not compile at the revision of the audit — the errors were
in files that the assignment does not give to the interface, but they
block the build of the whole module, so they were repaired with the
smallest possible change. **Every one of them is listed here.**

| File | The error of the compiler or of the lint | The repair |
|---|---|---|
| `MainActivity.kt` | `Type 'State<ReceiverUiState>' has no method 'getValue(Nothing?, KProperty0<*>)'` | `import androidx.compose.runtime.getValue` added |
| `bridge/ReceiverHost.kt` | `Unresolved reference 'context'` (the constructor parameter was not a property) | `class ReceiverHost(private val context: Context)` |
| `bridge/ReceiverDefaults.kt` | `Unresolved reference 'number'` (`IndexedValue` has `index`, not `number`) | `number.index + 1` |
| `model/ReceiverMapping.kt` | `Argument type mismatch: actual type is 'kotlin.Int', but 'kotlin.UShort' was expected` | `port = 0u` |
| `ReceiverNotification.kt` | `Unresolved reference 'Icon'` (`androidx.core.graphics.drawable.Icon` does not exist in core 1.15.0) | `IconCompat.createWithResource(...)` |
| `ReceiverService.kt` | the lint: `posting a permission requires holding the POST_NOTIFICATIONS permission` | `@SuppressLint("MissingPermission", "NotificationPermission")` — the notification of a foreground service does not need that permission; the reason is written next to the annotation |
| `app/src/test/.../ReceiverMappingTest.kt` | 5 × `Int` against `UShort`, `Unresolved reference 'AacLc'` | `port = 0u` / `port = 7000u`, `FfiAudioCodec.AAC_LC` |

No file of the UI layer needed a change for these errors — they were all
outside `ui/`.

The lint of the module pointed at three more things of the layer of the
interface, and they are repaired as well: the order of the parameters of
the four composables (`modifier` is the first optional parameter again),
the missing quantity `many` of the French plural, and fourteen strings of
the old telephone interface that nothing used any more — they are gone
from all three languages. The three locales carry the same 54 keys.

## 8. The measurements

### 8.1 The contrast of the focus frame

The frame is drawn with the accent `#60A5FA`, four dp thick, on the
surface of the card `#1A2029`; the background of the screen is `#0E1116`.
The values below are computed with the WCAG formula
`(L1 + 0.05) / (L2 + 0.05)` over the relative luminance of the sRGB
values.

| Pair | Contrast | The rule | Verdict |
|---|---|---|---|
| focus frame `#60A5FA` against the card `#1A2029` | **6.44 : 1** | ≥ 3 : 1 for the focus | holds |
| focus frame `#60A5FA` against the screen `#0E1116` | **7.44 : 1** | ≥ 3 : 1 | holds |
| focus frame `#60A5FA` against the *lit* card `#363B43` (the veil over the surface) | **4.43 : 1** | ≥ 3 : 1 | holds |
| the text of the body `#F3F6FA` on the card `#1A2029` | 15.11 : 1 | ≥ 4.5 : 1 | holds |
| the text of the body on the lit card `#363B43` | 10.40 : 1 | ≥ 4.5 : 1 | holds |
| the muted text `#A9B4C2` on the card | 7.79 : 1 | ≥ 4.5 : 1 | holds |
| the state `Idle #9AA5B1` on the card | 6.54 : 1 | ≥ 4.5 : 1 | holds |
| the state `Failure #FB7185` on the card | 6.08 : 1 | ≥ 4.5 : 1 | holds |
| the state `Streaming #22D3EE` on the card | 9.06 : 1 | ≥ 4.5 : 1 | holds |
| the state `Advertising #4ADE80` on the card | 9.40 : 1 | ≥ 4.5 : 1 | holds |
| the state `Transition #FBBF24` on the card | 9.81 : 1 | ≥ 4.5 : 1 | holds |

The lowest pair of text on a card is 6.08 : 1 — every text of the shell
is far above the 4.5 : 1 that the rule asks for, and the focus frame is
more than twice as strong as the 3 : 1 that the rule asks for.

### 8.2 The type of the shell (`ui/Theme.kt`)

| Where | Style | Size | The rule |
|---|---|---|---|
| the name of the protocol on the card | `titleLarge` / `titleMedium` on a small screen | **24 sp / 22 sp** | ≥ 22 sp |
| the state of the protocol | `titleMedium` / `titleSmall` on a small screen | **22 sp / 20 sp** | ≥ 20 sp |
| the ports and the senders | `bodySmall` | **16 sp** | ≥ 16 sp |
| the explanation of the state | `bodyMedium` | 17 sp | — |
| the name of the device | `headlineSmall` / `titleLarge` on a small screen | 26 sp / 24 sp | — |
| the big status of the receiver | `titleLarge` / `titleMedium` on a small screen | 24 sp / 22 sp | one big status |
| the help of the keys | `bodyMedium` / `bodySmall` on a small screen | 17 sp / 16 sp | the help is never dropped |
| the sender of the running session | `titleLarge` / `titleMedium` on a small screen | 24 sp / 22 sp | — |
| the codec and the duration | `bodySmall` | 16 sp | — |

The smallest **information** of the shell is 16 sp — the ports and the
codec. Above a name there is a decorative label of 14 sp (the word
"receiver", the word "now playing", the version), which carries no
information and is the first thing that goes on a small screen. Nothing
of the state falls below 20 sp.

### 8.3 The layout without scrolling

The height budget of the dashboard, computed with the type of the theme
(the line height is approximated with 1.2 × the size in sp). The card
chooses the tier that fits its own height: **tight** (68 dp, the name and
the state), **middle** (124 dp, plus the ports), **roomy** (165 dp, plus
the sentence that explains the state).

| Screen | dp | Session | The grid | A card | The tier | The card needs | |
|---|---|---|---|---|---|---|---|
| 1920 × 1080 @ 320 dpi | 960 × 540 | no | 399.6 dp | 192.8 dp | roomy | 164.8 dp | fits |
| 1920 × 1080 @ 320 dpi | 960 × 540 | yes | 272.4 dp | 129.2 dp | middle | 124.0 dp | fits |
| 1280 × 720 @ 240 dpi | 853 × 480 | no | 339.6 dp | 162.8 dp | middle | 124.0 dp | fits |
| 1280 × 720 @ 240 dpi | 853 × 480 | yes | 212.4 dp | 99.2 dp | tight | 68.4 dp | fits |
| 1280 × 720 @ 320 dpi | 640 × 360 | no | 258.0 dp | 125.0 dp | middle | 124.0 dp | fits |
| 1280 × 720 @ 320 dpi | 640 × 360 | yes | 178.4 dp | 85.2 dp | tight | 68.4 dp | fits |
| a telephone, portrait | 393 × 873 | yes | 605.1 dp | 295.6 dp | roomy | 164.8 dp | fits |
| a telephone, landscape | 873 × 393 | yes | 211.1 dp | 101.6 dp | tight | 68.4 dp | fits |

In no case does the content of a card exceed the card, and in no case is
a part of the dashboard below the edge of the screen: **there is nothing
to scroll**. The four protocols are visible at the same time on every
one of these screens.

The script that computes the table:

```python
LINE = 1.2
TYPE = {"labelSmall": 14, "bodySmall": 16, "bodyMedium": 17, "titleSmall": 20,
        "titleMedium": 22, "titleLarge": 24, "headlineSmall": 26}
# header(dense), panel(dense) and card(height) add up the height of the
# parts out of the type of ui/Theme.kt; the screen height is
# height_px * 160 / dpi.
```

### 8.4 The texts

Every `R.string` that the UI layer uses exists in `values`, `values-de`
and `values-fr` — checked with a comparison of the names:

```text
$ grep -rho "R\.string\.[a-z_0-9]*" java/org/phairplay/receiver/ui/ | sort -u | wc -l
46
$ comm -23 used.txt values.txt values-de.txt values-fr.txt                       # no output
```

The three locales carry the same 54 keys, and every one of the 46 strings
that the interface uses exists in all three of them:

```text
$ for loc in values values-de values-fr; do grep -o 'name="[a-z_0-9]*"' res/$loc/strings.xml | sort > /tmp/$loc.txt; done
$ diff /tmp/values.txt /tmp/values-de.txt && diff /tmp/values.txt /tmp/values-fr.txt   # no output
```

There is **no literal** in the whole UI layer any more:

```text
$ grep -rn '(text|contentDescription|label|description)\s*=\s*"[^"]*"' java/org/phairplay/receiver/ui/
(no match)
```

The one literal that the shell had — the description of a card for the
screen reader — was replaced by the resource `protocol_card_description`
(`"%1$s, %2$s"`, in all three languages), so that the description that a
screen reader reads comes from the resources as well.

### 8.5 The Leanback entry of the manifest

The merged manifest of the build (`app/build/intermediates/merged_manifests/debug/processDebugManifest/AndroidManifest.xml`)
carries:

```text
<uses-feature android:name="android.software.leanback" android:required="false" />
<uses-feature android:name="android.hardware.touchscreen" android:required="false" />
<application android:banner="@drawable/app_banner" ...>
    <activity android:name=".MainActivity" android:banner="@drawable/app_banner" ...>
        <intent-filter>
            <action android:name="android.intent.action.MAIN" />
            <category android:name="android.intent.category.LAUNCHER" />
            <category android:name="android.intent.category.LEANBACK_LAUNCHER" />
        </intent-filter>
```

The permissions of the merged manifest — **no location, no microphone, no
contacts**:

```text
android.permission.INTERNET
android.permission.ACCESS_NETWORK_STATE
android.permission.CHANGE_WIFI_MULTICAST_STATE
android.permission.WAKE_LOCK
android.permission.FOREGROUND_SERVICE
android.permission.FOREGROUND_SERVICE_MEDIA_PLAYBACK
android.permission.NEARBY_WIFI_DEVICES   (neverForLocation)
android.permission.ACCESS_WIFI_STATE
android.permission.CHANGE_WIFI_STATE
```

The banner exists: `res/drawable/app_banner.xml` is a layer list of
320 × 180 dp (the launcher scales it), with the sign of the shell
(`res/drawable/tv_banner_logo.xml`) in the middle.

## 9. What is NOT RUN, and why

| Item | Status | The exact reason |
|---|---|---|
| `./gradlew :app:compileGoogletvDebugKotlin` | **NOT RUN** | `Cannot locate tasks that match ':app:compileGoogletvDebugKotlin' as task 'compileGoogletvDebugKotlin' not found in project ':app'.` — the module declares no `productFlavors`. The variant `debug` (the same Kotlin) is green, section 2. |
| `./gradlew :app:assembleDebug` (the APK) | **NOT RUN** | the task `buildRustBridgeDebug` cannot even be created: `Could not create task ':app:buildRustBridgeDebug'. > NDK is not installed`. The NDK that is installed (`27.2.12479018`) is not the version that the Android Gradle plugin requires, and the task would in any case cross-compile the Rust bridge for `aarch64-linux-android` — the assignment says that no Cargo build is needed. |
| The dashboard on a device or on an emulator | **NOT RUN** | this sandbox has no emulator and no device: the SDK carries `platform-tools`, but there is no system image and no `emulator` binary. The proof of the order of the focus is therefore the 12 unit tests of `TvFocusGraphTest` and the merge of the manifest, not a screenshot. |
| A screenshot of the dashboard | **NOT RUN** | the same reason: no emulator, no device. |

## 10. The states as a word and as a form

| State | The word (en / de / fr) | The form | The colour |
|---|---|---|---|
| disabled | `Disabled in the settings` / `In den Einstellungen deaktiviert` / `Désactivé dans les paramètres` | a circle with a bar through it | `#9AA5B1` |
| not started / stopped | `Not started` / `Service stopped` / `Nicht gestartet` / `Dienst gestoppt` / `Pas démarré` / `Service arrêté` | an empty circle | `#9AA5B1` |
| starting / stopping | `Starting` / `Stopping` / `Startet` / `Stoppt` / `Démarrage` / `Arrêt` | a half circle | `#FBBF24` |
| advertising | `Advertising` / `Wird angeboten` / `Annoncé` | a ring with a dot in the middle | `#4ADE80` |
| streaming | `Streaming` / `Stream läuft` / `Diffusion` | a filled triangle | `#22D3EE` |
| error | `Error` / `Fehler` / `Erreur` | a triangle with a warning mark | `#FB7185` |

## 11. The accessibility of the focus

* Every focusable surface has a **visible frame** of 4 dp (the rule asks
  for 3 dp), a growth of 3 % and a veil — three signs, so that the focus
  is also found on a television with a bad panel.
* Every focusable surface has a **description for the screen reader**:
  the card announces its protocol, its state and its switch
  (`role = Role.Switch`, `contentDescription`, `stateDescription`, an
  `onClick` action with the sentence "Switch AirPlay 2 off"), the head
  and the banner are read as they are written.
* The order of the focus is **deterministic** and does not depend on the
  order of the composition — the framework never searches for the next
  focusable node, because the keys of the D-pad are eaten by the screen
  before they reach the framework.
* The key "back" has two meanings that do not collide: on a card or on
  the banner it brings the user back to the head, on the head it leaves
  the application.

## 12. The open points — a decision of the owner is needed

1. **A separate `googletv` flavour?** The assignment names the task
   `:app:compileGoogletvDebugKotlin`, but the module has no flavours.
   The shell is today one application for the telephone and for the
   television. If a second application id (a second entry in the store, a
   second release) is wanted, `apps/android/app/build.gradle.kts` needs a
   `productFlavors` block — that changes the release pipeline and was
   therefore not done here.
2. **The dense screen.** Below 420 dp of height the dashboard drops the
   word "receiver" above the name of the device, the title of the track
   in the panel of the session and the ports on the cards. The state of
   the protocol stays at 20 sp. If the ports have to stay visible on
   every screen, the panel of the session has to move beside the grid.
3. **The banner of the failure** takes a place of its own in the focus
   (between the head and the grid) and pushes the grid down by 90 dp. On
   a 720p screen with a failure the cards therefore fall into the tight
   tier. An alternative would be a banner that floats over the head.
4. **The picture of the sender** (the cover) is drawn with 72 dp in the
   panel of the session. On a dense screen it is the first thing that
   should go — it is not decided yet whether it disappears there.
5. **The `Switch` of the card is a picture** and cannot be reached with
   the focus. This is the intended behaviour ("the card is the button"),
   but it has to be confirmed by the owner, because it changes the
   meaning of the switch for a user who uses a touch screen.
