# CR-0002 — The interface as a remote control and the headless build

**Status:** implemented
**Date:** 2026-10-24
**Proposer role:** product and requirements role
**Reviewer roles:** Rust architect, Rust QA engineer, documentation role

## Problem

Two gaps separated the receiver from the device a household actually owns.

1. **The interface was a page for a mouse.** The dashboard and the native
   shells were operable with a keyboard, but they were not built for a
   remote control: the focus was thin, the switch of a protocol was a small
   target in the corner of a card, and the Android shell was a telephone
   application that never appeared in the list of a television.
2. **The receiver needed a screen.** The protocols were started by a click
   in a browser. On a Raspberry Pi or on a NAS there is no browser, so the
   receiver had to be started by hand — which means it was not running.

Neither gap is a defect of the engine. Both are gaps of the *product
surface*, and both were closed without touching the protocol engines.

## Requested Change

Build the dashboard and the native shells as television interfaces that a
remote control can operate, and give the receiver a headless mode in which
the program starts the protocols itself, reads its configuration from a file
and knows the profiles of the classes of devices.

## Reason

A remote control has an arrow pad, an *OK* button and — usually — no tab
key. An interface that assumes a tab key is an interface that a television
cannot reach: the user presses a key and nothing happens. The fix is the
pattern that the web platform already calls *roving tabindex* (one tab stop
for a group, the arrow keys move inside it) plus a drawn focus frame and a
target that is big enough to hit.

A receiver that waits for a click is not a receiver on a device without a
screen. The start belongs to the start of the process — and it belongs to
exactly one place, otherwise there are two truths about what runs.

## Impacted Requirement IDs

| Identifier | Change |
|---|---|
| `FR-096` | **New** — the group of the cards is one tab stop, the arrow keys move inside it and leave it at the edge |
| `FR-097` | **New** — the focus is visible and measured |
| `FR-098` | **New** — the card is the switch |
| `FR-099` | **New** — the Android shell is an application of the television |
| `FR-100` | **New** — the receiver starts the protocols itself |
| `FR-101` | **New** — the configuration of the receiver is read from a file |
| `FR-102` | **New** — the receiver knows the profiles of the classes of devices |
| `FR-103` | **New** — a control panel in the network is announced |
| `FR-079` | Unchanged — the dashboard still binds to the loopback interface; `FR-103` names what happens when the user asks for another address |
| `FR-084` | Extended — the accessibility of the dashboard now names the remote control operation explicitly |

## Impacted Architecture

- **The composition root** `apps/web/src/receiver.rs` is the one place that
  builds the engines, starts them and reports the outcome of every one of
  them. The rule "the web application speaks only through the supervisor,
  never directly with the engines" is written into
  [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) §3.4.
- **The command line** `apps/web/src/cli.rs`: the options `--config`,
  `--profile`, `--port`, `--bind`, `--no-dashboard`, `--version` and
  `--help`. Every option has a sentence in the help, because the help is the
  only documentation a device without a screen has.
- **The device profiles**: a new layer above the configuration model. The
  *names* of the profiles live in the core (`config::profiles::DeviceProfile`),
  the *values* live in the files under `config/profiles/`.
- **The dashboard**: a roving tabindex in the group of the cards, a drawn
  focus frame, the card as the switch, and a panel that can be switched off
  with `--no-dashboard` while the API stays available.
- **The native shells**: a deterministic focus graph in the Android shell
  (`TvFocusGraph`), the focus frame of the project in both shells, the
  `LEANBACK_LAUNCHER` category and the banner in the manifest of the Android
  shell.
- **The build contract** is not changed.

## Impacted Platforms

- **Android**: the shell becomes a television application; one application
  for the telephone and for the television.
- **macOS**: the shell reads the keys itself and draws its own focus frame.
- **Web**: the panel becomes a television interface and can run without its
  files.
- **Linux without a screen**: the receiver runs as a service or in a
  container. This is the platform that gains the most.

## Milestone Impact

- **M2** gains the remote control operation of the dashboard and the
  headless mode of the backend.
- **M3** gains the focus graph, the focus frame and the Leanback entry of the
  Android shell.
- **M4** gains the device profiles, the headless operation and the
  installation as a service and as a container.

No gate is weakened. The gates that need a television, a remote control or a
sender are written down as **NOT RUN** with the exact reason.

## Acceptance / Test Changes

- **New tests**: the keyboard check, the layout check and the contrast check
  of the dashboard in a real Chromium, driven with the keyboard only; the
  unit tests of the focus graph of the Android shell; the tests of the
  keyboard and of the focus of the macOS kit; the tests of the command line
  and of the composition root of the web backend.
- **New requirement identifiers**: `FR-096` … `FR-103`, each with a *How to
  Verify* and a *Pass Condition*.
- **The evidence** is filed in [`../audit/`](../audit/tv-web-dashboard.md),
  [`../audit/tv-android-shell.md`](../audit/tv-android-shell.md) and
  [`../audit/tv-macos-shell.md`](../audit/tv-macos-shell.md).
- **The acceptance records of M2 and M3** are still missing — they are
  carried by the milestones.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| The arrow keys make a group unreachable | medium | the edge of the group leads out of it to the next control of the page; the rule is proved by the keyboard check |
| A key press is undone by a reading of the API | high, without a guard | every local change carries a number, and a reading that is older than the last change does not overwrite it |
| The panel of a headless receiver is reachable from the network | medium, if the user asks for it | the default is the loopback interface, and a foreign address is announced with a warning before anything is started |
| A profile weakens the security model | low | the profile decides the device class and its budgets; the bind address and the pairing policy stay with the policy of the configuration |

## Decision

**Accepted and implemented** on 2026-10-24 by the product role, after the
review of the architect and of the QA engineer. The acceptance records of the
milestones M2, M3 and M4 and the device tests are still open and are carried
by the milestones.
