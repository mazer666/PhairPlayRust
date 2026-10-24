# PhairPlayRust Quality & Acceptance Model

A PhairPlayRust feature is not "done" because the happy path worked once.

Every substantial feature is accepted across six dimensions. A dimension that
is not examined is a dimension that is not accepted.

---

## 1. Functional acceptance

Questions:

- Does the feature do what the requirement says?
- Are the preset and the optional behaviour correct?
- Are the edge cases defined?
- Are the errors understandable and actionable?
- Does the honest limit of the product still hold?

Evidence:

- the acceptance scenarios of the requirement
- the automated tests
- the manual exploratory notes where automation is unsuitable

The requirement itself carries the *How to Verify* and the *Pass Condition*
in [`REQUIREMENTS.md`](REQUIREMENTS.md). The acceptance record repeats the
evidence, never the requirement.

---

## 2. Technical acceptance

Questions:

- Does the implementation follow the crate boundaries?
- Are the state and the ownership rules respected?
- Are the limits read from the configuration?
- Is the failure isolated?
- Is the code understandable and inside the file-size limits?
- Is the functionality implemented with authentic logic rather than with a
  stub or a mock?

Evidence:

- the review of the architecture
- the tests, which run the real engines against realistic data
- the real run on the target platform or device — no mock bridge
- the static analysis and the lint
- the review of the dependency

---

## 3. Performance acceptance

Questions:

- Is the time to the first picture inside the budget?
- Does the frame rate hold over a session of the configured length?
- Is the memory bounded and does it stay bounded over a long session?
- Does the CPU usage stay inside the budget?
- Does the work in the background yield to the interactive work?

Evidence:

- a reproducible measurement
- a named reference device and a named input
- the result stored with the milestone evidence

**Never write "fast" without a measurement.** The numeric budgets live in
[`config/performance-budgets.toml`](config/performance-budgets.toml); this
document does not repeat them.

---

## 4. Usability acceptance

Questions:

- Can a new user find the common path without an instruction?
- Is the default quiet?
- Are the advanced options available without cluttering the default?
- Is the number of actions competitive?
- Is the exit and the recovery path obvious?
- Does the interface distinguish "stopped" from "disabled"?

Evidence:

- the walkthrough of the task with the step count
- the visual review
- the accessibility review
- the comparison with the reference products of
  [`docs/BEST_IN_CLASS.md`](docs/BEST_IN_CLASS.md)

---

## 5. Security and privacy acceptance

Questions:

- Does the feature create a new network access?
- Does it store new user data?
- Is the temporary data minimised?
- Is the untrusted input bounded?
- Can a malformed input crash the component or escape it?
- Is the sensitive action explicit?
- Does the log still mask the sender?

Evidence:

- the threat review
- the negative tests, which really run the attack
- the review of the permissions
- the inspection of the logs and of the caches

The policy lives in [`config/security-policy.toml`](config/security-policy.toml).

---

## 6. Interoperability and data integrity acceptance

Questions:

- Does a real sender of the protocol work with the receiver?
- Does the receiver work without a component of the vendor of the sender?
- Are the standards used correctly?
- Is the unsupported data refused instead of being silently mangled?

Evidence:

- the tests against real senders
- the cross-sender matrix
- the reference decoders for the format layer
- the results of the device test plan

---

# The performance budgets

The budgets are **not** part of this document. They are owned by
[`config/performance-budgets.toml`](config/performance-budgets.toml) and are
referenced from the requirements by their key:

| Dimension | Budget key |
|---|---|
| Time to the first picture | `latency.first_frame_ms` |
| Audio and video synchronisation | `latency.audio_video_sync_ms` |
| Deadline of a silent peer | `latency.idle_timeout_ms` |
| Sustained frame rate | `throughput.minimum_fps` |
| Resident memory | `resources.max_memory_mb` |
| CPU usage | `resources.max_cpu_percent` |
| Queue budgets | `queues.*` |
| Limits of the untrusted input | `limits.*` |
| Background work | `jobs.*` |
| File size | `code_limits.*` |

A budget is a target, not an excuse to hide a slower behaviour. A budget that
has to be raised goes through a change request, and the reason is written
down.

---

# The severity levels

### S0 — Cosmetic

No functional impact.

### S1 — Minor

A small inconvenience; a safe workaround exists.

### S2 — Significant

A core workflow is degraded or confusing.

### S3 — Major

A wrong output, an interoperability failure, a major performance regression.

### S4 — Critical

A crash, a memory exhaustion, a security bypass, a data loss, an
unrecoverable corruption.

A milestone or a release cannot ship with a known S4 issue.

---

# The acceptance package of a feature

Every feature has:

1. the requirement identifiers
2. the user scenarios
3. the functional acceptance tests
4. the technical checks
5. the performance expectations and the measurement
6. the usability expectations
7. the security and the privacy considerations
8. the interoperability matrix where it is relevant
9. the known limitations
10. the links to the evidence

The template is
[`docs/acceptance/ACCEPTANCE_TEMPLATE.md`](docs/acceptance/ACCEPTANCE_TEMPLATE.md).

---

# The rules of the evidence

1. **Every statement names its evidence.** A test name, a command, or the
   output of a command. "Implemented" without evidence is not a statement, it
   is a hope.
2. **A check that did not run is `NOT RUN` with the exact reason.** A missing
   tool, a missing device, a missing credential — the reason is written down.
   It is never reported as a success.
3. **A measurement names the device, the input and the command.**
4. **A limitation is documented.** A known limitation is a fact; a hidden
   limitation is a defect.
5. **A stub never satisfies an acceptance.** A function that returns a fixed
   value proves nothing about the behaviour of the product.
