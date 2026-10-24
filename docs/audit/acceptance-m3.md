# The acceptance of M3 — the measured numbers

> **Status:** the acceptance record of M3 is
> [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md). This
> file is the short form: the numbers that were measured, the gates that are
> open, and everything that was **not** measured and why.
>
> **Revision under test:** `feat/other-devices` at `2d0c4b4`. The work of
> M3 itself is the commit `b0d0f40`. The sources of the three shells are
> identical to that commit; while this record was written, other agents
> changed `ROADMAP.md` and added their own acceptance records to the tree —
> none of them touches the shells, the manifest or the resources that this
> record measures.
>
> **Machine:** Linux x86_64, 2 CPUs, 8 GB RAM, shared with three other build
> agents. JDK 17.0.21, Gradle 8.13, Swift 6.0.3, Chromium (Playwright,
> headless), Android SDK `/workspace/.home/tools/android-sdk`.
>
> **The headline:** the three shells of M3 are written and their logic is
> tested, but **at the revision under test the Kotlin of the Android shell and
> the kit of the macOS shell do not compile** — the milestone M5 added a
> protocol kind and a limits field to the bridge and did not update the
> shells. The decision of the record is **not accepted**; the details are in
> §1a of the record.

---

## 1. The numbers that were measured in this cycle

| # | What | Command | Result |
|---|---|---|---|
| 1 | The keyboard of the dashboard | `node apps/web/tools/check-keyboard.mjs` | **35 checks passed, 0 failed** |
| 2 | The layout of the dashboard | `node apps/web/tools/check-layout.mjs` | **30 checks passed, 0 failed** |
| 3 | The contrast of the dashboard | `node apps/web/tools/check-contrast.mjs` | **38 pairs, every pair above its threshold** (36 distinct pairs — the script measures the badge of every card, and two of the five cards carry the state "advertising") |
| 4 | The crate boundaries | `bash scripts/check-crate-boundaries.sh` | 13 crates, 15 internal edges, **0 violations**, exit 0 |
| 5 | The links and the budgets of the documentation | `bash scripts/check-docs-links.sh` | 68 Markdown files scanned, 691 internal links checked, **0 broken links**, 16 mirrored budgets compared, 7 budgets owned alone, exit 0 — the scan includes the two documents of this cycle |
| 6 | The format of the Rust sources | `cargo fmt --all -- --check` | exit 0 |
| 7 | The licence headers | `bash scripts/check-license-headers.sh` | 0 violations, exit 0 |
| 8 | The length of the source files | `bash scripts/check-file-size.sh` | 9 files above the soft limit of 400 lines, **0 above the hard limit of 550**, exit 0 |
| 9 | The Kotlin of the Android shell | `cd apps/android && ./gradlew :app:compileDebugKotlin` | **BUILD FAILED in 5m 12s — 2 errors** |
| 10 | The kit of the macOS shell | `cd apps/macos && swift test` | **error: fatalError — 2 errors** (`switch must be exhaustive`, `missing argument for parameter 'audioQueueBytes'`) |
| 11 | The contrast of the palette of the Android shell | recomputed with the formula of WCAG 2.1 out of `ui/PhairPlayColors.kt` | 11 pairs; the focus frame **6.44:1** against the card and **7.44:1** against the screen; the lowest text pair **6.08:1** — the same numbers as §8.1 of the Android audit |
| 12 | The texts of the interface of the Android shell | `grep` over `ui/` and `res/` | **46** strings used by the interface, **0 literals** in the layer of the interface, **54 keys** in `values`, `values-de` and `values-fr`, the three files identical |
| 13 | The permission list of the Android shell | the source manifest and the merged manifest of the build | **9** platform permissions, **no** location, **no** microphone, **no** contact permission; plus one signature-level permission that AndroidX adds |
| 14 | The Rust workspace | `cargo test --workspace --offline` | **NOT RUN** — the run did not finish on a machine shared with three other build agents (§4) |

### The attribution — the same shell, two revisions

The four shell files that break are byte-identical to the commit of M3. To
prove that the break was **not** in the work of M3, the commit `b0d0f40` was
extracted with `git archive b0d0f40` into a scratch directory and **built
there** — the same toolchain, the same day:

| What | Command | Result |
|---|---|---|
| The Kotlin of the Android shell at `b0d0f40` | `./gradlew :app:compileDebugKotlin` | **`BUILD SUCCESSFUL in 2m 27s`**, `15 actionable tasks: 15 executed`, exit 0 |
| The tests of the Android shell at `b0d0f40` | `./gradlew :app:testDebugUnitTest` | **`BUILD SUCCESSFUL in 2m 34s`**, exit 0 — the result files: `TvFocusGraphTest` 12, `ReceiverMappingTest` 9, `ReceiverDefaultsTest` 7, `ProtocolPhaseTest` 7, `FormattingTest` 4 → **39 tests, `failures="0" errors="0" skipped="0"` in every file** |
| The bridge of `b0d0f40` | `cargo build -p phairplay-ffi --offline` | `Finished dev profile … in 5m 46s`, exit 0 |
| The kit of the macOS shell at `b0d0f40` | `swift test` | **`Executed 101 tests, with 0 failures (0 unexpected) in 0.357 (0.357) seconds`**, exit 0 |

**The same sources, the same day: green at `b0d0f40`, red at `2d0c4b4`.**


### The numbers that come from the audits of the platform workers

These were measured at the revision `7cd6fe0` + the working tree of the
platform workers — that is, at the state of M3 **before** the M5 work — and
they are quoted, not repeated as a measurement of the revision under test.

| What | Command | Result |
|---|---|---|
| The Kotlin of the Android shell at the commit of M3 | `./gradlew :app:compileDebugKotlin --offline --rerun-tasks` | `BUILD SUCCESSFUL in 2m 59s` — [`tv-android-shell-compile.log`](tv-android-shell-compile.log) |
| The tests of the Android shell at the commit of M3 | `./gradlew :app:testDebugUnitTest --offline --rerun-tasks` | `BUILD SUCCESSFUL in 2m 15s`; the result files are still on disk (`apps/android/app/build/test-results/testDebugUnitTest/`, 2026-10-24 09:49): `TvFocusGraphTest` 12, `ReceiverMappingTest` 9, `ReceiverDefaultsTest` 7, `ProtocolPhaseTest` 7, `FormattingTest` 4 — **39 tests, 0 failures, 0 errors, 0 skipped** |
| The lint of the Android module at the commit of M3 | `./gradlew :app:lintDebug` | `BUILD SUCCESSFUL in 4m 22s` — [`tv-android-shell-lint.log`](tv-android-shell-lint.log) |
| The tests of the macOS kit at the commit of M3 | `swift test` | `Executed 101 tests, with 0 failures (0 unexpected) in 0.534 (0.534) seconds`, `real 2m53.714s` — [`tv-macos-shell-swift-test.log`](tv-macos-shell-swift-test.log) |

---

## 2. The two defects of the revision under test

Both shells broke the same way: the **contract of the bridge** changed and
the shells were not updated.

```text
# Android — ./gradlew :app:compileDebugKotlin
e: …/receiver/bridge/ReceiverDefaults.kt:104:17 No value passed for parameter 'audioQueueBytes'.
e: …/receiver/model/ReceiverMapping.kt:91:56 'when' expression must be exhaustive. Add the 'SMART_VIEW' branch or an 'else' branch.
> Task :app:compileDebugKotlin FAILED
BUILD FAILED in 5m 12s

# macOS — swift test
…/ReceiverMapping.swift:83:9: error: switch must be exhaustive
   |         `- note: add missing case: '.smartView'
…/ReceiverDefaults.swift:124:83: error: missing argument for parameter 'audioQueueBytes' in call
error: fatalError
```

**The attribution** — the four shell files that break are byte-identical to
the commit of M3; only the generated bindings changed:

```text
$ git diff --stat b0d0f40..HEAD -- <the four shell files>
(no output)

$ git diff --stat b0d0f40..HEAD -- apps/android/app/src/main/java/uniffi/ apps/macos/Sources/PhairPlayFFI/
 …/java/uniffi/phairplay_ffi/phairplay_ffi.kt | 13 +++++++++++-
 …/macos/Sources/PhairPlayFFI/phairplay_ffi.swift | 24 ++++++++++++++++++++++
```

The diff of the bindings is `+ SMART_VIEW;` and `+ audioQueueBytes`. Both
come from `86c1b81` ("feat(m5): Samsung Smart View …").

**Why no gate caught it:** `.github/workflows/ci.yml` has four jobs —
*Quality Gate*, *Licenses*, *Security Audit*, *Structure*. **No Gradle task
and no Swift test is part of it**, and the workflow triggers on `push` to
`main` and on `pull_request` to `main` only.

---

## 3. The gates of the roadmap that are open

| Gate of `ROADMAP.md` §M3 | State | Why |
|---|---|---|
| Functional — the state of every protocol is visible in both shells, the four states are distinguishable, "stopped" is not "disabled" | **green in the code, NOT RUN on a device** | `ProtocolPhaseTest` (7), `ShellStatusTests` (12), `ShellToneTests` (7), the resources of three locales; the device part needs a television |
| Functional — a protocol can be switched on and off from the shell and the change is visible in the picker of the sender | **NOT RUN** | needs a sender and a device — `docs/guides/DEVICE_TEST_PLAN.md` §2 |
| Technical — the shell reaches the core only through the bridge | **green** | `scripts/check-crate-boundaries.sh` → 0 violations; the shells import the generated bindings only |
| Technical — the manifest has no location, no microphone and no contact permission | **green** | the merged manifest of the build — nine platform permissions, none of them one of the three |
| Performance — the memory and the CPU of the shell with a running session | **NOT RUN** | needs a device and a session — the plan §3 |
| Usability — a session survives the backgrounding, the notification is present | **NOT RUN** | needs a device |
| Usability — the texts of the interface are in the resources | **green** | 46 strings, 0 literals, 54 keys in three locales; `ShellCatalogueTests` (6) for the macOS catalogue |
| Security — no permission that the receiver does not need | **green** | the same manifest; the one exception is the signature-level permission that AndroidX adds |
| Interoperability — the picture of a real sender reaches the display of the real device | **NOT RUN** | needs hardware |
| Documentation — the screenshots and the steps of the device test plan are part of the acceptance record | **open** | the plan has **no section for the shells**, and no screenshot can be taken without a device |

---

## 4. The Rust workspace

**NOT RUN in this cycle.** `cargo test --workspace --offline` was started at
21:54 on the revision under test and did not finish: the machine was shared
with three other build agents (load average 8 on two CPUs), the compilation
of the test binaries of the whole workspace needed more than two hours, and
the run was stopped while it was still linking (`phairplay_ffi`,
`phairplay_web`, `phairplay_core`).

The last green measurement of the Rust side is the acceptance record of M0 at
the revision `7cd6fe0`: **1054 passed, 0 failed**. It is a measurement of
another revision and is not repeated here as one of this revision.

---

## 5. What was NOT measured, and why

| What | Why |
|---|---|
| The APK (`./gradlew :app:assembleDebug`) | the NDK of the version that the Android Gradle plugin wants is not installed (the installed one is `27.2.12479018`); the task `buildRustBridgeDebug` cannot even be created |
| `:app:compileGoogletvDebugKotlin` | the task does not exist — the module declares no `productFlavors` |
| The dashboard of the Android shell on a device or on an emulator | no emulator, no device, no system image in the SDK |
| The window of the macOS shell and the type check of its views | no macOS SDK under Linux — SwiftUI, AppKit and Combine exist only on macOS; `swiftc -parse` proves the syntax and nothing else |
| A screenshot of any shell | no device |
| The picture of a real sender, the picker of the sender, the switch seen from the sender | no sender, no device |
| The memory and the CPU of the shell with a running session | no device, no session |
| The screen reader (VoiceOver / TalkBack) | no device |
| `cargo clippy --workspace --all-targets -- -D warnings` | not measured in this cycle — the machine was shared with three other build agents; the last green measurement is the one of the M0 record at `7cd6fe0` |
| `cargo test --workspace --offline` | the run did not finish in the time of this record — more than two hours of compilation on a machine with a load average of 8 on two CPUs; see §4 | an idle machine, or a machine with a warm `target/` |
| The licence gate (`cargo-deny`) | the tool is not installed on this machine — `bash scripts/audit-licenses.sh --check-only` → `NOT RUN: cargo-deny is not installed on this machine.`, exit 2 |

---

## 6. What the owner has to do

1. **Repair the four compile errors** — a branch for `SMART_VIEW` and the new
   limit in `protocolIdOf` / `protocolId(of:)` and in the reader of the
   defaults, in the Kotlin and in the Swift. The repair belongs to the
   milestone that broke the build.
2. **Put the shells into the gate** — a Gradle task and a Swift test in
   `.github/workflows/ci.yml`, and a trigger that also covers the feature
   branches.
3. **Give the device test plan a section for the shells** — the D-pad, the
   Leanback launcher, the notification of the foreground service, the
   screenshot — and write the result into
   [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md).
