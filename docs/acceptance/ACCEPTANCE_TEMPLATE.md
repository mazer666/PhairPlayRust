# Acceptance — FEATURE/MILESTONE

**Requirement IDs:**
**Milestone:**
**Build/commit:**
**Tester/agent roles:**
**Date:**
**Reference device (hardware, OS, network):**

## 1. Functional

| Scenario | Expected | Result | Evidence |
|---|---|---|---|
| | | | |

Every row names the test or the command that produced the result. A row
without evidence is not a result.

## 2. Technical

- The crate boundaries are respected: yes / no
- The central configuration is respected: yes / no
- The dependency direction is unchanged: yes / no
- The tests are green: yes / no
- The lint is green (`cargo clippy --workspace --all-targets -- -D warnings`): yes / no
- The format check is green (`cargo fmt --all --check`): yes / no
- The documentation links resolve (`bash scripts/check-docs-links.sh`): yes / no
- The versioning or the migration is checked: yes / no / not applicable

## 3. Performance

| Metric | Budget key | Measured | Result |
|---|---|---|---|
| Time to the first picture | `latency.first_frame_ms` | | |
| Sustained frame rate | `throughput.minimum_fps` | | |
| Resident memory | `resources.max_memory_mb` | | |
| CPU usage | `resources.max_cpu_percent` | | |
| Audio and video synchronisation | `latency.audio_video_sync_ms` | | |

The budgets are owned by
[`config/performance-budgets.toml`](../../config/performance-budgets.toml). The
values are not repeated in this document.

Reference hardware, input and the exact command of every measurement:

## 4. Usability

- The number of steps of the task:
- The discoverability notes:
- The distinction of the states ("disabled", "advertising", "streaming", "error"):
- The clarity of an error message:
- The mobile / native behaviour:

## 5. Accessibility

- The keyboard operation:
- The screen reader (VoiceOver / TalkBack):
- The scaling, the high contrast, the reduced motion:

## 6. Security and privacy

- The new data that is stored:
- The new network access:
- The negative tests that were executed:
- The masking of the logs:
- The threat notes:

## 7. Interoperability

| Sender | Version | Result | Evidence |
|---|---|---|---|
| | | | |

- The formats that a reference implementation reads:
- The receiver without a component of the vendor of the sender:
- The deviations from the expectation of the sender:

## 8. The honest limits

- The exclusions of the requirements that apply here (`EXC-001` … `EXC-010`):
- The limitations that were found during the acceptance and that are not yet
  written down anywhere:
- The severity of every known issue (S0 to S4):

## 9. The checks that were not run

| Check | Reason | What would be needed |
|---|---|---|
| | | |

A check that did not run is **NOT RUN** with the exact reason. It is never
reported as a success.

## 10. Decision

accepted / rejected / accepted with a documented limitation

The decision names the person or the role that took it, and the date.
