# The self-test of the gate — 2026-10-24

> **Status:** binding. This document is the evidence that the gate of
> PhairPlayRust is a gate: a gate that has never refused anything is not a
> gate.

## 1. The command

```text
bash scripts/local-ci.sh --self-test
```

The self-test breaks every rule on purpose, proves that the gate goes red,
undoes the break and proves that the gate goes green again. The files it
touches are restored by a `trap`, so the working tree is unchanged when the
run ends — also when the run is interrupted.

The raw output is [`2026-10-24-gate-self-test.log`](2026-10-24-gate-self-test.log).

## 2. The output, word for word

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

## 3. What was broken, and how

| Rule | The violation | The file that was touched | The exit code that the rule returned |
|---|---|---|---|
| The lint of the workspace | `Some(1).unwrap()` in the code — the workspace denies `unwrap()` | `core/phairplay-core/src/identity.rs` | **101** (cargo) |
| The length of the source files | A file of 600 lines — the hard limit is 550 | `core/phairplay-core/src/self_test_long_file.rs` | **1** |
| The licence headers | `license = "MIT"` in a member manifest instead of `license.workspace = true` | `core/phairplay-core/Cargo.toml` | **1** |
| The links of the documentation | A link to a file that does not exist | `docs/audit/self-test-document.md` | **1** |

Every rule returned a non-zero exit code with the violation, and returned
**0** after the repair. The gate is therefore a gate.

## 4. The fifth rule: the licence audit

The licence audit of the dependencies has its own proof, because the rule
is not "a file is wrong" but "the tool has the wrong version".

```text
$ sed 's/CARGO_DENY_REQUIRED_VERSION="0.20.2"/CARGO_DENY_REQUIRED_VERSION="0.18.0"/' \
      scripts/audit-licenses.sh > scripts/audit-licenses-wrong-version.tmp.sh
$ bash scripts/audit-licenses-wrong-version.tmp.sh
NOT RUN: this machine carries cargo-deny 0.20.2, the project pins 0.18.0.
         The spellings of the GNU licences in deny.toml differ between the versions of the tool,
         so a run with another version would not read the configuration of the project.
         The licence configuration of the project was NOT checked in this run.
         Install the pinned version with:
           cargo install cargo-deny --version 0.18.0 --locked
$ echo $?
2
```

The exit code **2** means NOT RUN. A machine with another version of
cargo-deny does not report a false error and does not report a green
run — it reports that the proof did not happen.

With the pinned version, the audit is green:

```text
$ cargo deny --version
cargo-deny 0.20.2
$ bash scripts/audit-licenses.sh
==> cargo-deny 0.20.2 (the pinned version) reads deny.toml.
...
bans ok, licenses ok, sources ok

OK: cargo-deny found no violation of the licence policy of the project.
$ echo $?
0
```

## 5. The rule that the self-test does not cover

The job `audit` of the continuous integration runs `cargo audit` against
the advisory database of RustSec. The self-test does not break that rule:
a violation of the advisory database is a finding about a **dependency**,
and it is produced by adding a dependency to the lock file of the
repository — a change that the self-test must not make to the tree of a
contributor. The job is therefore verified by its own installation step:
`cargo install cargo-audit --locked` fails when the network is not there,
and the job then says NOT RUN with the reason instead of reporting a green
audit.
