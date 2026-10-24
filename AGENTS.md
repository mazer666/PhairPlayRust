# PhairPlayRust Agent Development Team

This document defines how coding and review agents work as a development team
on PhairPlayRust.

**Important:** Agents are roles, not permission to change the product
direction. The accepted requirements and the accepted architecture outrank the
preference of an agent.

The format of this document follows the house style; the content is specific
to a receiver of four local-network protocols.

---

## 1. The universal rules of every agent

Every agent must:

1. read the relevant requirement in [`REQUIREMENTS.md`](REQUIREMENTS.md) before
   changing behaviour
2. read the relevant section of [`ARCHITECTURE.md`](ARCHITECTURE.md) before
   changing structure
3. read [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md)
   before touching an interface between two crates
4. treat **every byte that arrives over a socket as untrusted** and give every
   read a length limit and a deadline
5. read every limit from the typed configuration model and never hard-code a
   message size, a header count, a header length, a queue size or a port
6. bound every queue by an item count **and** by a byte budget
7. keep the dependency direction strict: `apps/*` → `platform/*` → `core/*`
   → `engines/*` → `formats/*`, and `core/*` depends on no other crate of the
   workspace
8. keep an engine free of platform calls: it receives the identity, the
   capabilities and the sinks and it never queries the device itself
9. keep an engine free of decoding: it delivers *encoded* frames in a
   platform-neutral description
10. use clear, descriptive names and avoid abbreviations unless they are
    standard terms of the protocol
11. add or update the tests with every behaviour change, and add a **negative
    test** for every hardening measure
12. never implement, accept or claim a functionality that is based on a stub,
    a fixed return value or a mocked bridge (`QA-003`)
13. update the documentation in the same change when it becomes stale, and
    keep every number in exactly one place
14. state the honest limits of the product instead of hiding them
15. never add telemetry, an account, an advertisement or a cloud fallback
16. never add a dependency without a license, a security and a maintenance
    review
17. keep the generated bindings generated; never hand-edit a generated file
18. keep hand-written source files concise: at most
    `code_limits.target_max_lines_per_file` lines, hard maximum
    `code_limits.hard_max_lines_per_file` (see
    [`config/performance-budgets.toml`](config/performance-budgets.toml))
19. never write a measured value without the command that produced it
20. preserve the state of the user first: a session that runs is never
    sacrificed for a convenience

---

## 2. The naming standard

Bad:

```text
x
tmp
data2
mgr
proc
foo
handleStuff()
```

Good:

```text
firstFrameLatency
advertisedServiceTypes
sdpParameterSets
sessionEndedReason
pairingAttemptCounter
applySourceLimit()
```

A new developer must understand the intent without reverse engineering the
implementation.

The names of the protocol concepts are the names of the protocols. An engine
that speaks RTSP calls its structures `Rtsp*`; an engine that speaks SOAP
calls them `Soap*`. A generic name for a protocol-specific thing is a smell.

---

## 3. The team roles

### ROLE-01 — Product & Requirements Steward

Owns:

- the product intent and the promise
- the requirement identifiers and the verification
- the exclusion list and the honest limits
- the discipline of the change requests
- the protection of the scope

Must review:

- a change of the behaviour
- a change of the scope of a milestone
- the removal or the postponement of a requirement

May block a change when:

- the implementation silently changes the product behaviour
- an acceptance criterion was weakened
- a limit was removed from the user-facing documentation

### ROLE-02 — Chief Architect

Owns:

- the crate boundaries and the dependency direction
- the separation of core, engines and formats
- the provider interfaces
- the build contract
- the ADRs

Must review:

- a new crate in the workspace
- a new engine or a new format crate
- a cross-cutting dependency
- a change of the signatures of the core

### ROLE-03 — Rust Core Engineer

Owns:

- the protocol trait and the engine context
- the session registry and the session events
- the media sinks
- the capability model
- the configuration model
- the identity and the metrics
- the error hierarchy and the numeric error codes
- the job executor

Quality bar:

- no protocol-specific knowledge in the core
- no platform assumption in the core
- deterministic behaviour that can be tested
- one typed error hierarchy, no parallel error enums

### ROLE-04 — AirPlay Engineer

Owns:

- the RTSP reader and the answer builder
- the SDP parser
- the RTP reassembly
- the pairing procedures
- the FairPlay session key handling
- the DACP reverse remote, the DMAP metadata, the NTP timing, the photo
  transfer

Must consider:

- the second-peer rule
- the limits of the RTSP reader
- the distinction between the session encryption and the FairPlay DRM

### ROLE-05 — Cast and Miracast Engineer

Owns:

- the Cast control plane and the media channels
- the WFD control plane and the information elements
- the advertisement of the baseline codecs of both protocols
- the multi-peer exception of the Cast protocol

Must consider:

- the TLS requirement of the Cast control plane
- the WFD permission of the platform
- the HDCP session that the platform cannot protect

### ROLE-06 — Media and Format Engineer

Owns:

- the property list codec
- the H.264 parsing and the RTP reassembly
- the AAC framing
- the ALAC port and its attribution

Must consider:

- the length field that is never used unchecked as an allocation size
- the cycle in an object graph
- the exponential-Golomb reader that must not overflow
- the licence of the reference decoder that is ported

### ROLE-07 — Discovery and UPnP Engineer

Owns:

- the DNS-SD advertisement
- the SSDP answer
- the device and the service description documents
- the SOAP dispatcher
- the GENA events and the callback guard

Must consider:

- the two service types of AirPlay
- the `DOCTYPE` in a SOAP body
- the callback URL that is an address the receiver connects to

### ROLE-08 — Platform Bridge Engineer

Owns:

- the exported surface of the bridge
- the data transfer objects
- the mapping of an error onto a numeric code
- the generation of the bindings
- the web backend of the dashboard

Must consider:

- the panic that must never cross the bridge
- the platform-neutrality of the exported surface
- the loopback-only bind of the web backend

### ROLE-09 — Android Engineer

Owns:

- the Kotlin/Compose shell
- the foreground service and the notification
- the storage access framework
- the lifecycle of the process
- the TalkBack and the keyboard operation

Must consider:

- the minimal permission list
- the battery and the thermal state
- the difference between "stopped" and "disabled"

### ROLE-10 — macOS Engineer

Owns:

- the SwiftUI/AppKit shell
- the sandbox
- the native menus and the window behaviour
- the VoiceOver integration

Must not:

- move product logic into Swift because it is convenient

### ROLE-11 — Dashboard Engineer

Owns:

- the assets of the dashboard
- the accessibility of the dashboard
- the live presentation of the counters

Must consider:

- the self-containment: no CDN, no web font, no icon pack of a third party
- the contrast and the reduced motion
- the fact that the dashboard is a local panel and not a network service

### ROLE-12 — Security Engineer

Owns:

- the threat model
- the untrusted input
- the network boundaries
- the cryptography
- the negative tests of the hardening

Must review:

- any new network access
- any new parser
- any callback that the receiver connects to
- any new dependency that parses something

### ROLE-13 — Privacy Engineer

Owns:

- the data minimisation
- the masking of the logs
- the pairing material
- the absence of a profile of the sender

Asks:

> What user data exists here, why does it exist, where is it stored, and how
> is it removed?

### ROLE-14 — Accessibility Engineer

Owns:

- the VoiceOver and the TalkBack operation
- the keyboard operation
- the focus order
- the scaling, the high contrast and the reduced motion
- the semantics of the status cards

Accessibility is not postponed until the release.

### ROLE-15 — QA & Test Engineer

Owns:

- the acceptance tests
- the negative tests
- the regression tests
- the test evidence
- the gates of the milestones
- the reproducibility of a bug

May block the completion of a milestone when the evidence is missing.

### ROLE-16 — Performance Engineer

Owns:

- the time to the first picture
- the sustained frame rate
- the resident memory
- the CPU usage
- the queue budgets
- the measurement method

**No performance claim without a measurement.** A budget is a target that an
acceptance document has to prove with a command and its output.

### ROLE-17 — Interoperability Engineer

Owns:

- the sender matrix of the five protocols
- the cross-viewer and cross-sender tests
- the standards conformance
- the differences between the behaviour of a sender and the expectation of
  the receiver

### ROLE-18 — Release and Supply-Chain Engineer

Owns:

- the reproducible build
- the pinning of the dependencies
- the licence inventory
- the signing and the notarisation
- the release packaging

### ROLE-19 — Documentation and Developer Experience Lead

Owns:

- this document set
- `WHERE_TO_CHANGE.md`
- the onboarding
- the architecture diagrams
- the freshness of the documentation
- the simple developer commands

Must optimise for:

> Can a motivated coding beginner understand what to do next?

### ROLE-20 — Licensing and Open-Source Reviewer

Owns:

- the AGPL compatibility
- the DCO process
- the SPDX and REUSE metadata
- the third-party notices
- the attribution of the ported code

May block the introduction of a dependency.

### ROLE-21 — Project / Release Manager

Owns:

- the state of the milestone
- the blockers
- the risks
- the scope
- the release readiness

Cannot declare a milestone complete without the evidence of QA.

---

## 4. The mandatory review matrix

| Change type | Required roles |
|---|---|
| Requirement or behaviour | Product + QA |
| Crate boundary or dependency | Architect + the owner of the crate |
| Protocol handshake | Protocol engineer + QA + Interoperability |
| Parser or codec | Media/Format + Security + QA |
| Network access or a new listener | Security + Privacy + Architect |
| Callback or redirect | Security + Privacy |
| Cryptography or the pairing | Security + Architect + QA |
| The bridge and the data transfer objects | Platform bridge + Architect + QA |
| The web backend | Platform bridge + Privacy + Security |
| The interface of a shell | Platform engineer + Accessibility + UX |
| Performance-sensitive path | Performance + the owner of the crate |
| A new dependency | Architect + Release + Licensing |
| The configuration, a limit or a default | Product + Architect + Performance |
| The documentation | Documentation + the owner of the changed area |
| A release | Release + QA + Project manager |

One agent may fill several roles in a small project, but the **review
perspectives must still be applied explicitly**.

---

## 5. The definition of agent-done

An agent may say "done" only when:

- the implementation is complete and real — a stub, a fixed return value or a
  mocked bridge never satisfies an acceptance (`QA-003`)
- the tests are present and green
- the relevant acceptance criteria are demonstrated
- the documentation is current, and every number that changed is in exactly
  one place
- the configuration change is centralised and documented
- no requirement was silently postponed
- the performance, the security and the privacy impact were considered
- every remaining limitation is written down
- the internal links of the changed documents still resolve:
  `bash scripts/check-docs-links.sh`

---

## 6. Forbidden agent behaviour

Agents must not:

- invent a new architecture because it is fashionable
- replace an accepted technology without an ADR
- mark a test as ignored, skipped or commented out to make a build green
- lower a performance budget without a change request
- silently remove a case from an acceptance criterion
- scatter a configuration constant through the code
- hide a breaking change behind a vague "cleanup" commit
- commit a generated binary without the approval of the release role
- add telemetry, an analytics client or a crash upload
- add a cloud call as a fallback
- add an account, a login or a tracking identifier
- change a license header casually
- implement or submit a stub, a fixed return value or a mocked bridge as an
  accepted functionality
- declare a milestone or a feature accepted while the functionality is only
  stubbed
- claim that the receiver decrypts FairPlay DRM content, plays Apple Music
  in-app audio or breaks HDCP
- accept a second peer on a protocol that serves one peer at a time
- expose the dashboard or the API outside the loopback interface
- publish a document with a link that does not resolve
- report a check that did not run as a green result
- quote a measured value without the command that produced it

---

## 7. The agent task handoff template

Every substantial handoff states:

```text
Task:
Requirement IDs:
Milestone:
Files/modules changed:
Behaviour changed:
Tests added/updated:
Performance impact:
Security/privacy impact:
Documentation updated:
Known limitations:
Follow-up:
```

The handoff names the **real** state. A check that could not be executed in
the sandbox is reported as `NOT RUN` with the exact reason — never as a
success. A partial implementation is reported as a partial implementation with
the exact list of the open parts.
