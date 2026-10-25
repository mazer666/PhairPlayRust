# PhairPlayRust 0.3.0-alpha.1 — the release notes

> **What this document is:** the description of the release **v0.3.0-alpha.1**
> — what it contains, what was measured before it was published, and what it
> does **not** claim.
>
> **Date:** 2026-10-25 — the date of the tag `v0.3.0-alpha.1`.
> **Revision:** the commit of the tag on the branch `main`.
> **Kind:** an **alpha**. The code of the milestones is delivered and the
> gate of the project is green; the evidence that only a device can produce
> is missing.
>
> **The rule that governs this document:** a release may only claim what an
> acceptance record proves. A claim that no record proves is written down as
> **NOT RUN** with the exact reason.

---

## 1. The short version

PhairPlayRust is a receiver for AirPlay 2, Miracast, Google Cast, DLNA/UPnP
and Samsung Smart View. It runs on the local network, without an account,
without telemetry and without a cloud.

**v0.3.0-alpha.1 is the first release in which all five protocols run at the
same time.** The release v0.2.0 carried the control plane of the five
protocols, but the receiver lost the fifth one to a conflict of the
discovery port: the DLNA renderer and the Samsung Smart View responder both
wanted the SSDP group, and Linux allows one socket per multicast group. The
receiver now owns **one** listener for the whole process, and every service
registers itself with it.

This release carries:

- **five protocols that run together** — the run of the receiver reports
  **5 of 5 protocols are running**;
- the **native shells that compile** — the Kotlin of the Android shell and
  the kit of the macOS shell, with their tests green;
- the **plan of the releases** — when a release happens, what triggers it,
  and which platform ships a binary with it;
- the **headless receiver** as a binary for the server platforms.

**What it does not carry:** a picture from a Galaxy device. The message
format of the Smart View channel is proprietary and is not implemented
(`EXC-011`). It is written down as a limit, not as a defect.

**And it does not carry a binary of a shell.** The rule of the project is
that a binary is published when **its own** device test is green, and no
device test of a shell has been run (§4).

---

## 2. What is in this release

### The five protocols run together

The receiver speaks five protocols and discovers two of them — DLNA and
Samsung Smart View — over the same SSDP multicast group. Linux allows only
one socket per multicast group, and `SO_REUSEADDR` does not lift that
restriction, because it is the *membership* and not the port that is
exclusive.

The process therefore owns **one** SSDP listener
([`../adr/ADR-0007-one-ssdp-listener-for-the-whole-receiver.md`](../adr/ADR-0007-one-ssdp-listener-for-the-whole-receiver.md)),
and every service registers itself with it. The listener asks every service
whether a message is a search that it answers:

| The sender searches for | the renderer answers | the framework answers |
|---|---|---|
| `urn:schemas-upnp-org:device:MediaRenderer:1` | yes | no |
| `urn:samsung.com:service:MultiScreenService:1` | no | yes |
| `ssdp:all` | yes, with the type of the renderer | yes, with the type of the framework |

The wildcard is answered once per service type, as the UPnP device
architecture asks. A service that answers for a type it does not have
appears in the device list of every control point on the network — so the
renderer does not answer for the framework and the framework does not answer
for the renderer.

### The user decides which protocols run

The switch of a protocol is the key `enabled` in its section of the
configuration:

```toml
[protocols.dlna]
enabled = false
```

**Switching one of the two protocols off keeps the other one running.** The
listener belongs to the process, not to one protocol: a service that stops
leaves the registry, and the listener stays and keeps answering for the
services that are still registered. The help of the program names the
switch.

### The native shells compile and are tested

The Kotlin of the Android shell and the kit of the macOS shell are built
and their tests are green — **39 tests** on Android, **102 tests** on macOS,
0 failures each. The continuous integration builds both on every push, in
the job *Shells (Android, macOS)*.

### The plan of the releases

The roadmap now says when a release happens, what triggers it and which
platform ships a binary with it. The trigger is a **state** and not a date:
the project promises no date, and a condition can be checked by anybody
while a date can only be waited for.

---

## 3. The measured evidence

Everything in this section was measured on the tree of this release. The
commands are the ones that were run.

| What | The command | The result |
|---|---|---|
| The workspace compiles | `cargo build --workspace` | finished, 0 errors |
| The tests of the workspace | `cargo test --workspace` | **1241 passed, 0 failed, 0 ignored** |
| The format | `cargo fmt --all -- --check` | clean |
| The lint | `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| The documentation | `cargo doc --workspace --no-deps` | 0 warnings |
| The licence gate | `cargo deny 0.20.2` against `deny.toml` | `licenses ok` |
| The crate boundaries | `bash scripts/check-crate-boundaries.sh` | 13 crates, 15 internal edges, **0 violations** |
| The length of the sources | `bash scripts/check-file-size.sh` | **0** above the hard limit |
| The links of the documentation | `bash scripts/check-docs-links.sh` | 73 files, 765 internal links, **0 broken** |
| The version of the workspace | `bash scripts/check-version-consistency.sh` | `VERSION`, `Cargo.toml`, the Android shell and the changelog carry **0.3.0** |
| The gate as a whole | `bash scripts/local-ci.sh` | **green** — 0 failed checks, 0 NOT RUN |
| The macOS shell | `swift build` and `swift test` in `apps/macos` | `Build complete!` — **102 tests, 0 failures** |
| The Android shell | `./gradlew :app:testDebugUnitTest` in `apps/android` | `BUILD SUCCESSFUL` — **39 tests, 0 failures, 0 errors, 0 skipped** |
| The continuous integration | the run of the commit of this release | **all five jobs green** — *Quality Gate*, *Licenses*, *Security Audit*, *Structure*, *Shells (Android, macOS)* |
| The receiver | `phairplay-web --profile generic --no-dashboard --port 7879` | **5 of 5 protocols are running** |
| The discovery in the network | an `M-SEARCH` for the renderer, for the framework and for `ssdp:all` | the renderer answers for the renderer, the framework for the framework, and both for the wildcard — **0 foreign answers** |

**The two defects that the release repairs were proved against the old
code**, not asserted:

| The defect | The proof |
|---|---|
| The registry of the shared listener was keyed by the unique name of the device, so the second service pushed the first out of it | the test `two_services_of_the_same_device_stay_in_the_registry` fails with the old code and passes with the new one |
| The bindings of the bridge carried a stale checksum, so every native shell died at the start | the test `BindingChecksumTests.testTheBindingsMatchTheLibrary` crashes with the message `UniFFI API checksum mismatch` with the old bindings and passes with the generated ones |
| The new gate of the version catches a version that differs in one of the four places | `VERSION` was set to `0.9.9` on purpose: the gate answers with exit code 1 and names every place that differs; after the repair it is green again |

---

## 4. What is **not** in this release

| What | Why | What would be needed |
|---|---|---|
| **A binary of the Android shell** | the device test of the shell has not been run. The rule is that a binary is published when **its own** device test is green | the device test plan on a Google TV or a Fire TV |
| **A binary of the macOS shell** | the same; and the SwiftUI part of the application needs a Mac | the device test plan on a Mac |
| **The picture of a Galaxy device** | the message format of the Smart View channel is proprietary and is not implemented (`EXC-011`) | the documentation of the format by Samsung, or another implementation under a licence that allows the port |
| **The device test plan of the Smart View scenario** | the plan has no section for it — the sections 2.1 to 2.4 cover AirPlay 2, Miracast, Cast and DLNA | the plan, extended by the owner |
| **A performance budget that is measured on a device** | the measurement needs a running receiver on a real device with a real sender | the device test plan, §3 |
| **The interoperability with a real sender** | no sender, no television and no device were available | the device test plan, §2 |

**No row of the device test plan has been executed.** The alpha exists
because the software is measured and the device evidence is not.

---

## 5. The limits of the product

The limits are not a postscript of these notes; they are part of the
product and they are listed in one place:
[`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) carries the exclusion list
`EXC-001` to `EXC-011`. The three that a user meets first:

| The limit | The identifier |
|---|---|
| FairPlay DRM content cannot be played — no open-source receiver can decrypt it | `EXC-001` |
| The message format of the Smart View channel is proprietary — the receiver finds the sender and holds the channel, but no picture flows | `EXC-011` |
| The media path of the platform belongs to the native shell — the engines deliver encoded frames, the shell decodes them | the acceptance record of M3 |

---

## 6. The upgrade from v0.2.0

The receiver of v0.2.0 was published as a **pre-release**: the code of the
five milestones was delivered, but the fifth protocol did not start in the
default configuration, and the native shells did not compile.

| What changed | What the user has to do |
|---|---|
| the fifth protocol now starts — the receiver reports **5 of 5 protocols are running** | nothing; the conflict of the discovery port was in the receiver, not in the configuration |
| the native shells compile again | nothing for the receiver; a user who built the shells of v0.2.0 by hand has to build them again |
| the version of the workspace is `0.3.0` and says so in `VERSION`, in `Cargo.toml`, in the Android shell and in the changelog | nothing |
| the configuration of the protocol sections gained `[protocols.smartview]` | a configuration file that is written by hand gains three keys; a profile of the repository has them already |

---

## 7. The binaries of this release

**This release ships the headless receiver for the server platforms.** The
plan of the releases in [`../../ROADMAP.md`](../../ROADMAP.md) names the
artefacts and the platforms:

| Platform | The artefact | The trigger that it is published |
|---|---|---|
| **Linux** (x86-64, aarch64) | `phairplay-web` — the headless receiver with the dashboard | the alpha: the code is delivered and the gate is green |
| **Windows** (x86-64) | `phairplay-web.exe` | the alpha |
| **Android** (arm64-v8a, API 33+) | the APK with the native bridge | the **beta**: the device test of the shell is green |
| **macOS** (13+, Apple silicon) | the `.app` | the **beta**: the device test of the shell is green |

**Why the shells wait.** A binary is a build artefact of a platform, of a
toolchain and of a licence, and a binary that nobody checked is a promise
that nobody made. The trigger of a binary is **its own device test**, not
the release as a whole — and no device test of a shell has been run.

The notes name the toolchain of every binary, because a binary without its
toolchain is not reproducible.

---

## 8. How to check this release yourself

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
git checkout v0.3.0-alpha.1

# the shared library of the bridge — the test of the foreign ABI loads it,
# and `cargo test` builds the test harness, not the shared library
cargo build -p phairplay-ffi

# the workspace compiles and its tests are green
cargo test --workspace

# the lint and the format of the gate
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# the structural gate and the links of the documentation
bash scripts/check-crate-boundaries.sh
bash scripts/check-file-size.sh
bash scripts/check-license-headers.sh
bash scripts/check-docs-links.sh
bash scripts/check-version-consistency.sh

# the gate as a whole, and the proof that the gate refuses its violations
bash scripts/local-ci.sh
bash scripts/local-ci.sh --self-test

# the receiver itself
cargo build --release -p phairplay-web
./target/release/phairplay-web --profile generic --no-dashboard --port 7879
```

The expected output of the last command is:

```text
phairplay-web: the receiver starts 5 protocols:
  AirPlay 2        advertising
  Miracast         advertising
  Google Cast      advertising
  DLNA / UPnP      advertising
  Samsung Smart View advertising
phairplay-web: 5 of 5 protocols are running.
```

---

## 9. The provenance of these notes

Every number in this document comes from a command that was run on the tree
of the release. The acceptance records of the milestones carry the detailed
evidence:
[`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) to
[`../acceptance/M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md).

The machine-readable state of the project is
[`../../project/project.yaml`](../../project/project.yaml); the narrative of
the milestones is in [`../../ROADMAP.md`](../../ROADMAP.md).

Nothing in this document is a claim that was not measured. Where a thing was
not measured, it says **NOT RUN** and where to measure it.
