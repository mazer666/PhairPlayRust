# The acceptance of M4 — the measured numbers, the open gates and what was not measured

> **Status:** binding. This is the report behind
> [`../acceptance/M4-ACCEPTANCE.md`](../acceptance/M4-ACCEPTANCE.md). It holds
> the raw numbers of the run, the gates of the milestone that are still open,
> and the list of the checks that were **not** run — with the exact reason.
>
> **The rule of the milestone:** a number without a command and a revision is
> not a number. A check that did not run is **NOT RUN**, never a success.

---

## 1. What was measured

| Item | Value |
|---|---|
| Revision | `2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` (`feat/other-devices`, *"fix(ci): the licence gate has to pass on a clean checkout"*, 2026-10-24 13:35:31 +0000) |
| Toolchain | `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Host | Linux x86_64, 2 CPUs, 8 GB RAM |
| Target directory | `/workspace/target-qa4w` |
| Worktree | `/workspace/scratch/m4-acc` |
| Date | 2026-10-24 |

**Why a worktree.** The working tree of `/workspace/PhairPlayRust` was being
modified by another agent while this acceptance was written. The tree carried
a half-finished refactoring of `engines/dlna/src/ssdp.rs` that did **not**
compile:

```text
error[E0560]: struct `SsdpService` has no field named `primary_target`
  --> engines/dlna/src/ssdp.rs:141:9
```

A number measured on that tree would have described a tree that does not
exist. Every number below therefore belongs to the **committed** revision,
measured in a worktree — the same method that
[`2026-10-24-baseline.md`](2026-10-24-baseline.md) §1 uses.

---

## 2. The test suite

### 2.1 The command

```text
cd /workspace/scratch/m4-acc
export HOME=/workspace/.home PATH=$HOME/.cargo/bin:$PATH
CARGO_TARGET_DIR=/workspace/target-qa4w cargo build --offline -p phairplay-ffi
CARGO_TARGET_DIR=/workspace/target-qa4w cargo test  --offline --workspace --no-fail-fast
```

### 2.2 The result

| Metric | Value |
|---|---:|
| Tests in total | **1224** |
| Passed | **1224** |
| Failed | **0** |
| Ignored | **0** |
| Measured | **0** |
| Filtered out | **0** |
| Unit tests | 998 |
| Integration tests | 82 |
| Doctests | 144 |
| Test binaries | 38 |
| Run time of the 38 test binaries (the sum of every `finished in`) | **62.35 s** |
| `cargo build -p phairplay-ffi` | `Finished 'dev' profile … in 8m 41s` |
| `cargo test` — compilation of the test targets | `Finished 'test' profile … in 13m 41s` |
| Wall clock of the whole chain | **24m 40.665 s** |

The exit code of the chain was **0**.

### 2.3 The per-crate result

| Crate | Unit | Integration | Doctests |
|---|---:|---:|---:|
| `phairplay-core` | 94 | — | 19 |
| `phairplay-plist` | 84 | — | 7 |
| `phairplay-h264` | 51 | — | 9 |
| `phairplay-aac` | 24 | — | 6 |
| `phairplay-alac` | 61 | — | 5 |
| `phairplay-discovery` | 52 | — | 8 |
| `phairplay-airplay` | 230 | 9 | 30 |
| `phairplay-cast` | 80 | 6 | 11 |
| `phairplay-dlna` | 119 | 21 | 10 |
| `phairplay-miracast` | 35 | — | 3 |
| `phairplay-smartview` | 75 | 24 | 13 |
| `phairplay-ffi` | 43 | 22 | 14 |
| `phairplay-web` | 50 | — | 9 |
| **Total** | **998** | **82** | **144** |

### 2.4 The history of the number

| Revision | Tests | Passed | Failed | Ignored | Source |
|---|---:|---:|---:|---:|---|
| `6947f35…` (the untouched commit) | 961 | 948 | 13 | 0 | [`2026-10-24-baseline.md`](2026-10-24-baseline.md) §3 — and the run did not terminate before the deadlock of `phairplay-cast` was repaired |
| after the work of the QA of M0 | 1054 | 1054 | 0 | 0 | [`2026-10-24-baseline.md`](2026-10-24-baseline.md) §6 |
| `2d0c4b4a…` (M4) | **1224** | **1224** | **0** | **0** | this report §2.2 |

The 170 tests between M0 and M4 are the hardening of the engines (the
negative tests of DLNA, Smart View and Cast), the fifth protocol and the
headless build.

### 2.5 The prerequisite that is not obvious

`cargo test --workspace` **alone** is red on this revision. The round-trip
test of the bridge loads the shared library of the bridge, and `cargo test`
builds the test harness of the library, not its `cdylib`:

```text
running 2 tests
test a_foreign_caller_round_trips_through_the_c_abi ... FAILED
test the_bindings_of_every_shell_can_be_generated ... FAILED

thread 'a_foreign_caller_round_trips_through_the_c_abi' panicked at platform/phairplay-ffi/tests/foreign_abi_roundtrip.rs:50:5:
the shared library of the bridge must be built next to the test binary

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

This is **not** a defect of the product: the continuous integration has the
step *"The shared library and the binding generator of the bridge"*
(`cargo build -p phairplay-ffi`) before *"The tests of the workspace"*, and
`scripts/local-ci.sh` has the same step. A contributor who runs
`cargo test --workspace` without it will see two failures that say nothing
about the code. **The instruction belongs in `CONTRIBUTING.md`.**

### 2.6 Nothing is skipped

- `grep -rn "#\[ignore"` over `core/`, `engines/`, `formats/`, `platform/`,
  `apps/` → **no hit**.
- `grep -rn "todo!("` → **no hit**.
- `grep -rn "unimplemented!("` → **no hit**.
- All 38 `test result` lines of the run read `0 ignored` and
  `0 filtered out`.

---

## 3. The gates of the milestone, one by one

The gates are the section *"Gates"* of
[`../../ROADMAP.md`](../../ROADMAP.md), milestone **M4**. Every line gets a
result and an evidence.

| # | The gate line of `ROADMAP.md` | Result | The evidence |
|---|---|---|---|
| 1 | *Functional:* every requirement is either demonstrated by a test or named in the known limitations | **partly open** | The requirements of the hardening, of the gate and of the licence are demonstrated (§4, §5 of the acceptance record). The requirements that need a sender — `FR-008` … `FR-031`, `NFR-INT-001` … `NFR-INT-003`, `NFR-PERF-001` … `NFR-PERF-005` — are named in the known limitations of the record (§8) and in the device test plan. |
| 2 | *Technical:* `cargo build`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` are green | **green** | build: `Finished 'dev' profile … in 8m 41s`, exit 0. test: **1224 passed, 0 failed**. clippy: exit 0, no warning. fmt: exit 0, no output. |
| 3 | *Technical:* the documentation check, the boundary check and the licence check are green | **green** | `check-docs-links.sh` → 488 links, 0 broken, exit 0. `check-crate-boundaries.sh` → 13 crates, 15 edges, 0 violations, exit 0. `audit-licenses.sh` → cargo-deny **0.20.2** → `bans ok, licenses ok, sources ok`, exit 0. |
| 4 | *Technical:* no test is skipped, ignored or commented out; no `todo!()`, no `unimplemented!()` | **green** | §2.6 of this report. |
| 5 | *Technical:* no file is longer than the hard limit | **green** | `check-file-size.sh` → `Files measured: 365 / Above the hard limit: 0`, exit 0. Eight files are above the **soft** limit of 400 — a target, not a rule. |
| 6 | *Performance:* the measured latency, frame rate, memory and CPU are inside the budgets | **NOT RUN** | Needs a device test. The budgets live in `config/performance-budgets.toml`; the commands are in `docs/guides/DEVICE_TEST_PLAN.md` §3. |
| 7 | *Usability:* the dashboard and the shells reach the common tasks in a comparable number of steps | **open** | Needs a device test on both sides. |
| 8 | *Usability:* the receiver is usable without a screen | **green** | `docs/audit/other-devices-guide.md` §4 — the start report names every protocol with its state and the reason of every failure; the API answers `/api/status`, `/api/sessions`; the process leaves with the status 0 on `SIGTERM`. The tests of the command line (`apps/web/src/cli.rs`, 21) and of the composition root (`apps/web/src/receiver.rs`, 4) are green. |
| 9 | *Security:* every attack of the threat model has a negative test that really runs it | **green** | The table in §6 of the acceptance record names **64** negative tests, one per attack of `SECURITY.md` §1. Every one of the 64 names was checked against the tree of the revision — **none is invented**. |
| 10 | *Security:* the mask of the logs is proven by a test that searches a rendered line | **NOT PROVEN — open** | The helpers are proved (16 unit tests + 6 doctests in `core/phairplay-core/src/privacy.rs`) and every call site uses them — but **no test renders a log line**. `tracing-subscriber` is in the tree only in `apps/web/src/main.rs`, never in a test. The method of verification that `NFR-PRIV-002` names does not exist in the tree. **S1** of the acceptance record. |
| 11 | *Security:* the dependency licences are compatible with the AGPL, and the attribution of the ALAC port is in place | **green** | cargo-deny 0.20.2 → `bans ok, licenses ok, sources ok`. `THIRD_PARTY.md` names the origin, the licence and the eight derived files of the ALAC port; every file of `formats/alac/src/` carries an `ATTRIBUTION:` header; the licence texts are in `LICENSES/`. |
| 12 | *Security:* no telemetry, no account, no outbound connection outside the local subnet | **green** | `config/security-policy.toml`; `Cargo.lock` has no telemetry client and no outbound HTTP client — the only HTTP stack is the **server** of the dashboard (`hyper`/`axum`). |
| 13 | *Interoperability:* the device test plan is executed and every sender of the matrix has a result | **NOT RUN** | No sender, no television, no second device in the environment. Every row of `docs/guides/DEVICE_TEST_PLAN.md` §2 is open. |
| 14 | *Interoperability:* the limitations of the interoperability are written in the user-facing documentation | **green** | The exclusion list of `REQUIREMENTS.md` (`EXC-001` … `EXC-011`) and the section of `README.md`. |
| 15 | *Documentation:* every document of the set is current, no statement contradicts the code | **green with one exception** | `check-docs-links.sh` → 488 links, 0 broken; 16 budgets compared with the typed model, 7 owned alone. **One exception:** `cargo doc --workspace --no-deps` emits one warning (§6.1). |
| 16 | *QA evidence:* the acceptance record of the workspace, with the evidence of every gate | **green for M4** | This record and the acceptance record of M4. The records of M1, M2 and M3 belong to other milestones. |

---

## 4. The structural gates, the output of this run

Every command was run in the worktree of the revision
`2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0`.

| Command | The numbers of the output | Exit |
|---|---|---:|
| `bash scripts/check-crate-boundaries.sh` | `Crates checked: 13 / Internal edges: 15 / Violations: 0` | 0 |
| `bash scripts/check-crate-boundaries.sh --self-test` | 5 cases, each `exit code 1`; `OK: the gate fails on every violation and passes on the clean tree.` | 0 |
| `bash scripts/check-file-size.sh` | `Files measured: 365 / Generated files skipped: 2 / Above the soft limit: 8 / Above the hard limit: 0` | 0 |
| `bash scripts/check-license-headers.sh` | `Member manifests checked: 13 / Source files checked: 251 / Derived files checked: 10 / Violations: 0` | 0 |
| `bash scripts/check-docs-links.sh` | `Markdown files scanned: 59 / Internal links checked: 488 / Broken links: 0 / Budgets compared: 16 / Budgets owned alone: 7` | 0 |
| `bash scripts/audit-licenses.sh` | cargo-deny **0.20.2** → `bans ok, licenses ok, sources ok` | 0 |
| `cargo audit --ignore RUSTSEC-2023-0071` | 274 crate dependencies, 0 vulnerabilities, 1 allowed warning (`RUSTSEC-2026-0374`) | 0 |
| `cargo fmt --all -- --check` | no output | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | no warning | 0 |
| `cargo doc --workspace --no-deps` | **1 warning** — see §6.1 | 0 |
| `bash scripts/local-ci.sh --self-test` | `rules proven: 4 / failed rules: 0` — see §5 | 0 |

---

## 5. The proof that the gate is a gate

A gate that has never refused anything is not a gate. Two negative proofs
were run.

### 5.1 The boundary gate — five deliberate violations

`bash scripts/check-crate-boundaries.sh --self-test` copies the manifests of
the workspace into a temporary directory, injects one violation per case and
proves that the gate answers with the exit code **1**. The repository itself
is never modified.

```text
==> Self test of the crate boundary gate
    sandbox: /tmp/tmp.XmZ3nY4ksE

    [ ok ] the unmodified copy is green
    [ ok ] formats -> engines: the gate answered with exit code 1
           | !!! phairplay-plist (formats) depends on engines/airplay (engines) — a format crate is pure: bytes into bytes, no protocol, no core
           | Violations          : 1
           | FAILED: a crate depends on a layer it may not depend on.
    [ ok ] core -> formats: the gate answered with exit code 1
           | !!! phairplay-core (core) depends on formats/plist (formats) — the core is the shared vocabulary and depends on no other crate of the workspace
    [ ok ] engines -> platform: the gate answered with exit code 1
           | !!! phairplay-airplay (engines) depends on platform/phairplay-ffi (platform) — an engine may not depend on the platform bridge
    [ ok ] platform -> formats: the gate answered with exit code 1
           | !!! phairplay-ffi (platform) depends on formats/plist (formats) — the platform bridge may not depend on a format crate directly
    [ ok ] app-backend -> formats: the gate answered with exit code 1
           | !!! phairplay-web (app-backend) depends on formats/plist (formats) — the local web backend may not depend on a format crate directly

OK: the gate fails on every violation and passes on the clean tree.
```

### 5.2 The whole gate — four deliberate violations

`bash scripts/local-ci.sh --self-test` (8m 25.779 s, exit 0) breaks the lint,
the length of the files, the licence headers and the links of the
documentation in turn, and restores every file with a `trap`.

```text
==> The self-test of the gate of PhairPlayRust
    Every rule is broken on purpose, the gate has to go red, the
    break is undone and the gate has to go green again.

--- 1. the lint of the workspace: an unwrap() in the code
    clippy with an unwrap() -> exit code 101
    OK: the lint refused the unwrap().
    clippy after the repair -> exit code 0
    OK: the gate is green again.

--- 2. the length of the source files: a file above the hard limit
    the file-size check with a long file -> exit code 1
    OK: the check refused the long file.
    the file-size check after the repair -> exit code 0
    OK: the gate is green again.

--- 3. the licence headers: a literal licence in a member manifest
    the licence-header check with a literal licence -> exit code 1
    OK: the check refused the literal licence.
    the licence-header check after the repair -> exit code 0
    OK: the gate is green again.

--- 4. the links of the documentation: a link to a file that does not exist
    the link check with a broken link -> exit code 1
    OK: the check refused the broken link.
    the link check after the repair -> exit code 0
    OK: the gate is green again.

==> The self-test of the gate
    rules proven   : 4
    failed rules   : 0

OK: every rule of the gate refused its violation and was green again after the repair.
```

The fifth rule — the licence audit — has its own proof, because the rule is
not *"a file is wrong"* but *"the tool has the wrong version"*. A machine
with another cargo-deny reports **NOT RUN** with the reason and the exit code
**2**; it never reports a false error and never reports a green run for a
configuration it did not read
([`2026-10-24-gate-self-test.md`](2026-10-24-gate-self-test.md) §4).

`git status --porcelain` was empty after both self-tests.

---

## 6. What the run found

### 6.1 One rustdoc warning that the gate does not catch

```text
warning: unresolved link to `ProtocolEngine`
  --> engines/smartview/src/lib.rs:56:26
   |
56 | //! | [`engine`] | The [`ProtocolEngine`] implementation: the state machine and the lifecycle. |
   |                          ^^^^^^^^^^^^^^ no item named `ProtocolEngine` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`
   = note: `#[warn(rustdoc::broken_intra_doc_links)]` on by default

warning: `phairplay-smartview` (lib doc) generated 1 warning
```

The trait lives in `phairplay_core::protocol::ProtocolEngine`, and the
example two lines below uses exactly that path. The fix is a path.

**Why the gate does not see it:** the job *Quality Gate* runs
`cargo doc --workspace --no-deps` **without** `RUSTDOCFLAGS="-D warnings"`,
so the warning is printed and the step is green. The record of M0 states
*"0 warnings"* for the same command
([`2026-10-24-baseline.md`](2026-10-24-baseline.md) §6) — that is no longer
true. **Recommendation for the owner:** add `RUSTDOCFLAGS: "-D warnings"` to
the environment of the workflow, so that the gate is red on the next broken
link.

### 6.2 The mask of the log is not proved on a rendered line

`NFR-PRIV-002` in `REQUIREMENTS.md` and the gate of M4 in `ROADMAP.md` both
name *"a test that searches a rendered log line for a complete name or
address"*. **Such a test does not exist in the tree.**

What exists:

- 16 unit tests and 6 doctests in `core/phairplay-core/src/privacy.rs` that
  assert on the **return value** of `mask_name`, `mask_ip`,
  `mask_socket_addr`, `mask_host`, `mask_url` and `mask_optional_name`.
- The call sites of the log, all of which use a helper:
  `engines/airplay/src/engine/connection.rs`, `engines/cast/src/peer.rs`,
  `engines/cast/src/engine.rs`, `engines/miracast/src/engine.rs`,
  `engines/dlna/src/gena.rs`.

What does **not** exist:

- A test that installs a `tracing` subscriber and captures a line.
  `tracing-subscriber` appears in the tree **only** in
  `apps/web/src/main.rs` — in the production code, never in a test.

**The gap is therefore a verification gap, not a known leak.** The privacy
promise is *believed* to hold because every call site uses a helper; it is
not *demonstrated*. The recommended test: a `tracing_subscriber` with a
`MakeWriter` that writes into a buffer, a call of the engine that logs a
peer, and an assertion that the rendered line does not contain the complete
name or the complete address.

### 6.3 A stale comment in the dashboard

`apps/web/assets/protocol-cards.js` opens with *"The four receiver cards of
the dashboard"*, but the list `PROTOCOLS` of `apps/web/assets/api.js` has
**five** entries — `airplay`, `miracast`, `cast`, `dlna`, `smartview` — and
the glyph `#glyph-smartview` exists in `apps/web/index.html`. The panel
builds five cards; the comment describes four. A documentation defect, S4.

### 6.4 The working tree of the moment did not compile

See §1. The measurement of this report belongs to the committed revision. The
owner has to re-run the gate on the merged tree before a release.

---

## 7. The release evidence — the continuous integration

The gate ran on the GitHub-hosted runners on exactly the revision of this
report.

| Item | Value |
|---|---|
| Run | **40098804432** — <https://github.com/mazer666/PhairPlayRust/actions/runs/40098804432> |
| Head SHA | `2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` |
| Event | `push` on `main` |
| Started / finished | 2026-10-24 13:35:38Z → 13:38:51Z |
| Conclusion | **success** |

| Job | Job id | Steps | Conclusion |
|---|---:|---:|---|
| Quality Gate (format, lint, tests, documentation) | 120886521026 | 10 | **success** |
| Licenses (cargo-deny, pinned) | 120886521011 | 6 | **success** |
| Security Audit (cargo-audit) | 120886520855 | 7 | **success** |
| Structure (boundaries, file size, licence headers, documentation) | 120886521060 | 7 | **success** |

**The limit of this evidence.** The conclusions of the jobs and of every step
were read from the GitHub API
(`GET /repos/mazer666/PhairPlayRust/actions/runs/40098804432/jobs`). The
**raw log of a job is not downloadable** by this account:

```text
GET /repos/mazer666/PhairPlayRust/actions/jobs/120886521026/logs
HTTP 403
{"message": "Must have admin rights to Repository.", "documentation_url": "https://docs.github.com/rest/actions/workflow-jobs#download-job-logs-for-a-workflow-run", "status": "403"}
```

The runner therefore proves that the *gate* is green; the **local** run of
§2 proves *what* is green and with which numbers.

### 7.1 The history of the gate on the runner

| Run | Head SHA | Conclusion |
|---|---|---|
| 40098804432 | `2d0c4b4a…` | **success** — the revision of this record |
| 40098034415 | `f87ef70a…` | failure |
| 40095757610 | `86c1b819…` | failure |
| 40086303685 | `80ac0b86…` | failure |
| 40084145359 | `b0d0f40c…` | success |
| 40061588015 | `7cd6fe04…` | success |
| 40061298163 | `2a9662c6…` | success |
| 40060846773 | `6565b016…` | failure |

The gate was **red three times** on the way to the revision of this record.
A gate that was never red is not a gate; this one was.

---

## 8. The open gates

| # | The gate | Why it is open | What closes it |
|---|---|---|---|
| 1 | The device test plan of `docs/guides/DEVICE_TEST_PLAN.md` | no sender, no television, no second device | the hardware of §1 of the plan and the numbered script of every row |
| 2 | The performance budgets | a build server measures a build server | a running receiver with a real sender |
| 3 | The mask of the log on a rendered line | the test that `NFR-PRIV-002` names does not exist | a test that captures a rendered line (§6.2) |
| 4 | The rustdoc warning | the gate does not run rustdoc with `-D warnings` | `RUSTDOCFLAGS: "-D warnings"` in the workflow, and the fix of the link |
| 5 | The build of the container image | `docker: command not found` | a machine with Docker |
| 6 | The installation as a service | `systemctl: command not found` | a Debian-family machine |
| 7 | The comparison of the steps of the dashboard and the shells | the comparison needs a device on both sides | the device test plan |
| 8 | The acceptance records of M1, M2 and M3 | they do not exist | the milestones of those records |
| 9 | The release notes | `docs/releasing/` was empty at the revision of this record | the release procedure |
| 10 | The pointer of `project/project.yaml` | it still names `docs/acceptance/M0-ACCEPTANCE.md` | the owner of that file |

---

## 9. The checks that were not run — and why

| Check | The exact reason |
|---|---|
| The device test plan, every row | the environment has no sender, no television and no second device |
| The performance budgets | the budgets are properties of the running receiver with a real sender |
| `docker build -f deploy/docker/Dockerfile -t phairplayrust:latest .` | `docker: command not found` |
| `docker compose up -d phairplay` | `docker: command not found` |
| `docker run --net=host phairplayrust:latest --profile nas` | `docker: command not found` |
| `systemctl enable --now phairplayrust.service` | `systemctl: command not found` |
| `sudo ufw allow …` | there is no `ufw` in the sandbox |
| `iw phy phy0 info` | the sandbox has no Wi-Fi adapter |
| The download of the raw log of the CI run | `403 — Must have admin rights to Repository.` |
| A screen-reader pass | a screen reader needs the platform |
| The APK of the Android shell and the SwiftUI part of the macOS shell | no Android SDK licence and no macOS SDK here — the status is in `tv-android-shell.md` and `tv-macos-shell.md` |
| A coverage measurement | **no tool for it in the environment**. **This report contains no coverage percentage, because none was measured.** |

---

## 10. The decision

**ACCEPTED with a documented limitation.** The revision
`2d0c4b4a87629e7ab815a0d8ce6900a5ea2390c0` builds, the lint is clean, the
format is clean, **1224** tests pass with **0** failed and **0** ignored, the
structural gates and the licence gate are green, and the continuous
integration ran the four jobs on exactly this revision with four green
conclusions.

The reservations are named in §8 of the acceptance record. The two that
matter most:

1. **The interoperability is unconfirmed.** No sender of the matrix has been
   tested. The milestone may be *announced* only together with the device
   evidence of the device test plan.
2. **The mask of the log is proved on the helper, not on the rendered line.**
   The method of verification that `NFR-PRIV-002` names does not exist in
   the tree.

The owner has to re-run the gate on the merged tree: the working tree of the
moment of this acceptance was modified by another agent and did not compile.
