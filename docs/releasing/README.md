# The release directory of PhairPlayRust

> **What this document is:** the index of the directory that holds the
> **process** of a release and the **notes** of every release of the project.
>
> **Status:** living.
>
> **The rule that governs this directory:** a release may only claim what an
> acceptance record proves. A claim that no record proves is written down as
> **NOT RUN** with the exact reason — the rule of
> [`../acceptance/ACCEPTANCE_TEMPLATE.md`](../acceptance/ACCEPTANCE_TEMPLATE.md)
> and of [`../../QUALITY.md`](../../QUALITY.md).

---

## What this directory holds

| File | What it is | The state |
|---|---|---|
| [`RELEASING.md`](RELEASING.md) | the process: what has to be true before a release, how the version is set, which gate runs, how the commit and the tag are made, how the release is published, and what does **not** belong into a release | living |
| [`RELEASE-NOTES-v0.3.0-alpha.1.md`](RELEASE-NOTES-v0.3.0-alpha.1.md) | the notes of the release v0.3.0-alpha.1 — what it contains, what it does not, and the source of every number | written for an **alpha**: the code of the milestones is delivered and the gate is green, the device tests have not been run, and the release ships the headless receiver for the server platforms |
| [`RELEASE-NOTES-v0.2.0.md`](RELEASE-NOTES-v0.2.0.md) | the notes of the release v0.2.0 — what it contains, what it does not, and the source of every number | written for a **pre-release**: the tag `v0.2.0` exists in the repository, the publication on the release page is the step of the owner ([`RELEASING.md`](RELEASING.md) §7) |

The index of the whole documentation tree is in [`../README.md`](../README.md).

---

## The two kinds of release

| Kind | What it means |
|---|---|
| **A pre-release** | the code of one or more milestones is delivered and the gate is green, but the evidence that only a device can produce is missing — the device tests have not been run, an acceptance record is missing, a performance budget is unmeasured. The notes say so in the first paragraph. |
| **A release** | every milestone that the release carries has a green acceptance record, the device tests have been run, and the budgets are measured. |

The criteria of a release are the ones of
[`../../PROJECT_MANAGEMENT.md`](../../PROJECT_MANAGEMENT.md) §12. The
process that turns the criteria into steps is in
[`RELEASING.md`](RELEASING.md).

**v0.2.0 is a pre-release.** The device tests have not been run, the
acceptance records of the milestones M1, M2 and M4 are missing, and the
acceptance record of M3 says **NOT ACCEPTED at the revision of the release**
— the native shells do not compile there. The record of M5 is *accepted with
a documented limitation*: the discovery of the receiver works, the picture
does not flow. The record of M0 is the oldest one that exists, and it names
its own reservations.

---

## The rules that a release follows

| The rule | Where it is written down |
|---|---|
| The same-change rule — a change that makes a document wrong updates it in the same change | [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §1 |
| The single-source rule — every number exists in exactly one place | [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §2 |
| The evidence rule — a statement about the state of the product names its evidence | [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md) §5, [`../../QUALITY.md`](../../QUALITY.md) |
| The criteria of a release | [`../../PROJECT_MANAGEMENT.md`](../../PROJECT_MANAGEMENT.md) §12 |
| The state of the milestones | [`../../ROADMAP.md`](../../ROADMAP.md) |
| The state of the project, machine-readable | [`../../project/project.yaml`](../../project/project.yaml) |
| What changed, in the order a human reads it | [`../../CHANGELOG.md`](../../CHANGELOG.md) |

---

## Where the numbers of a release live

A release note repeats no number that has an owner elsewhere. The owner of
each number is named here, so that a reader of the notes knows where to look
for the evidence — and so that a writer of the next release knows where to
put a number that is new.

| The number | Its owner |
|---|---|
| The version of the workspace | [`../../VERSION`](../../VERSION) and the `[workspace.package]` table of [`../../Cargo.toml`](../../Cargo.toml) |
| The budgets and the limits | [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml) |
| The counters of the tests and of the checks | [`../../project/project.yaml`](../../project/project.yaml) → `verification` |
| The measured values of a shell, of an engine or of a document | [`../audit/`](../audit/) — one file per measurement, with the command and the raw output |
| The acceptance of a milestone | [`../acceptance/`](../acceptance/ACCEPTANCE_TEMPLATE.md) — one record per milestone |
| The requirement identifiers and the exclusion list | [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) |
| The licences of the third-party components | [`../../THIRD_PARTY.md`](../../THIRD_PARTY.md) |

---

## The gate of a release

The gate of the project is the gate of a release. It is run with
`make check` or with `bash scripts/local-ci.sh`, and it comprises the format
check, the lint of the workspace, the tests, the crate boundaries, the
length of the source files, the licence headers, the links of the
documentation and the licence audit of the dependencies.

The proof that the gate refuses its violations is
`bash scripts/local-ci.sh --self-test`; the output of the run of the day is
in [`../audit/2026-10-24-gate-self-test.md`](../audit/2026-10-24-gate-self-test.md).
