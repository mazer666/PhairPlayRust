# Documentation Policy

The documentation of PhairPlayRust is part of the product
([`../README.md`](../README.md),
[`../PROJECT_MANAGEMENT.md`](../PROJECT_MANAGEMENT.md) §7).

---

## 1. The same-change rule

A code or a configuration change is **incomplete** if it makes any document
stale. The update happens in the **same commit**:

| What changed | What is updated |
|---|---|
| The behaviour | [`../REQUIREMENTS.md`](../REQUIREMENTS.md) — through a change request when the change is material |
| The structure, a boundary, a dependency | [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — through an ADR when the change is material |
| An interface between two crates | [`design/BUILD-CONTRACT.md`](design/BUILD-CONTRACT.md) — through a change request |
| A default, a limit, a budget, a policy | [`../config/`](../config/README.md) **and** the typed model in `core/phairplay-core/src/config.rs` |
| The milestone or the state | [`../ROADMAP.md`](../ROADMAP.md) and [`../project/project.yaml`](../project/project.yaml) |
| A gate, a test, an acceptance criterion | [`../QUALITY.md`](../QUALITY.md) and [`acceptance/`](acceptance/ACCEPTANCE_TEMPLATE.md) |
| The entry point of a change | [`../WHERE_TO_CHANGE.md`](../WHERE_TO_CHANGE.md) |

A change that only touches the code and knowingly makes a document wrong is
not finished.

---

## 2. The single-source rule

**Every number and every rule of the documentation tree is written down
exactly once.** The other documents link to it.

| What | Where it lives |
|---|---|
| The numeric performance budgets and the resource limits | [`../config/performance-budgets.toml`](../config/performance-budgets.toml) |
| The user-visible default behaviour | [`../config/app-defaults.toml`](../config/app-defaults.toml) |
| The security and the privacy policy | [`../config/security-policy.toml`](../config/security-policy.toml) |
| The requirement identifiers and their verification | [`../REQUIREMENTS.md`](../REQUIREMENTS.md) |
| The state of the project | [`../project/project.yaml`](../project/project.yaml) |
| The interfaces between the crates | [`design/BUILD-CONTRACT.md`](design/BUILD-CONTRACT.md) |
| The historical snapshots | `../archive/` (read-only) |

A budget that appears in two documents is a budget that will drift. The
requirements therefore name the key of the budget and never repeat the value:

```text
Good:  "within the budget `latency.first_frame_ms` of config/performance-budgets.toml"
Bad:   "within <the number>"   ← a second place to change
```

The Rust constants in `core/phairplay-core/src/config.rs` are the **typed
mirror** of the TOML files, not a second documentation source. A value that
changes in a TOML file changes in the Rust default in the same change.

The check `bash scripts/check-docs-links.sh` compares the documented budgets
with the typed model and fails on a difference.

---

## 3. The house style

**Language.** The content of the repository is English. The prose is short
sentences, active voice, no marketing. A reader who is not a native speaker
must be able to follow it.

**Headings.** A document starts with the title, then a block of metadata
(version, status, the rule that governs the document), then the content.

**Structure.** Every document answers the question of its section in the
first paragraph. A section that only repeats another document links to it
instead of copying it.

**Tables.** A table carries a header row. A table is used when the
information is a comparison, a limit or a mapping — not to lay out a
paragraph.

**Examples.** A command that a reader can copy is given in a fenced block with
the language tag. Every command in the documentation is a command that works
in a clean checkout.

**Numbers.** A number in the prose always names its source.

**No placeholders.** "Coming soon", "TODO" and "Lorem ipsum" do not appear in
a document that is part of the set. A part that does not exist yet is named
as *planned* in the roadmap, with the milestone that delivers it.

---

## 4. The link rule

Every internal link of the documentation resolves to a file that exists.

```bash
bash scripts/check-docs-links.sh
```

The script is part of `make check`. A pull request with a broken link does
not pass the gate.

Rules:

- a link is relative to the file that carries it
- a link to a directory is allowed; a link to a file that will be created
  later is not
- an external link is allowed, but it is never the only way to reach a fact of
  the project
- an anchor inside a document is allowed; the script does not verify the
  anchor itself

---

## 5. The evidence rule

A statement about the state of the product names its evidence: a test name, a
command, the output of a command.

- "the mirror is stable" is a statement; "the mirror is stable — `cargo test
  -p phairplay-core` — 42 passed" is evidence
- a check that could not be executed is written as **NOT RUN** with the exact
  reason
- a measured value names the device, the input and the command

The rules of the evidence are in [`../QUALITY.md`](../QUALITY.md).

---

## 6. The lifecycle of a document

| State | Meaning |
|---|---|
| `planned` | The document is named in the roadmap and does not exist yet |
| `living` | The document is maintained with the code |
| `frozen` | The document records a decision and is not changed anymore |
| `archived` | The document is superseded and lives under `archive/` |

The ADR and the CR templates carry their own status. A superseded ADR stays
in place; the new one links to it.

---

## 7. Who owns what

| Area | Owner |
|---|---|
| The root documents, this directory, the guides | The documentation and developer experience role |
| The requirements and the exclusion list | The product and requirements role |
| The architecture and the build contract | The architect role |
| The acceptance records | The QA role |
| The ADRs | The architect role |
| The change requests | The product role |
| The configuration files | The architect role, with the product role for the defaults |

Two people may write in the same document. **One place owns the truth.**
