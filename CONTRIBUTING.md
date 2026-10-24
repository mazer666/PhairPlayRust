# Contributing to PhairPlayRust

Thank you for contributing to PhairPlayRust!

PhairPlayRust is a receiver for the local network: AirPlay 2, Miracast,
Google Cast and DLNA/UPnP. It has no account, no telemetry, no advertising
and no cloud dependency, and it keeps every byte it receives on the device it
runs on.

---

## 1. The Developer Certificate of Origin (DCO)

PhairPlayRust uses the **Developer Certificate of Origin (DCO)** instead of a
Contributor License Agreement (CLA). Contributors keep their copyright.

Every commit carries a `Signed-off-by` line (`git commit -s`):

```text
Signed-off-by: Your Name <your.email@example.com>
```

By signing off you certify that you have the right to submit the contribution
under the open-source license of the project (**AGPL-3.0-or-later**).

---

## 2. Before you change anything

1. Read [`README.md`](README.md) and
   [`WHERE_TO_CHANGE.md`](WHERE_TO_CHANGE.md).
2. Check [`REQUIREMENTS.md`](REQUIREMENTS.md) for the product behaviour.
3. Check [`ARCHITECTURE.md`](ARCHITECTURE.md) for the module boundaries and
   the technical constraints.
4. Check [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md)
   before you touch an interface between two crates.
5. Check [`AGENTS.md`](AGENTS.md) for the coding standards, the naming
   conventions and the review matrix.

---

## 3. The rules that matter most

- **Every byte from the network is untrusted.** A length limit and a deadline
  on every read.
- **Every limit comes from the configuration.** Never a literal in an engine
  or in a shell.
- **Every queue is bounded twice**: by an item count and by a byte budget.
- **Every public function has a test.** Every hardening measure has a
  negative test that really runs the attack.
- **No `unwrap()`, no `expect()`, no `unsafe`.** A missing value is an error,
  not a panic.
- **No stub, no fixed return value, no mocked bridge** as an accepted
  functionality.
- **The documentation changes with the code.** A change that makes a document
  wrong is an unfinished change.
- **A number exists in exactly one place.** The other documents link to it.
- **No telemetry, no account, no cloud fallback, no advertisement.**
- **The honest limits of the product stay visible.** FairPlay DRM is not
  decryptable, the in-app audio of the Apple Music app is protected, and a
  Miracast session needs the WFD permission of the platform.

---

## 4. The checklist of a pull request

- [ ] The requirements and the architecture boundaries are respected; the
      dependency direction is unchanged.
- [ ] The defaults, the limits and the policy stay centralised under
      [`config/`](config/README.md) and the typed model in the core.
- [ ] The unit, the integration and the negative tests were added or updated.
- [ ] The documentation was updated in the same change if the behaviour, a
      limit, a default or a structure changed.
- [ ] `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D
      warnings` are green.
- [ ] `cargo test --workspace` is green.
- [ ] `bash scripts/check-docs-links.sh` is green.
- [ ] The commit is signed off (`git commit -s`).
- [ ] The known limitations of the change are named in the handoff.

---

## 5. How a change is reported

A pull request says what it changes, which requirement identifiers it touches,
which tests prove it, and which limitations it has. A check that could not be
executed is reported as **NOT RUN** with the exact reason — never as a
success. A partial implementation is reported as partial, with the exact list
of the open parts.

---

## 6. Reporting a problem

A bug report names the version, the platform, the sender that was used and
the steps that reproduce it. The build information of the running receiver is
in the settings of the application; it belongs in the report, because a bug
without a version cannot be reproduced.

A **security** problem is not a public issue. It is reported privately to the
maintainers of the repository.
