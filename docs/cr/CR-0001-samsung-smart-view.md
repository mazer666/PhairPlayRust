# CR-0001 — Samsung Smart View as the fifth protocol

**Status:** implemented
**Date:** 2026-10-24
**Proposer role:** product and requirements role
**Reviewer roles:** Rust architect, Rust QA engineer, documentation role

## Problem

The receiver speaks four protocols today. Samsung Smart View — the screen mirroring
of the Galaxy phones and tablets — is not one of them, and a household with a
Galaxy device cannot send its screen to the receiver.

The protocol has a second, harder problem: its **message format is
proprietary and not publicly documented**. The Smart View SDK realises it as
closed `.jar` libraries. A receiver can therefore be a *Smart View target* —
the device appears in the list of the phone and the phone connects to it —
without ever delivering a picture.

Without a written decision, the project would either claim a protocol it
cannot implement or stay silent about a limit that a user runs into.

## Requested Change

Add Samsung Smart View as the fifth protocol of the product, with the
discovery and the handshake implemented, with the message format of the
channel named as an explicit exclusion, and with the whole change visible in
the requirements, the architecture, the roadmap and the glossary.

## Reason

The discovery and the handshake of the MultiScreen Framework are documented by
Samsung ("Device Discovery, Authentication, and Pairing"): the search of the
service target, the description document that the `LOCATION` header points
at, and the handshake in which the sender announces its address and its
name. Those three steps are implementable and testable.

The content of the channel is not. Inventing a message format that nobody
documents would produce a receiver that looks complete and is not — exactly
the kind of claim this project does not make. The honest way to ship the
protocol is to implement the part that is documented and to name the rest as
an exclusion (`EXC-011`), so that the state of the product is the same in the
code, in the dashboard and in the documentation.

## Impacted Requirement IDs

| Identifier | Change |
|---|---|
| `FR-006` | The wording of "four protocols" becomes "five protocols" |
| `FR-009` | The service type `_samsungmsf._tcp` joins the advertised types |
| `FR-104` … `FR-107` | **New** — the discovery, the API and the handshake, the single-session rule and the honest limit |
| `EXC-011` | **New** — the message format of the Smart View channel |
| `EXC-001` | Unchanged — Smart View is not a DRM path and is not affected by the exclusion of FairPlay |

## Impacted Architecture

- **New crate** `engines/smartview`: the SSDP answer of the MultiScreen
  search, the description document, the HTTP API below `/api/v2/`, the
  handshake of the sender and the WebSocket upgrade of the channel. The
  crate depends on the core only — no format crate, no web framework.
- **The core**: a new variant of `ProtocolKind`, the display name, the
  service type in `config::service_types` and the port of the API in
  `config::ports`. No new module.
- **The bridge**: the new kind in the data transfer objects. No new exported
  function.
- **The build contract** is not changed: the engine follows the rules that
  already exist (one peer at a time, every read has a deadline, every input
  is length-limited).
- **The composition root** (`platform/phairplay-ffi/src/composition.rs`) registers the engines
  of the receiver. The fifth engine is registered there like every other
  one, so the receiver starts it with the others. The **card of the fifth
  protocol in the dashboard is still missing** — the panel of the browser
  lists five protocols. That is the open point of the milestone M5 in
  [`../../ROADMAP.md`](../../ROADMAP.md).

## Impacted Platforms

- The **dashboard** and the **native shells** are not changed by this
  request. The fifth protocol appears in the status of the protocols like
  every other one; the card of the dashboard is a separate change.
- The **headless build** is not changed: the discovery of the Smart View
  receiver happens on the SSDP multicast of the network, exactly like the
  DLNA renderer.

## Milestone Impact

The milestone **M5 — Smart View** is new. It carries the engine, the
tests and the evidence. Its gate that needs a Galaxy device is **NOT RUN**
and stays open; the gate that the message format of the channel is not
implemented is closed by `EXC-011`.

The milestones M0 to M4 are not changed by this request.

## Acceptance / Test Changes

- **New tests**: the unit tests of the codec of the SSDP answer, of the
  description document, of the API and of the handshake; the integration
  tests over a real socket (`discovery`), the control plane over a real
  connection (`control_plane`) and the negative tests of the reader and of
  the channel (`hardening`). The output of the run is in
  [`../audit/smartview-engine-test.log`](../audit/smartview-engine-test.log).
- **New requirement identifiers**: `FR-104` … `FR-107`, each with a *How to
  Verify* and a *Pass Condition*.
- **No acceptance criterion is weakened.** The criterion "a Galaxy device
  finds the receiver and the picture flows" is **not** weakened — it is
  split: the discovery is proved by a test, the picture is named as a limit
  of the product and not as a criterion.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| The receiver claims a picture that does not flow | high, if the status of the protocol is read as "streaming" | the status reports the session of the sender, and the documentation names the limit in every place that names the protocol |
| The protocol of Samsung changes and the answer becomes wrong | medium | the answer is behind a pure codec with its own tests, so a change is a change of one function and its test |
| The port of the API collides with another service of the device | low | the port is owned by `config::ports`, and the profile of the device can switch the protocol off |
| The exclusion is read as "the feature is broken" | medium | the wording of `EXC-011` says what is implemented and what is not, in the same sentence |

## Decision

**Accepted and implemented** on 2026-10-24 by the product role, after the
review of the architect and of the QA engineer. The engine is delivered; the
acceptance record of the milestone M5 and the device test with a Galaxy
device are still open and are carried by the milestone.
