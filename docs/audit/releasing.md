# The release notes of PhairPlayRust — the report of the release round

> **Status:** living. This record holds the evidence behind the release
> process and the release notes in [`../releasing/`](../releasing/README.md):
> what was written, which number was quoted and where it comes from, what
> the gate reported, and what was missing.
>
> **The rule of this round:** a number that is quoted names its source. A
> number that could not be reproduced from its source is not quoted at all
> — it is written down here as an open point for the owner.
>
> **The branch:** `feat/other-devices`. **No commit, no push, no tag** — the
> handover is the working tree and this report.

---

## 1. What was written

| Path | What it is | Lines |
|---|---|---|
| [`docs/releasing/RELEASING.md`](../releasing/RELEASING.md) | the **process**: the two kinds of release, the six steps, the version, the gate, the commit and the tag, the publication, what does not belong into a release, the checklist | 392 |
| [`docs/releasing/RELEASE-NOTES-v0.2.0.md`](../releasing/RELEASE-NOTES-v0.2.0.md) | the **notes of v0.2.0**: what it contains, what it does not, the evidence, the limits, the upgrade, the milestones | 473 |
| [`docs/releasing/README.md`](../releasing/README.md) | the **index** of the directory, the two kinds of release, the rules, the owner of every number | 92 |
| `docs/audit/releasing.md` | this report | — |

The four files are the only ones of this round. The rules of the house are in
[`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md).

### 1.1 The round had to correct itself — twice

**The acceptance records of the milestones M3 and M5 arrived while this round
was running.** They are the work of the QA role, they are written against the
same revision as the notes (`2d0c4b4`), and they changed the facts that the
notes had to carry:

| The record | What it says | What the notes had to change |
|---|---|---|
| [`M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) | **NOT ACCEPTED at the revision under test** — the Kotlin of the Android shell and the kit of the macOS shell do not compile. Four errors, one cause: the commit `86c1b81` (M5) added a protocol kind and a limits field to the bridge, regenerated the bindings, and the shells were not updated. | the head block of the notes, §1, §2.5, the evidence table, §4, the upgrade table, §9 and a new **§6.4** — the defect that this release brings with it |
| [`M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) | **ACCEPTED with a documented limitation** — the discovery works, the picture does not flow (`EXC-011`). 112 tests of the crate pass. The limitations L-1 to L-6 are named with their owner. | the head block, the evidence table, §4 (the SSDP conflict with its severity and its measured mechanism, L-2 and L-3 as rows) and the milestone table |

The files were written in the order in which the evidence arrived: the notes
were drafted against the state of the tree, and every statement of them was
re-checked against the two records before the handover. **A release note
that is written before the evidence exists is a draft; it is not a note.**

---

## 2. The numbers that were quoted, and where they come from

Every number of the release notes was read out of a file of the repository.
The table names the file and the place in it.

| The number | Where it is quoted | The file that carries it | How it was checked |
|---|---|---|---|
| 1224 tests passed, 0 failed | the notes §3 | `project/project.yaml` → `verification.tests_passed`; the run is quoted in [`docs-truthfulness.md`](docs-truthfulness.md) §4.3 | read out of both files |
| 13 members of the workspace | the notes §3 | [`../../Cargo.toml`](../../Cargo.toml) → `[workspace] members` | counted in the manifest |
| 5 protocol engines, 4 format crates | the notes §3 | `project/project.yaml` → `architecture.engines` / `architecture.formats` | read out of the file |
| 35 keyboard checks, 30 layout checks, 38 measured colour pairs | the notes §3 | [`M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §10; the raw output in [`tv-web-dashboard.md`](tv-web-dashboard.md) | read out of both files |
| 39 tests of the Android shell, 101 tests of the macOS kit — **at the commit of M3** | the notes §3 | [`M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1 and §10 | read out of the record |
| 2 Kotlin errors, 2 Swift errors — **at the revision of this release** | the notes §6.4 | [`M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1a | read out of the record |
| 112 tests of the Smart View engine, 0 failed | the notes §3 | [`M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §10; the raw output in [`smartview-engine-test.log`](smartview-engine-test.log) | read out of the record, and the five `test result` lines of the log were added up |
| the severity S1 of the conflict of the SSDP port | the notes §4 | [`M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §8, L-1 | read out of the record |
| the number of the tests of the baseline (1054) and of the baseline itself (961, 13 failed) | the process §6.2 (the message of the tag `v0.1.0`) | the tag `v0.1.0` of the repository, and [`2026-10-24-baseline.md`](2026-10-24-baseline.md) §3 | read out of the tag and of the file |
| the five check scripts, the exit codes 0/1/2 of the gate | the process §5 | [`../../scripts/local-ci.sh`](../../scripts/local-ci.sh), the header | read out of the script |
| the four jobs of the continuous integration | the process §3.2, the notes §3 | [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml) | read out of the workflow |
| `FR-096` to `FR-099`, `EXC-001` to `EXC-011` | the notes §2.5, §5 | [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) | read out of the file |
| the commit `2d0c4b4` | the notes, the status block | `git log --oneline` and the tag `v0.2.0` | `git for-each-ref refs/tags` |

### 2.1 The numbers that could not be reproduced

Two counters of the machine-readable state do not agree with the files they
name. The release notes quote the **files**, not the counters.

| The counter | The state | The source |
|---|---|---|
| `project/project.yaml` → `verification.shell_checks.smartview_engine_tests: 103` | the log that the file names carries five `test result` lines — 75 unit tests, 7 + 5 + 12 end-to-end tests, 13 doctests — and the acceptance record of M5 says **112 tests**. Neither is 103. | [`smartview-engine-test.log`](smartview-engine-test.log), [`M5-ACCEPTANCE.md`](../acceptance/M5-ACCEPTANCE.md) §10 |
| `project/project.yaml` → `verification.shell_checks.dashboard_layout_checks: 31` | the amendment of the dashboard audit and the record of M3 say **30** after the fifth card joined the panel | [`tv-web-dashboard.md`](tv-web-dashboard.md) (the amendment), [`M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §10 |

Both counters are open points for the owner (§5).

---

## 3. The checks

This round changed no source file — only documents. The checks that cover
documents were run; no build was needed and none was made.

### 3.1 The link gate — the run of the day

The run with the three release documents in the tree:

```text
$ bash scripts/check-docs-links.sh
==> Checking the internal links of the documentation...
==> Checking the documented budgets against the typed model...

Markdown files scanned : 62
Internal links checked : 604
Broken links           : 0
Budgets compared       : 16
Budgets owned alone    : 7

OK: every internal link resolves, every mirrored budget matches the typed model, and every budget owned alone exists exactly once.
```

Exit code **0**. The run covers the three new files: 25 links in
`README.md`, 37 in `RELEASING.md`, 69 in `RELEASE-NOTES-v0.2.0.md`. **No
link is broken, and no budget of
[`../../config/performance-budgets.toml`](../../config/performance-budgets.toml)
is repeated in the three files** — the second part of the same check is the
guard against a number that drifts into a second place.

The run of the tree at the moment of the handover — with this report and
with the two acceptance records of the QA role in it — is:

```text
Markdown files scanned : 68
Internal links checked : 691
Broken links           : 0
Budgets compared       : 16
Budgets owned alone    : 7
```

Exit code **0**. **The tree is being worked on at the same time** — the two
acceptance records were written during this round, and the repair of the
conflict of the SSDP port is in the working tree (§5.10). The count of the
links therefore moves with every change; the owner repeats the run before
the commit. **The number of the broken links is zero in every run of this
round.**

### 3.2 The link gate — the negative case

A gate that has never refused anything is not a gate. The rule was broken on
purpose: a link to `docs/acceptance/M6-ACCEPTANCE.md` — a file that does not
exist — was appended to the notes, and the gate had to go red.

```text
$ bash scripts/check-docs-links.sh
!!! 1 broken link(s) found:
    ./docs/releasing/RELEASE-NOTES-v0.2.0.md: docs/acceptance/M6-ACCEPTANCE.md
…

Markdown files scanned : 68
Internal links checked : 691
Broken links           : 1
Budgets compared       : 16
Budgets owned alone    : 7

FAILED: the documentation is not consistent with the repository.
```

Exit code **1**. The broken link is the one that was broken on purpose — a
link to a file that does not exist. The rule of
[`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §4 is exact: *a
link to a file that will be created later is not* — and the gate refuses it.

The same rule met the round twice: before this report existed, the link from
the notes to it was refused as well, with the same exit code.

The break was undone, the file was restored, and the gate was green again
(§3.1).

### 3.3 The other four checks of the gate

They cover the source of the repository, and this round changed none of it.
They were run anyway, because a release is only as good as its gate:

```text
$ bash scripts/check-crate-boundaries.sh
Crates checked      : 13
Internal edges      : 15
Violations          : 0
OK: every dependency points the way the build contract demands.
EXIT 0

$ bash scripts/check-license-headers.sh
Member manifests checked : 13
Source files checked     : 251
Derived files checked    : 10
Violations               : 0
OK: every licence header of the repository is correct.
EXIT 0

$ bash scripts/check-file-size.sh
Files measured          : 369
Generated files skipped : 2
Above the soft limit    : 9
Above the hard limit    : 0
EXIT 0

$ bash scripts/audit-licenses.sh
bans ok, licenses ok, sources ok
OK: cargo-deny found no violation of the licence policy of the project.
EXIT 0
```

The nine files above the soft limit are the ones of the engines, of the
bridge and of the scripts; none of them is a file of this round, and the
hard limit is not broken.

### 3.4 What was not run

| What | Why | Where |
|---|---|---|
| the tests of the workspace, the lint, the format check, the build of the documentation | this round changed no source file; the state of the gate is the state of the commit `2d0c4b4` | the record of M0, [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md); the number of the tests of the revision is in [`docs-truthfulness.md`](docs-truthfulness.md) §4.3 |
| the Gradle build of the Android shell and the Swift build of the macOS kit | they **fail** at the revision of this release — that is the finding of the record of M3, and it is in the notes | [`../acceptance/M3-ACCEPTANCE.md`](../acceptance/M3-ACCEPTANCE.md) §1a |
| the continuous integration on the commit of the release | the runners are not reachable from the sandbox | [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml) |
| the device tests | no sender, no television, no device | [`../guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) |
| the performance budgets | they are budgets; a measurement needs a running receiver on a device | [`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) §3 |

---

## 4. What was missing

The notes were written anyway, and they name the gap instead of hiding it.

| What was missing | How the notes deal with it |
|---|---|
| the acceptance records of the milestones **M1, M2 and M4** — the paths `docs/acceptance/M1-ACCEPTANCE.md`, `M2-ACCEPTANCE.md` and `M4-ACCEPTANCE.md` do not exist | the notes name them as paths in §4 and in §8, and they are **not** linked — a link to a file that will be created later is refused by the gate (§3.2). The records of M0, M3 and M5 exist and are linked. |
| the device tests of the interoperability | the notes carry them in §4 as **NOT RUN**, with the reason and the plan that closes them |
| the measured performance budgets | the notes name the keys of the budget file instead of the values, and they say that a budget is not a measurement |
| the state of the continuous integration on the commit of the release | the notes say in §3 that it is not verifiable from the working tree, and that these notes do not claim it |
| `gh` on the machine of the sandbox — the tool of the publication step is not installed | the process names the command, names its prerequisite, and names the alternative in the web interface |
| the binaries of the shells — the APK is **NOT RUN**, the SwiftUI part of the macOS shell is **NOT RUN**, and the shells do not even build at this revision | the process keeps them out of a release; step 6 is the decision of the owner, and the notes carry the build break in §6.4 |

---

## 5. The open points for the owner

1. **Two documents of the tree now say something that is wrong.** The index
   [`../README.md`](../README.md) lists `docs/releasing/` among the
   directories that *"stay empty until the gate that owns them delivers its
   first file"*, and [`../../ROADMAP.md`](../../ROADMAP.md) carries the
   deliverable *"The release notes — `docs/releasing/` is empty"* as an open
   box of the milestone M4. Both are wrong since this round. The
   same-change rule of [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md)
   §1 asks for the correction **in the same change**; the file list of this
   round does not contain those two files, so the correction is the
   owner's.
2. **The native shells do not build at the revision of the release.** The
   notes carry it in §6.4; the repair is two branches per shell and belongs
   to the milestone that broke the build (the record of M3 names the
   conditions for its acceptance).
3. **The native shells are outside every continuous gate.** No Gradle task
   and no Swift test is part of the CI of the repository. The record of M3
   names it as the second condition for its acceptance; until it is closed, a
   break of a shell reaches the branch unnoticed.
4. **Two counters of the machine-readable state do not match their sources**
   (§2.1): the tests of the Smart View engine and the layout checks of the
   dashboard.
5. **A pointer in the requirements is stale.** `REQUIREMENTS.md` FR-104
   points the composition root at `apps/web/src/receiver.rs`; since ADR-0006
   it is `platform/phairplay-ffi/src/composition.rs`. The record of M5 names
   it as the limitation L-5. The file is not part of this round.
6. **The tag `v0.2.0` already exists**, annotated, and it points at the
   commit `2d0c4b4`. `VERSION` and the `[workspace.package]` table of
   `Cargo.toml` still carry `0.1.0`, and the changelog still carries the
   section as *Unreleased*. **Step 2 of the process is therefore still
   open** — the tree and the tag do not name the same version yet.
7. **The conflict of the SSDP port** — the DLNA renderer and the Smart View
   responder want the same port, and the renderer holds it first. The notes
   name it as an open point of the release with its severity (S1) and its
   measured mechanism; the decision belongs to the engines.
8. **The decision about the binaries** — whether a binary is published at
   all, and with which licence notice and signature. Step 6 of the process
   keeps it a decision of the owner.
9. **The working tree is not the tagged commit** — `ROADMAP.md` is modified
   in the working tree (the card of the fifth protocol in the dashboard),
   and that change is not part of the tag `v0.2.0`.
10. **The repair of the conflict of the SSDP port is in flight.** The file
    that the limitation L-1 names — `engines/dlna/src/ssdp.rs` — is modified
   in the working tree: the DLNA responder now sets `SO_REUSEADDR` before it
   binds the socket. **The notes of this release describe the tagged
   revision `2d0c4b4`, where the fifth protocol does not start in the
   default configuration.** If the repair is committed, then §4 of the notes
   and the limitation L-1 of the record of M5 have to be updated **in the
   same change**, and the tag has to name the revision that carries the
   repair.

---

## 6. The handover

No commit, no push, no tag, no new release. The four files of this round
are in the working tree, and this report is their evidence.
