# The release process of PhairPlayRust

> **What this document is:** the description of the **release process** of
> PhairPlayRust — what has to be true before a release, what is written
> down, and what is published. It is the procedure, not the report: the state
> of one particular release lives in the notes of that release, in
> [`docs/releasing/`](README.md).
>
> **Status:** living.
>
> **The rule that governs this document:** a release may only claim what an
> acceptance record proves. A claim that no record proves is written down as
> **NOT RUN** with the exact reason — the rule of
> [`../acceptance/ACCEPTANCE_TEMPLATE.md`](../acceptance/ACCEPTANCE_TEMPLATE.md)
> and of [`../../QUALITY.md`](../../QUALITY.md).

---

## 1. What a release is in this project

A release of PhairPlayRust is a **named state of the tree** that a user can
name, that the project can point at with evidence, and that carries a list
of what it contains and a list of what it does not contain.

The criteria of a release are the ones of
[`../../PROJECT_MANAGEMENT.md`](../../PROJECT_MANAGEMENT.md) §12: the scope
of the milestone complete, the acceptance criteria green, no unresolved
critical security issue, no unresolved data-loss issue, the documentation
current, the licences and the notices current, the release notes
reproducible, the known limitations documented, and the honest limits of the
product visible to the user.

**A release that hides a limitation is a defect with a release date.**

There are two kinds of release in this project:

| Kind | What it means | When it is used |
|---|---|---|
| **A pre-release** | The code of one or more milestones is delivered and the gate of the workspace is green, but the evidence that only a device can produce is missing. The notes say so, in the first paragraph. | The device tests of the interoperability have not been run. The acceptance records of the milestones are missing. A performance budget is unmeasured. |
| **A release** | Every milestone that the release carries has a green acceptance record, the device tests have been run, and the budgets are measured. | The milestone is `done` in [`../../ROADMAP.md`](../../ROADMAP.md). |

A pre-release is not a failure of the process. It is the honest form of a
release whose device evidence is missing — the same evidence that the
acceptance record of M0 names as **NOT RUN**
([`../acceptance/M0-ACCEPTANCE.md`](../acceptance/M0-ACCEPTANCE.md) §3 and
§7).

---

## 2. The steps

| # | The step | What it produces | The document that owns it |
|---|---|---|---|
| 1 | **The state is proven** | the acceptance records and a green run of the continuous integration | [`../../ROADMAP.md`](../../ROADMAP.md), [`../acceptance/`](../acceptance/ACCEPTANCE_TEMPLATE.md) |
| 2 | **The version is set** | `VERSION`, `Cargo.toml`, `CHANGELOG.md` | this document, §3 |
| 3 | **The gate runs** | a green `make check` and a green `scripts/local-ci.sh` | [`../../Makefile`](../../Makefile), [`../../scripts/local-ci.sh`](../../scripts/local-ci.sh) |
| 4 | **The commit and the tag** | the commit and the annotated tag `v<major>.<minor>.<patch>` | this document, §5 |
| 5 | **The release is published** | the release page on GitHub with the notes of this directory | this document, §6 |
| 6 | **What does not belong into a release** | the decision about the binaries — a decision of the owner, not an automation | this document, §7 |

The steps run in this order. A step that is skipped is written down in the
notes of the release; it is never silently dropped.

---

## 3. Step 1 — the state is proven

A release is only made from a state that somebody has already measured.
Before the version is set, three things have to be true.

### 3.1 The acceptance records are green

Every milestone that the release carries has a record in
[`../acceptance/`](../acceptance/ACCEPTANCE_TEMPLATE.md), and every record
is green or states its reservations. The record names the commands that were
run — a statement about the state of the product that does not name its
command is not evidence.

**If a record is missing,** the release is a **pre-release**. The notes name
the missing records — the record of this milestone — and the missing
acceptance record of a milestone is a fact of the release, not a detail of
the process.

**A record that says NOT ACCEPTED is a red gate.** The milestone it covers
may not be announced in the release; the notes name the record, the reason
and what the repair is. A release that announces a milestone whose record
says NOT ACCEPTED contradicts the evidence it publishes.

**A build failure is a red gate, too.** If the code of the release does not
compile — a shell, an application, a script — the release may still be made
as a **pre-release**, and the notes say which part does not build, where the
error is and who repairs it. What it may not do is hide the failure behind
a milestone that is marked as delivered.

### 3.2 The continuous integration is green

The workflow [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
runs four jobs on the GitHub runners: the *Quality Gate* (the format, the
lint, the tests, the documentation), the *Licenses* (cargo-deny with the
pinned version), the *Security Audit* (cargo-audit against the advisory
database of RustSec) and the *Structure* (the crate boundaries, the length
of the source files, the licence headers, the links of the documentation).

All four jobs have to be green **on the commit of the release**. A green run
on an earlier commit is the evidence for that commit and not for this one.

### 3.3 The device test plan is executed — or it is named as open

The plan is [`../guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md).
Every row of it is **NOT RUN** until somebody carries it out on a real
sender, a real television or a real device.

- The rows are executed → the results belong in the acceptance record of the
  milestone, and the release may be a release.
- The rows are open → the release is a **pre-release**, and the notes name
  the plan and say that no row of it has been executed.

### 3.4 The state of the project is written down

[`../../project/project.yaml`](../../project/project.yaml) carries the
machine-readable state: the status of the product, the current milestone,
the verification block with its numbers, and the source of every one of
them. It is written in the same change as the code that moved the state.

---

## 4. Step 2 — the version is set

The version of the workspace lives in exactly three places. A release sets
all three in one change; a tree in which they differ is a tree that nobody
can name.

| Place | What it carries | The edit of the release |
|---|---|---|
| [`../../VERSION`](../../VERSION) | the version of the workspace, one line | `0.1.0` → `0.2.0` |
| [`../../Cargo.toml`](../../Cargo.toml) | `[workspace.package] version` | `0.1.0` → `0.2.0` — every crate of the workspace inherits it |
| [`../../CHANGELOG.md`](../../CHANGELOG.md) | the section of the release | the heading `## [Unreleased]` becomes `## [0.2.0] — 2026-10-24` |

The section of the changelog also gets its link at the bottom of the file:

```text
[Unreleased]: https://github.com/mazer666/PhairPlayRust/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mazer666/PhairPlayRust/releases/tag/v0.2.0
```

The text of the changelog section is written **before** the release, not
after it: the changelog of a release describes what the release contains,
and it is the summary that the release notes expand. The notes live in
[`docs/releasing/`](README.md) and are the detailed form.

The version follows [semantic versioning](https://semver.org/):

| Change | The version |
|---|---|
| a fix that changes no interface | `0.2.1` |
| a new capability, a new protocol, a new interface | `0.3.0` |
| an interface that breaks for a user | `1.0.0` |

---

## 5. Step 3 — the gate runs

The gate is the same gate that the continuous integration runs, and it has
to be run on the tree of the release.

### 5.1 The quick form

```bash
make check
```

`make check` runs the format check, the lint, the tests of the workspace and
the check of the links of the documentation. The targets are in
[`../../Makefile`](../../Makefile).

### 5.2 The full form

```bash
bash scripts/local-ci.sh
```

The script runs the structural checks first, the licences, then the
compiler, and the tests last — the order of the cost. Its exit code is the
verdict:

| Exit code | Meaning |
|---|---|
| `0` | the gate is green |
| `1` | a check found a defect — the gate is red |
| `2` | a check could not be run (**NOT RUN**) — the reason is named, and the gate does **not** claim to be green |

A **NOT RUN** is never reported as a green gate. That is the contract of
[`../../scripts/local-ci.sh`](../../scripts/local-ci.sh) and of the
acceptance template.

### 5.3 The five check scripts

The five scripts below are the structural gate. Each of them is a command of
its own, and each of them may be run on its own:

```bash
bash scripts/check-crate-boundaries.sh   # the dependency direction of the crates
bash scripts/check-file-size.sh          # the length of the hand-written sources
bash scripts/check-license-headers.sh    # the licence header of every source file
bash scripts/check-docs-links.sh         # every internal link resolves, every budget matches
bash scripts/audit-licenses.sh           # the licences of the dependencies (cargo-deny, pinned)
```

**The gate has to be a gate.** A rule that has never refused anything is not
a rule. The self-test proves that each of them refuses its violation:

```bash
bash scripts/local-ci.sh --self-test
```

The self-test breaks every rule on purpose, proves that the gate goes red,
undoes the break and proves that the gate goes green again. The output of
the run of the day is in
[`../audit/2026-10-24-gate-self-test.md`](../audit/2026-10-24-gate-self-test.md).

### 5.4 The three checks of the dashboard

The dashboard of the receiver has its own three checks, because a browser is
not a build server. They are not part of `make check`; they are part of the
release when the dashboard is part of the release:

```bash
node apps/web/tools/check-keyboard.mjs
node apps/web/tools/check-layout.mjs
node apps/web/tools/check-contrast.mjs
```

The measured results are in
[`../audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md).

---

## 6. Step 4 — the commit and the tag

### 6.1 The commit

The release is one commit that carries the version, the changelog and the
notes of the release. Nothing else: the tree of the release is the tree
that the gate ran on.

### 6.2 The tag

The tag is annotated and signed. Its name is `v<major>.<minor>.<patch>`.

```bash
git tag -a v0.2.0 -m "PhairPlayRust 0.2.0 — <what the release contains>

<the evidence: what was measured, with its source>

<what the release does not contain: the device tests, the budgets, the limits>

Licensed under AGPL-3.0-or-later."
```

**The message of the tag says two things: what the release contains and
what it does not.** A tag message that only says what it contains is an
advertisement, and the project does not write advertisements.

The message of the tag `v0.1.0` is the model. It names the evidence — the
workspace compiles, the lint is clean, the number of tests, the continuous
integration — and it names the reservation:

```text
PhairPlayRust 0.1.0 — the first baseline with the four receiver engines

The workspace compiles, the lint is clean under -D warnings, 1054 tests
pass, the continuous integration is green on the GitHub runners, and the
structural gates and the licence gate are green.

The device tests are outstanding. Every row of the acceptance record that
needs hardware is marked NOT RUN and names where it will be measured.

Licensed under AGPL-3.0-or-later.
```

The first line of that message says *with the four receiver engines*, while
the changelog of v0.1.0 says that no protocol was implemented in that
release — the engines were stubs. A tag is not edited after the fact. That
difference is the reason why the version and the changelog are set **before**
the tag, and why the notes of a release are written before it too.

**A tag is not moved in silence.** A tag that names a wrong state is
corrected by the owner, with a note in the release, and the old tag stays
where it is — the history of the project is read by people who were not
there.

---

## 7. Step 5 — the publication on GitHub

The release is published on the release page of the repository, with the
notes of this directory as the body of the release.

```bash
gh release create v0.2.0 \
  --title "PhairPlayRust 0.2.0" \
  --notes-file docs/releasing/RELEASE-NOTES-v0.2.0.md \
  --prerelease
```

The command needs a machine with `gh` and with an account that may publish
a release. The sandbox of the project has no `gh`, so this step is the step
of the owner; the equivalent in the web interface of the repository is the
form *Releases → Draft a new release*.

**`--prerelease` is not a decoration.** The flag marks the release as a
pre-release on the page of the project, and a pre-release is a release whose
device evidence is missing. A release whose notes say that the device tests
have not been run and that carries no pre-release flag is a release that
contradicts itself.

The release page carries:

| What | Where it comes from |
|---|---|
| the title | the version and the name of the release |
| the body | the notes of this directory — [`RELEASE-NOTES-v0.2.0.md`](RELEASE-NOTES-v0.2.0.md) |
| the tag | the tag of step 4 — the release attaches to the tag, it does not create a new one |

---

## 8. Step 6 — what does not belong into a release

**A binary that was not built and checked does not belong into a release.**
The gate of the project runs the source; a binary is a build artefact of a
platform, of a toolchain and of a licence, and a binary that nobody
checked is a promise that nobody made.

The publication of a binary is therefore **a decision of the owner**, and it
is a separate step with its own checklist. It is not an automation of this
process, and no script of this repository attaches a file to a release.

Before a binary is attached, the owner decides — and the decision is
written down:

| Question | The answer belongs in |
|---|---|
| Which platform, which architecture, which toolchain? | the decision of the owner |
| Was the binary built from the tag of the release, with the pinned dependencies of [`../../Cargo.lock`](../../Cargo.lock)? | the decision of the owner |
| Which licences and which notices travel with the binary? | [`../../THIRD_PARTY.md`](../../THIRD_PARTY.md), the licence inventory |
| Is the binary signed, and is the signature verifiable by the user? | the decision of the owner |
| Was the binary itself tested — the APK on a device, the SwiftUI part on macOS? | the acceptance record of the milestone |

**The state of the binaries of this project today is written down in
[`../../project/project.yaml`](../../project/project.yaml)**: the Android
shell compiles and its tests are green, but the APK is **NOT RUN** — the
NDK of the version that the Android Gradle plugin wants is not installed on
the machine of the audit. The SwiftUI part of the macOS shell is **NOT
RUN** — the machine of the audit has no macOS SDK. A binary that is not
tested is not published.

---

## 9. The checklist before the publication

| # | The question | The source of the answer |
|---|---|---|
| 1 | Is every acceptance record that the release carries green, or is the missing one named in the notes? | [`../acceptance/`](../acceptance/ACCEPTANCE_TEMPLATE.md) |
| 2 | Are the four jobs of the continuous integration green on the commit of the release? | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) |
| 3 | Do `VERSION`, `Cargo.toml` and the changelog carry the same version? | step 2 |
| 4 | Is `make check` green on the tree of the release? | [`../../Makefile`](../../Makefile) |
| 5 | Is `bash scripts/local-ci.sh` green, without a single **NOT RUN** that matters? | [`../../scripts/local-ci.sh`](../../scripts/local-ci.sh) |
| 6 | Does the gate refuse its violations? | `bash scripts/local-ci.sh --self-test` |
| 7 | Does the message of the tag name what the release contains **and** what it does not? | step 4 |
| 8 | Do the notes name every number with its source? | [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §5 |
| 9 | Do the notes say what is **NOT RUN** and why? | the acceptance template, §9 |
| 10 | Is the release marked as a pre-release, if a device test is open? | step 5 |
| 11 | Is every internal link of the notes resolvable? | `bash scripts/check-docs-links.sh` |
| 12 | Is the decision about the binaries written down, or is it explicitly deferred to the owner? | step 6 |

---

## 10. What this process does not prove

The process above proves that the **code** of the release is what the gate
measured. It does not prove that the receiver works in a living room, and
it does not claim to.

| What | Where it is measured | The state of the day |
|---|---|---|
| The interoperability of the five protocols | [`../guides/DEVICE_TEST_PLAN.md`](../guides/DEVICE_TEST_PLAN.md) | **NOT RUN** — it needs a sender, a television and a device |
| The performance budgets of the running receiver | the keys of [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml), measured in the acceptance record of the milestone | **NOT RUN** — a budget is a target that a measurement has to prove |
| The reading of the panel on a television in a bright room | [`../audit/tv-web-dashboard.md`](../audit/tv-web-dashboard.md) §8 | **NOT RUN** — the contrast is measured with the formula of WCAG; the room is not |
| The APK of the Android shell | [`../audit/tv-android-shell.md`](../audit/tv-android-shell.md) §9 | **NOT RUN** — no NDK of the wanted version on the machine of the audit |
| The SwiftUI part of the macOS shell | [`../audit/tv-macos-shell.md`](../audit/tv-macos-shell.md) §7 | **NOT RUN** — no macOS SDK on the machine of the audit |

A release that hides one of these rows is a defect with a release date.
