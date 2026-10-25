# Acceptance — M3 Native shells

**Requirement IDs:** FR-070 … FR-078, FR-084, FR-096 … FR-099, NFR-PERF-001 … NFR-PERF-005, NFR-INT-001, NFR-REL-002, NFR-DOC-003
**Milestone:** M3 — Native shells
**Build/commit:** branch `feat/other-devices` at `2d0c4b4`. The work of M3 itself is the commit `b0d0f40` ("feat(m3): an interface that a remote control can operate"). The sources of the three shells are identical to that commit.
**Tester/agent roles:** QA & Test Engineer (this record, the re-measurement), Platform Worker (the audits of the Android and the macOS shell), Frontend Worker (the audit of the dashboard)
**Date:** 2026-10-24
**Reference device (hardware, OS, network):** Linux x86_64, 2 CPUs, 8 GB RAM, Android SDK `/workspace/.home/tools/android-sdk`, JDK 17.0.21, Gradle 8.13, Swift 6.0.3, Chromium (Playwright, headless). **No television, no telephone, no macOS machine and no sender were available** — see §3, §7 and §9.

**The one sentence of this record:** the three shells of M3 are written, their
logic is covered by tests, the manifest carries no location, microphone or
contact permission and the texts live in the resources — but **at the revision
under test the Kotlin of the Android shell and the kit of the macOS shell do
not compile**, because the milestone M5 added a protocol kind and a limits
field to the bridge and did not update the shells. The four compile errors are
named, reproduced and localised in §1a; the M3 sources themselves are
unchanged since `b0d0f40` and were green there — measured for this record:
`BUILD SUCCESSFUL in 2m 27s` and 39 green tests.

**Continuous integration:** the workflow of the repository
(`.github/workflows/ci.yml`) was **not** executed by this record — there is no
runner here. What *is* measured is the part of the gate that runs on this
machine: `scripts/check-crate-boundaries.sh` (0 violations),
`scripts/check-docs-links.sh` (0 broken links) and `cargo fmt --all -- --check`
(exit 0). **The native shells are not part of that gate** — no Gradle task
and no Swift test runs in the CI of the repository, and the workflow triggers
on `push` to `main` and on `pull_request` to `main` only. That is the reason
why a defect of §1a could reach the branch: the gate that would have caught
it does not exist.

---

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| The Kotlin of the Android shell compiles | no error | **fail** at the revision under test | `cd apps/android && ./gradlew :app:compileDebugKotlin` → `e: .../receiver/model/ReceiverMapping.kt:91:56 'when' expression must be exhaustive. Add the 'SMART_VIEW' branch or an 'else' branch.` and `e: .../receiver/bridge/ReceiverDefaults.kt:104:17 No value passed for parameter 'audioQueueBytes'.` → `> Task :app:compileDebugKotlin FAILED`, `BUILD FAILED in 5m 12s` |
| The Kotlin of the Android shell compiled at the commit of M3 | no error | **pass** | measured for this record: the archive of the commit `b0d0f40` (`git archive b0d0f40`), `cd apps/android && ./gradlew :app:compileDebugKotlin` → `BUILD SUCCESSFUL in 2m 27s`, `15 actionable tasks: 15 executed`, exit 0. The audit of the platform worker measured the same state: [`docs/audit/tv-android-shell-compile.log`](../audit/tv-android-shell-compile.log) → `BUILD SUCCESSFUL in 2m 59s` |
| The unit tests of the Android shell pass | 39 tests, 0 failures | **pass at the commit of M3** — **NOT RUN at the revision under test** (the module does not compile, §1 row 1) | measured for this record in the archive of `b0d0f40`: `./gradlew :app:testDebugUnitTest` → `BUILD SUCCESSFUL in 2m 34s`, exit 0; the result files `app/build/test-results/testDebugUnitTest/*.xml` of that run: `TvFocusGraphTest` 12, `ReceiverMappingTest` 9, `ReceiverDefaultsTest` 7, `ProtocolPhaseTest` 7, `FormattingTest` 4 — **39 tests, `failures="0" errors="0" skipped="0"` in every file**. The audit of the platform worker measured the same state: [`docs/audit/tv-android-shell-test.log`](../audit/tv-android-shell-test.log) → `BUILD SUCCESSFUL in 2m 15s`; its result files are still on disk (`apps/android/app/build/test-results/testDebugUnitTest/*.xml`, 2026-10-24 09:49) |
| The kit of the macOS shell compiles | no error | **fail** at the revision under test | `cd apps/macos && swift test` → `ReceiverMapping.swift:83:9: error: switch must be exhaustive` / `note: add missing case: '.smartView'` and `ReceiverDefaults.swift:124:83: error: missing argument for parameter 'audioQueueBytes' in call` → `error: fatalError` |
| The kit of the macOS shell compiled at the commit of M3 | no error | **pass** | measured for this record: the archive of the commit `b0d0f40` (`git archive b0d0f40`) with the bridge of the same commit (`cargo build -p phairplay-ffi --offline` → `Finished dev profile … in 5m 46s`, exit 0), `cd apps/macos && swift test` → `Test Suite 'All tests' passed at 2026-10-24 22:44:59.434` / `Executed 101 tests, with 0 failures (0 unexpected) in 0.357 (0.357) seconds`, exit 0. The audit of the platform worker measured the same state: [`docs/audit/tv-macos-shell-swift-test.log`](../audit/tv-macos-shell-swift-test.log) → `Executed 101 tests, with 0 failures (0 unexpected) in 0.534 (0.534) seconds` |
| The tests of the macOS kit pass | 101 tests, 0 failures | **pass at the commit of M3** — **NOT RUN at the revision under test** (the kit does not compile, §1 row 4) | the run of the row above: `Executed 101 tests, with 0 failures (0 unexpected)`; the eleven suites are listed in [`docs/audit/tv-macos-shell.md`](../audit/tv-macos-shell.md) §1 |
| The state of every protocol is visible in the Android shell | one card per protocol with its state | **pass** (code level) | `apps/android/app/src/main/java/org/phairplay/receiver/ui/ProtocolGrid.kt` (the grid of two rows of two), `ProtocolCard.kt` (the card), the mapping `model/ReceiverMapping.kt` → `protocolCard()` |
| The four states — disabled, advertising, streaming, error — are distinguishable in the Android shell | a word and a shape each | **pass** (code level) | `ui/StateLabels.kt` (the word of the eight phases) and `ui/StatusGlyph.kt` (the shape of every phase: a circle with a bar, a ring with a dot, a filled triangle, a triangle with a warning mark); the words in `res/values/strings.xml` lines 44–59, `res/values-de/strings.xml` lines 39–54, `res/values-fr/strings.xml` lines 43–58 |
| "Stopped" and "disabled" are not the same message in the Android shell | two different words | **pass** (code level) | `res/values/strings.xml`: `protocol_state_disabled` = "Disabled in the settings" (line 44) against `protocol_state_stopped` = "Service stopped" (line 48) and `protocol_state_not_started` = "Not started" (line 46); the mapping is proved by `ProtocolPhaseTest.stoppedAndDisabledAreNotTheSame` and `aStoppedProtocolIsStopped` |
| The four states are distinguishable in the macOS shell | a word, a shape and a tone each | **pass** (code level) | `Sources/PhairPlayMacKit/ReceiverPhaseStyle.swift` and `ReceiverStatus.swift`; the proof is `ShellStatusTests` (12 tests) and `ShellToneTests` (7 tests) — the log of the run: `Executed 12 tests, with 0 failures` and `Executed 7 tests, with 0 failures` |
| "Stopped" and "disabled" are not the same message in the macOS shell | two different words | **pass** (code level) | `Sources/PhairPlayMacKit/ReceiverModel.swift` — the phases `.disabled`, `.notStarted` and `.stopped` are three members; `ShellStatusTests.testAReceiverWithoutAnyServiceIsStopped` and `testTheWordOfAStateIsNoKey` |
| The card is the switch in the Android shell | *OK*, *Enter* and *Space* switch the protocol that holds the focus | **pass** (code level) | `ui/ProtocolCard.kt` lines 121–133: `.focusable()`, `.onKeyEvent { … }`, `role = Role.Switch`, `stateDescription = switchLabel`; the keys are listed in `ProtocolCard.kt` line 305: `Key.Enter, Key.NumPadEnter, Key.Spacebar, Key.DirectionCenter -> true` |
| The card is the switch in the macOS shell | *Space* switches the protocol that holds the focus | **pass** (code level) | `Sources/PhairPlayMacKit/ShellKeyboard.swift` line 188–189: `case .space: return ShellKeyOutcome(focus: focus, action: toggleAction(for: focus))`; the proof is `ShellKeyboardTests.testTheSpaceKeySwitchesTheFocusedProtocol` |
| The order of the focus is deterministic in the Android shell | every key of the D-pad leads to the same place | **pass** (code level) | `ui/TvFocus.kt` — `TvFocusGraph.move()` is a pure function; `TvFocusGraphTest` (12 tests, 0 failures) proves every line of the table of [`docs/audit/tv-android-shell.md`](../audit/tv-android-shell.md) §6 |
| The order of the focus is deterministic in the macOS shell | every key leads to the same place | **pass** (code level) | `Sources/PhairPlayMacKit/ShellFocus.swift`; `ShellFocusTests` (12 tests, 0 failures) |
| The dashboard of the browser is operable with the keyboard alone | the arrow keys move the focus, the group is one tab stop | **pass** (measured in a real Chromium) | `node apps/web/tools/check-keyboard.mjs` → `35 checks passed, 0 failed` — among them `exactly one card is in the tab order (tabindex: 0, -1, -1, -1, -1)`, `1920x1080: ArrowRight moves the focus to card-miracast`, `the arrow key at the edge of the group reaches metric-fps` |
| The switch of the dashboard calls the real endpoint of the receiver | `POST /api/protocols/<id>/enable` | **pass** (measured in a real Chromium) | the same run: `Enter switches the receiver (aria-pressed false → true, POST /api/protocols/cast/enable)`, `the switch calls the API of the receiver (POST /api/protocols/cast/enable)`, `the next reading of the API keeps the state of the card (false)` |
| The focus frame of the dashboard is visible and measured | ≥ 3 px and ≥ 3:1 in both themes | **pass** (measured in a real Chromium) | `node apps/web/tools/check-contrast.mjs` → `every measured pair reaches its threshold`; the pairs: the ring `rgb(196, 181, 253)` **9.96:1** against the card and **10.65:1** against the page (dark), `rgb(75, 49, 190)` **8.46:1** / **7.62:1** (light) |
| The receivers of the dashboard are visible without scrolling on a television | one row at 1920 × 1080 | **pass** (measured in a real Chromium) | `node apps/web/tools/check-layout.mjs` → `the four receivers are visible without scrolling at 1920x1080 (5 column(s), 247–746 of 1080; title 28px, state 24px, detail 18px, label 16px)` — the wording of the check still says "four", the branch now carries five receivers (M5); see limitation 11 |
| The Android shell is an application of the television | `LEANBACK_LAUNCHER`, a banner, `uses-feature` of the television | **pass** (manifest level) | the merged manifest of the build of 2026-10-24 09:48 (`app/build/intermediates/merged_manifests/debug/processDebugManifest/AndroidManifest.xml`): `<category android:name="android.intent.category.LEANBACK_LAUNCHER" />`, `android:banner="@drawable/app_banner"`, `<uses-feature android:name="android.software.leanback" android:required="false" />`; the banner itself: `res/drawable/app_banner.xml` (320 × 180 dp) and `res/drawable/tv_banner_logo.xml` |
| The application appears in the list of the television | the entry of the launcher | **NOT RUN** | needs a television — §9 |
| A protocol can be switched on and off from the shell, and the change is visible in the picker of the sender | the switch of the sender | **NOT RUN** | needs a sender and a device — §9, the script is in [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) |

### 1a. The two defects of the revision under test — in full

Both shells broke in the same way, and both defects are the same kind of
defect: **the contract of the bridge changed and the shells were not
updated.**

**Android** — `cd apps/android && ./gradlew :app:compileDebugKotlin`:

```text
e: file:///…/apps/android/app/src/main/java/org/phairplay/receiver/bridge/ReceiverDefaults.kt:104:17 No value passed for parameter 'audioQueueBytes'.
e: file:///…/apps/android/app/src/main/java/org/phairplay/receiver/model/ReceiverMapping.kt:91:56 'when' expression must be exhaustive. Add the 'SMART_VIEW' branch or an 'else' branch.

> Task :app:compileDebugKotlin FAILED
BUILD FAILED in 5m 12s
```

**macOS** — `cd apps/macos && swift test`:

```text
…/Sources/PhairPlayMacKit/ReceiverMapping.swift:83:9: error: switch must be exhaustive
83 |         switch kind {
   |         |- error: switch must be exhaustive
   |         `- note: add missing case: '.smartView'
…/Sources/PhairPlayMacKit/ReceiverDefaults.swift:124:83: error: missing argument for parameter 'audioQueueBytes' in call
error: fatalError
```

**Where the regression comes from.** The four shell files that break are
**byte-identical** to the commit of M3, and only the generated bindings have
changed since. **Measured, not argued:** the archive of `b0d0f40` compiles
(`BUILD SUCCESSFUL in 2m 27s`, 39 green tests) and runs its 101 Swift tests
(0 failures), the branch under test compiles neither shell.

```text
$ git diff --stat b0d0f40..HEAD -- \
    apps/android/app/src/main/java/org/phairplay/receiver/model/ReceiverMapping.kt \
    apps/android/app/src/main/java/org/phairplay/receiver/bridge/ReceiverDefaults.kt \
    apps/macos/Sources/PhairPlayMacKit/ReceiverMapping.swift \
    apps/macos/Sources/PhairPlayMacKit/ReceiverDefaults.swift
(no output — the files are untouched)

$ git diff --stat b0d0f40..HEAD -- apps/android/app/src/main/java/uniffi/ apps/macos/Sources/PhairPlayFFI/
 …/java/uniffi/phairplay_ffi/phairplay_ffi.kt | 13 +++++++++++-
 …/macos/Sources/PhairPlayFFI/phairplay_ffi.swift | 24 ++++++++++++++++++++++
```

The diff of the bindings is exactly the two things the shells do not know:

```text
+    SMART_VIEW;                 // a fifth protocol kind
+    var `audioQueueBytes`: kotlin.ULong,   // a sixth field of the limits
```

Both come from the commit `86c1b81` ("feat(m5): Samsung Smart View, and the
composition root moves to the bridge"), which regenerated the bindings.

**What the fix is.** Two places per shell: a branch for the fifth protocol
kind in `protocolIdOf` / `protocolId(of:)` — the shell would leave the card
out, as the comment of the function already says — and the new limit in the
reader of the defaults. It is a small change, but it is **not** part of this
record, because this record is the acceptance of M3 and the repair belongs to
the milestone that introduced the break.

**Why the CI did not catch it.** The gate of the project
(`.github/workflows/ci.yml`) runs `cargo fmt`, `cargo clippy`, `cargo test`,
`cargo doc`, `cargo-deny`, `cargo audit` and the four structural scripts.
**No Gradle task and no Swift test is part of it.** The shells of M3 are
therefore outside every continuous gate of the repository — a finding of its
own, recorded in §8.

## 2. Technical

- The crate boundaries are respected: **yes** — `bash scripts/check-crate-boundaries.sh` → `Crates checked: 13`, `Internal edges: 15`, `Violations: 0`, exit 0. The shells themselves reach the core only through the generated bindings (`uniffi.phairplay_ffi.Receiver` in `apps/android/…/bridge/ReceiverHost.kt`, `FfiReceiverEvent` in `apps/macos/Sources/PhairPlayMacKit/ReceiverClient.swift`); neither shell contains a socket of its own (FR-070).
- The central configuration is respected: **yes** — `bash scripts/check-docs-links.sh` → `Markdown files scanned: 68`, `Internal links checked: 691`, `Broken links: 0`, `Budgets compared: 16`, `Budgets owned alone: 7`, exit 0. The scan **includes** the two documents of this cycle (`docs/acceptance/M3-ACCEPTANCE.md` and `docs/audit/acceptance-m3.md`) — they add links and no broken one.
- The dependency direction is unchanged: **yes** — the same gate; the six ADRs under `docs/adr/` record the decisions (FR-078).
- The tests are green: **no** — the Kotlin module of the Android shell and the kit of the macOS shell do not compile at the revision under test (§1a). The Rust workspace was **not measured** in this cycle — see §2a.
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): **not measured in this cycle** — the machine was shared with three other build agents and the clippy run of the whole workspace did not finish inside the time of this record. The last green measurement of the lint is the one of the acceptance record of M0, at the revision `7cd6fe0`: `cargo clippy --workspace --all-targets -- -D warnings` → no warning, exit 0 ([`docs/acceptance/M0-ACCEPTANCE.md`](M0-ACCEPTANCE.md) §2). **That is a measurement of another revision, not of this one.**
- The format check is green (`cargo fmt --all -- --check`): **yes** — exit 0.
- The documentation links resolve (`bash scripts/check-docs-links.sh`): **yes** — exit 0, `Broken links: 0`.
- The versioning or the migration is checked: **not applicable** — the project is at version 0.1.0 (`VERSION`) and has no migration to prove.
- The licence headers are present: **yes** — `bash scripts/check-license-headers.sh` → `Violations: 0`, `OK: every licence header of the repository is correct.`, exit 0.
- The length of the source files is within the limit: **yes** — `bash scripts/check-file-size.sh` → `Above the soft limit: 9`, `Above the hard limit: 0`, `OK: every hand-written source file is inside the hard limit of 550 lines.`, exit 0. The Swift kit carries the same rule as a test, `ShellKitPurityTests` ("no file of the shell is above 400 lines").
- The dependency policy is satisfied: **NOT RUN** in this cycle — `bash scripts/audit-licenses.sh --check-only` → `NOT RUN: cargo-deny is not installed on this machine.`, exit 2. The exit code 2 is the contract of the script for "the proof did not happen". The last green measurement is the one of the M0 record at the revision `7cd6fe0` (`bans ok, licenses ok, sources ok` with the pinned cargo-deny 0.20.2).

### The Rust workspace at the revision under test

The run was started and did not finish — §2a states the reason and what the
number of the last green run is.

## 2a. The Rust workspace — NOT RUN in this cycle

`cargo test --workspace --offline` was started for this record at 21:54 on
the revision under test. It did **not** produce a result: the machine was
shared with three other build agents (load average 8 on two CPUs), the
compilation of the test binaries of the whole workspace needed more than two
hours, and the run was stopped while it was still linking
(`phairplay_ffi`, `phairplay_web`, `phairplay_core`).

**The Rust side of this branch is therefore NOT measured by this record.** The
last green measurement is the acceptance record of M0 at the revision
`7cd6fe0` — **1054 passed, 0 failed**. That is a measurement of *another*
revision and it is deliberately not repeated here as if it were a measurement
of this one.

The Rust side is not the subject of M3 — the milestone is the shells — and
nothing in the gates of the milestone above depends on it. The gap is
recorded in §9.

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | not measured | **NOT RUN** |
| Sustained frame rate | `throughput.minimum_fps` | not measured | **NOT RUN** |
| Resident memory | `resources.max_memory_mb` | not measured | **NOT RUN** |
| CPU usage | `resources.max_cpu_percent` | not measured | **NOT RUN** |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | not measured | **NOT RUN** |

**Why:** every one of these budgets is a property of the *running* receiver
on a *real* device, with a *real* sender. The sandbox has neither a
television nor a sender, and a measurement on a build server would be a
number without meaning. The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml) and
are **not** repeated here.

**The gate of M3 that belongs to this section** — "the resident memory and the
CPU usage of the shell stay inside the budgets with a running session" — is
**NOT RUN** for the same reason, and it is the gate of §9 of the roadmap that
is open.

**How it will be measured:** the device test plan
([`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md)) names the
sender, the command and the expected result for every one of them, in §3
("The criteria that need a measurement, not a sender"). The plan has **no
section for the native shells** — see §8, limitation 5.

## 4. Usability

- The number of steps of the task: **not measured on a device.** What the code promises is written down: the head of the Android dashboard carries the help of the keys ("Use the arrow keys to choose, press OK to switch"), and the hint of the macOS window is built out of the groups of shortcuts of the kit, so that it cannot name a key that the shell does not answer to (`ShellShortcutTests.testEveryKeyOfTheShellBelongsToExactlyOneGroup`).
- The discoverability notes: the Android shell answers the two first questions of the user in the largest type of the screen — the name of the device and one big status ("The receiver is running" / "is not running") (`ui/ReceiverHeader.kt`); the dashboard of the browser shows the hint of the arrow keys **only** on a device without a fine pointer — measured: `node apps/web/tools/check-layout.mjs` → `a device without a mouse sees the hint for the arrow keys`, `a device with a mouse does not see the hint`.
- The distinction of the states ("disabled", "advertising", "streaming", "error"): **proved in the code, in three shells.** No state is carried by a colour alone: the Android shell draws a shape per phase (`ui/StatusGlyph.kt`), the macOS shell a symbol (`ReceiverPhaseStyle.swift`), the dashboard a shape per badge — the disc of "advertising", the triangle of "streaming", the bar of "disabled", the triangle of "error" (`apps/web/assets/cards.css`, `.state__glyph`), the word and the tone per state (`apps/web/assets/states.js`). The dashboard measures 38 colour pairs in two themes — every one of them above its threshold.
- The clarity of an error message: every failure of the core carries a typed error with a stable numeric code (`phairplay_core::error::ErrorCode`); the shells show the reason of the last failure on the card and in the banner of a failure (`ui/FailureBanner.kt`, `FailureBannerView.swift`). The dashboard repeats the banner after a failed reading of the API and keeps it closed until the next success — measured: `a receiver that cannot be reached shows a banner`, `Escape closes the banner`, `the closed banner does not come back with the next reading of the API`.
- The mobile / native behaviour: **NOT RUN** — the device test plan. The layout of the shells was *computed*, not measured on a screen: the audit of the Android shell §8.3 shows that the four cards fit on eight screen sizes without scrolling; the dashboard was measured in the browser at six widths (§1).

## 5. Accessibility

- The keyboard operation: **proved** for all three shells. The dashboard drives a real Chromium with the keyboard only — the pointer is never used — and reports `35 checks passed, 0 failed`. The Android shell proves the order of the focus with 12 tests (`TvFocusGraphTest`), the macOS shell with 12 (`ShellFocusTests`) and 10 (`ShellKeyboardTests`).
- The screen reader (VoiceOver / TalkBack): **NOT RUN** — that needs a device. What the code carries: the Android card announces its protocol, its state and its switch (`role = Role.Switch`, `contentDescription`, `stateDescription`, an `onClick` with the sentence "Switch AirPlay 2 off" — `ui/ProtocolCard.kt` lines 121–133); the macOS shell gives every state a symbol (`ShellStatusTests.testEveryStateHasItsOwnShape`); the dashboard gives every card a name for the screen reader — measured: `every card carries a name for the screen reader`.
- The scaling, the high contrast, the reduced motion: **measured**. The dashboard: 38 pairs in two themes, every one above its threshold; the focus ring is 3 px wide in both themes and is not animated into existence; under `prefers-reduced-motion` the ring stays and the movement stops — measured: `the ring stays, the movement stops (solid 3 px, transform none)`. The Android shell: the palette was recomputed for this record with the formula of WCAG 2.1 out of the constants of `ui/PhairPlayColors.kt` — the focus frame `#60A5FA` against the card `#1A2029` **6.44:1** and against the screen `#0E1116` **7.44:1** (the rule asks for 3:1); the lowest text pair is the failure `#FB7185` on the card with **6.08:1** (the rule asks for 4.5:1); the eleven pairs are in §8.1 of [`docs/audit/tv-android-shell.md`](../audit/tv-android-shell.md) and were reproduced exactly by this recomputation.

## 6. Security and privacy

| Measure | Result | Evidence |
|---|---|---|
| The manifest asks for no location permission | **pass** | the source manifest `apps/android/app/src/main/AndroidManifest.xml` and the merged manifest of the build: nine platform permissions — `INTERNET`, `ACCESS_NETWORK_STATE`, `CHANGE_WIFI_MULTICAST_STATE`, `WAKE_LOCK`, `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `NEARBY_WIFI_DEVICES` (with `android:usesPermissionFlags="neverForLocation"`), `ACCESS_WIFI_STATE`, `CHANGE_WIFI_STATE`. **No `ACCESS_FINE_LOCATION`, no `ACCESS_COARSE_LOCATION`, no `RECORD_AUDIO`, no `READ_CONTACTS`.** |
| The manifest asks for no microphone permission | **pass** | the same list — no member of it is `RECORD_AUDIO` |
| The manifest asks for no contact permission | **pass** | the same list — no member of it is a contact permission |
| The permission list is minimal | **pass** (with one note) | the merged manifest carries **one** permission beyond the nine of the source manifest: `org.phairplay.receiver.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` with `android:protectionLevel="signature"`. It is added by AndroidX, it is not shown to the user and it grants nothing (FR-075). |
| The shell asks for no permission for its notification | **pass** | `ReceiverService.kt` line 57: `@SuppressLint("MissingPermission", "NotificationPermission")` — the reason is written next to it: the notification of a foreground service does not need `POST_NOTIFICATIONS` |
| The shell opens no network access of its own | **pass** | the shells talk to the core **in process** through the UniFFI bridge (`uniffi.phairplay_ffi.Receiver`, `FfiReceiverEvent`); the only network access is that of the receiver itself |
| The negative tests of the shells | **pass** | `ProtocolPhaseTest.aProtocolThatIsSwitchedOffIsDisabledEvenWhileItStops` and `stoppedAndDisabledAreNotTheSame` (a state that is off must not read as a running one), `TvFocusGraphTest.theEdgeOfTheRowKeepsTheFocus`, `theLowerRowHasNoRowBelowIt`, `aDashboardWithoutCardsKeepsTheFocus`, `aGridWithOneCardStaysInItsColumn` (the focus must never get lost), `ShellFocusTests` (the edges of the grid), `ShellCatalogueTests.testNoViewWritesATextOfItsOwn` (no text of its own in a view) |
| The log of the shell names no peer | **pass** | the Android shell writes exactly two log lines — `Log.e(TAG, "the receiver could not be started: …")` (`bridge/ReceiverHost.kt` line 110) and `Log.w(TAG, "the multicast lock could not be taken: …")` (`ReceiverService.kt` line 74). Neither carries a name, an address or a socket address. The masking of the core is proved by the tests of `phairplay-core` (see the M0 record §6) |
| The dependency policy is satisfied | **NOT RUN** in this cycle | `bash scripts/audit-licenses.sh --check-only` → `NOT RUN: cargo-deny is not installed on this machine.`, exit 2 (the contract of the script: 2 = "the proof did not happen"). The last green measurement: the M0 record §2 at the revision `7cd6fe0` — `bans ok, licenses ok, sources ok` with the pinned cargo-deny 0.20.2. |

**Threat notes:** the shell is a surface that runs on a television in a living
room. Its attack surface is the **focus graph** (a key press must never
reach a place the user did not ask for — proved by `TvFocusGraphTest` and
`ShellFocusTests`) and the **semantic description of the card** (the
`contentDescription` comes from the resources, not from a literal — proved in
§8.4 of the Android audit). The shell itself parses nothing and opens no
socket of its own.

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| An Apple sender (macOS, iOS) | the handshake and the picture on the screen of the shell | **NOT RUN** | needs hardware — [`docs/guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) §2.1 |
| A Windows sender ("Cast to Device") | the handshake | **NOT RUN** | needs hardware — §2.2 and §2.3 of the plan |
| VLC and other UPnP control points | the description and the SOAP answers | **NOT RUN** | needs a control point — §2.4 of the plan |
| A Chromecast | the receiver handshake | **NOT RUN** | needs a sender — §2.3 of the plan |

- The formats that a reference implementation reads: **not a question of M3** — the format layer belongs to M0 and is covered by the M0 record.
- The receiver without a component of the vendor of the sender: **not measured.** What the source shows is that the shell is written in Kotlin, Swift and JavaScript against one bridge (`apps/android`, `apps/macos`, `apps/web`) and contains no code of any sender vendor — but "the receiver works without a sender of the same vendor" (`NFR-INT-002`) is a statement about a *running* system and stays **NOT RUN**.
- The deviations from the expectation of the sender: **NOT RUN** — nothing was mirrored from a sender.

## 8. Known limitations

1. **The shells of the branch do not compile.** Two errors in the Kotlin and
   two in the Swift, all four of them the same cause: the bridge gained a
   protocol kind and a limits field in `86c1b81` (M5) and the shells were not
   updated. **Severity S3** — a major build failure; the application of the
   branch cannot be built, and a milestone that claims "the native shell is
   delivered" is not true of this branch. It is **not** S4: nothing crashes,
   no data is lost, no security is bypassed — the build stops.
2. **The native shells are outside every continuous gate.** The CI of the
   repository (`.github/workflows/ci.yml`) runs four jobs — *Quality Gate*
   (format, lint, tests, documentation), *Licenses* (cargo-deny),
   *Security Audit* (cargo-audit) and *Structure* (the four scripts). **No
   Gradle task and no Swift test is part of it**, and the workflow triggers
   on `push` to `main` and on `pull_request` to `main` — a push to the
   feature branch runs nothing at all. The defect of limitation 1 reached the
   branch because of that. **Severity S2.**
3. **No hardware test has been run.** Every interoperability row and every
   performance row of this record is NOT RUN. The record proves the software,
   not the product on a television. **Severity S3** for the milestone (the
   device plan of the M3 gates is not fulfilled), **S0** for the code.
4. **The performance budgets are unmeasured.** They are budgets; a budget is
   a target that a measurement has to prove. **Severity S2.**
5. **The device test plan has no section for the shells.** It names the four
   senders and the five measurements, but no script for "the ten-foot
   interface with a real remote control" — no step for the D-pad, no step for
   the Leanback launcher, no step for the notification of the foreground
   service. The gate of the roadmap "the screenshots and the steps of the
   device test plan are part of the acceptance record" therefore stays
   **open**. **Severity S2.**
6. **The APK was not built.** The task `:app:assembleDebug` needs the NDK of
   the version that the Android Gradle plugin wants; the installed NDK is
   `27.2.12479018`. **Severity S1** for this record — the Kotlin compiles at
   `b0d0f40`, the packaging is a device-test step.
7. **The window of the macOS shell was never built.** There is no macOS SDK
   under Linux; the SwiftUI part of the application is **NOT RUN** — only
   `swiftc -parse` (a syntax check, not a type check) was possible.
8. **The dashboards of the three platforms carry three palettes.** The five
   states are the same everywhere, the colours are not
   (`docs/audit/tv-macos-shell.md` §9.5). **Severity S1.**
9. **The phases "not started" and "stopped" share the same sign** in the
   Android shell (an empty circle); the word distinguishes them, the shape
   does not. The four states of the gate — disabled, advertising, streaming,
   error — have a shape each. **Severity S1.**
10. **Two of the 38 measured colour pairs of the dashboard are the same
    pair.** `apps/web/tools/check-contrast.mjs` measures the badge of **every**
    card (line 223: `for (const badge of values.badges)`), and two of the five
    cards carry the state "advertising" — the same pair is therefore measured
    twice in each theme. The result is not wrong, but the number of
    *distinct* pairs is 36 and not 38. **Severity S0.**
11. **On a short screen the dashboard scrolls by 16 px.** With the five cards
    of M5 the grid of the dashboard needs 16 px of scrolling at 1280 × 720
    (measured: `note the four receivers need 16 px of scrolling at 1280x720
    (5 column(s), 191–736 of 720)`) and 549 px at 768 × 1024 — at that width
    the number of the columns is fractional (`1.6666666666666667 column(s)`).
    The gate of the M3 "all four states are visible at a glance" holds from
    1920 px on. **Severity S1.**

### The exclusions that apply here

`EXC-001` (FairPlay), `EXC-002` (Apple Music in-app audio), `EXC-003` (HDCP),
`EXC-004` (Widevine / PlayReady), `EXC-009` (transcoding) — none of them
belongs to a shell; they are exclusions of the media path. `EXC-005` (cloud
and remote streaming) and `EXC-006` (publishing the receiver to the internet)
are the reason the shell talks to the bridge in process and asks for no
permission beyond the network ones.

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| `./gradlew :app:assembleDebug` (the APK) | the NDK of the version that the Android Gradle plugin wants is not installed (the installed one is `27.2.12479018`); the task `buildRustBridgeDebug` cannot even be created | the matching NDK and a cross-build of the Rust bridge for `aarch64-linux-android` |
| `:app:compileGoogletvDebugKotlin` | the task does not exist — the module declares no `productFlavors` | a decision of the owner (open point 1 of the Android audit); the variant that exists, `debug`, carries the same Kotlin |
| The dashboard of the Android shell on a device or on an emulator | no emulator and no device: the SDK carries `platform-tools`, but no system image and no `emulator` binary | a television with Android TV, or an emulator image |
| The window of the macOS shell | no macOS SDK under Linux — SwiftUI, AppKit and Combine exist only on macOS | a Mac with the Swift 6 toolchain (the script is in §7 of the macOS audit) |
| The type check of the SwiftUI views | the same reason; `swiftc -parse` proves the syntax and nothing else | a Mac |
| The picture of a real sender on the display of the shell | no sender, no display | the device test plan §2 |
| The switch of a protocol seen in the picker of the sender | no sender | the device test plan §2 |
| The memory and the CPU of the shell with a running session | no device, no session | the device test plan §3 |
| The survival of a session in the background and the notification of the service | no device | a script on the device: start a session, send the application to the background, read the notification |
| The screen reader (VoiceOver / TalkBack) | no device | a device with the screen reader switched on |
| A screenshot of the dashboard on a television | no device | a television and a camera |
| `cargo clippy --workspace --all-targets -- -D warnings` | not measured in this cycle — the machine was shared with three other build agents; the run of the whole workspace did not finish in the time of this record | an idle machine; the last green measurement is the one of the M0 record at `7cd6fe0` |
| `cargo test --workspace --offline` | the run did not finish in the time of this record — more than two hours of compilation on a machine with a load average of 8 on two CPUs (§2a) | an idle machine, or a machine with a warm `target/` |

## 10. Decision

**ACCEPTED with a documented limitation, at the revision `c4476db` of
`main`.**

The decision of the first version of this record was **NOT ACCEPTED** at the
revision under test (`2d0c4b4` of `feat/other-devices`), and the reason was
one sentence long and it was measured: **the Kotlin of the Android shell and
the kit of the macOS shell did not compile.** Two errors in the Kotlin
(`ReceiverMapping.kt:91`, `ReceiverDefaults.kt:104`) and two in the Swift
(`ReceiverMapping.swift:83`, `ReceiverDefaults.swift:124`) — all four of them
from the same cause, the new contract of the bridge of the milestone M5.

**That reason does not hold any more, and it is measured on the revision of
this decision:**

| The shell | The command | The result |
|---|---|---|
| Android | `./gradlew :app:compileDebugKotlin` | `BUILD SUCCESSFUL in 4m 48s` |
| Android | `./gradlew :app:testDebugUnitTest` | **39 tests, 0 failures, 0 errors, 0 skipped** |
| Android | `./gradlew :app:assembleRelease` | `BUILD SUCCESSFUL in 12m 39s` — the APK is built |
| macOS | `swift build` | `Build complete!` |
| macOS | `swift test` | **102 tests, 0 failures** — the 101 of the kit and the new test of the checksum of the bindings |

**The APK of the release is built and it carries what a television needs:**

| What | The measurement |
|---|---|
| the package | `org.phairplay.receiver`, `versionName='0.3.0'`, `versionCode='1'` |
| the architecture | `native-code: 'arm64-v8a'` — the library of the bridge is in the package: `lib/arm64-v8a/libphairplay_ffi.so` |
| the launcher of the television | `leanback-launchable-activity: name='org.phairplay.receiver.MainActivity'` — the application appears in the list of the applications of a Google TV and of a Fire TV |
| the permissions | nine, and **no** location and **no** microphone: `INTERNET`, `ACCESS_NETWORK_STATE`, `CHANGE_WIFI_MULTICAST_STATE`, `WAKE_LOCK`, `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `NEARBY_WIFI_DEVICES` (with `neverForLocation`), `ACCESS_WIFI_STATE`, `CHANGE_WIFI_STATE` |
| the signature | `apksigner verify` → **Verifies**, the scheme **v2**, one signer — the RSA key of 4096 bits of the test store |
| the target | `targetSdkVersion='35'`, `minSdkVersion='33'` |

**The signature of the test store is not the signature of a release.** The
key of a release belongs to the owner of the project and does not live in
the repository; the build reads it from the machine (the properties
`phairplay.signing.*` — see `apps/android/app/build.gradle.kts`). The APK of
this measurement is signed with a **test key** that was made for the
measurement and that signs nothing else. The signature of a release is the
trigger of the beta in the release plan — and that trigger has **not**
been reached, because the device test of §9 has not been run.

The four places that the first version of this record named are the four
places that carried the defect; they carry the repair now. The cause was
that the bridge of M5 requires the byte budget of the audio queue and that
neither shell handed it over, and that the fifth protocol was missing in
both. Both are fixed, and the two shells are built and tested by the job
*Shells (Android, macOS)* of the continuous integration.

**What stays open** is the device evidence of §9 — the rows that need a
sender, a television, a device or a Mac. They are **NOT RUN**, and they are
the limitation that travels with this acceptance. The SwiftUI part of the
macOS application is still **NOT RUN**: the machine of this measurement has
Swift for Linux and no macOS SDK.

**The APK that is built here is not a release.** It is signed with a test
key, it was never installed on a device, and the application was never
started. It is the proof that the shell **can** be built for a television —
not the proof that it **works** on one.

**What the record does accept:**

- The deliverable of M3 itself is **complete and proven** at the commit
  `b0d0f40`: the Kotlin compiles (`BUILD SUCCESSFUL in 2m 27s`, measured for
  this record; the audit of the platform worker measured `2m 59s`), its 39
  unit tests are green (0 failures, 0 errors, 0 skipped), the kit of the macOS
  shell runs 101 tests with 0 failures (`Executed 101 tests, with 0 failures
  (0 unexpected)`), and the sources of both shells are **byte-identical** to
  the revision under test. The defect is not in the work of M3 — it is a
  missing follow-up in M5.
- The gates of the roadmap that do not need a device are **green and measured
  by me**: the four states are distinguishable and "stopped" is not "disabled"
  in three shells; the manifest carries **no** location, no microphone and no
  contact permission; the texts of the interface live in the resources — three
  locales with the same 54 keys, 46 strings used by the interface, **no
  literal** in the layer of the interface.
- The boundary check is green (`13` crates, `15` edges, `0` violations), the
  documentation gate is green (`0` broken links, `16` mirrored budgets), the
  format check is green, and the dashboard of the browser passes **35**
  keyboard checks, **30** layout checks and **38** measured colour pairs in a
  real Chromium.

**The conditions for the acceptance:**

1. The four compile errors of §1a are repaired and `./gradlew :app:compileDebugKotlin`
   and `swift test` are green on the branch. The repair is two branches per
   shell; it belongs to the milestone that broke the build.
2. The native shells join the continuous gate — a Gradle task and a Swift
   test in `.github/workflows/ci.yml`. A shell that no gate compiles is a
   shell that breaks silently.
3. The device test plan gets a section for the shells — the D-pad, the
   Leanback launcher, the notification of the foreground service, the
   screenshot — and the result is written into this record.

**Until then this record says exactly what it has:** the code of M3 is
written and tested, the gates that need a build machine are green, the gates
that need a device are **NOT RUN**, and the branch that carries the code does
not build the code.

**Decided by:** the QA & Test Engineer of the PhairPlayRust team
(acceptance record of M3), **2026-10-24**.
