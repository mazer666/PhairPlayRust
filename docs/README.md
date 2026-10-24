# PhairPlayRust Documentation Index

This directory holds the detailed living documentation of PhairPlayRust. The
documents at the root of the repository are the entry points; the documents
here go deeper.

## The core documents (root)

- [`../README.md`](../README.md) — the entry point, the orientation, the
  quickstart
- [`../REQUIREMENTS.md`](../REQUIREMENTS.md) — the product requirements: every
  identifier, every *How to Verify*, every *Pass Condition*
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the layers, the crate graph, the
  data flow of every protocol, the threading model, the error and event flow
- [`../ROADMAP.md`](../ROADMAP.md) — the milestones, the gates, the state
- [`../QUALITY.md`](../QUALITY.md) — the six dimensions of the acceptance model,
  the severity levels, the rules of the evidence
- [`../AGENTS.md`](../AGENTS.md) — the roles, the universal rules, the review
  matrix, the definition of done
- [`../PROJECT_MANAGEMENT.md`](../PROJECT_MANAGEMENT.md) — the sources of
  truth, the change requests, the ADRs, the archive policy
- [`../WHERE_TO_CHANGE.md`](../WHERE_TO_CHANGE.md) — the navigation for a
  newcomer
- [`../GLOSSARY.md`](../GLOSSARY.md) — the terms of the protocols
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — the DCO, the checklist
- [`../CHANGELOG.md`](../CHANGELOG.md) — what changed
- [`../THIRD_PARTY.md`](../THIRD_PARTY.md) — the inventory of the third-party
  components with their licences
- [`../config/README.md`](../config/README.md) — the defaults, the budgets, the
  policy
- [`../project/project.yaml`](../project/project.yaml) — the machine-readable
  project state

## The detailed living documentation (`docs/`)

- [`BEST_IN_CLASS.md`](BEST_IN_CLASS.md) — the measurable quality grid: what is
  measured, how, and against which reference
- [`DOCUMENTATION_POLICY.md`](DOCUMENTATION_POLICY.md) — the same-change rule,
  the single-source rule, the house style
- [`GETTING-STARTED.md`](GETTING-STARTED.md) — the guide for a newcomer
- [`design/BUILD-CONTRACT.md`](design/BUILD-CONTRACT.md) — the binding
  interfaces between the crates

### Templates

- [`acceptance/ACCEPTANCE_TEMPLATE.md`](acceptance/ACCEPTANCE_TEMPLATE.md) — the
  template for the acceptance record of a milestone or a feature
- [`adr/ADR_TEMPLATE.md`](adr/ADR_TEMPLATE.md) — the template for an
  architecture decision record
- [`cr/CR_TEMPLATE.md`](cr/CR_TEMPLATE.md) — the template for a change request

### The guides

- [`guides/PROTOCOL_OVERVIEW.md`](guides/PROTOCOL_OVERVIEW.md) — what a sender
  actually does in each of the five protocols, and where the limits are
- [`guides/OTHER-DEVICES.md`](guides/OTHER-DEVICES.md) — on which devices the
  receiver runs and how it gets there: Raspberry Pi, container, NAS, VM,
  mini PC, desktop, WSL (German)
- [`guides/OTHER-DEVICES.en.md`](guides/OTHER-DEVICES.en.md) — the same guide
  in English; both editions are complete
- [`guides/CONFIGURATION.md`](guides/CONFIGURATION.md) — how the receiver is
  configured, and where every value lives
- [`guides/DEVICE_TEST_PLAN.md`](guides/DEVICE_TEST_PLAN.md) — the checks that
  need a real sender, a real television or a real device; every row is NOT
  RUN until somebody executes it
- [`guides/HARDENING_CHECKLIST.md`](guides/HARDENING_CHECKLIST.md) — the
  checklist for a new parser, a new listener or a new callback
- [`guides/TROUBLESHOOTING.md`](guides/TROUBLESHOOTING.md) — the symptom, the
  cause, the check

### The measured evidence — `docs/audit/`

The audit records carry the measured proof of a shell, of an engine or of a
document. A number in one of them always names the command that produced it.

- [`audit/tv-web-dashboard.md`](audit/tv-web-dashboard.md) — the dashboard as a
  remote control interface: the keyboard, the focus order, the measured
  contrast and the layout
- [`audit/tv-android-shell.md`](audit/tv-android-shell.md) — the Android shell:
  the compile, the tests, the lint, the order of the focus and the Leanback
  entry of the manifest
- [`audit/tv-macos-shell.md`](audit/tv-macos-shell.md) — the macOS shell: the
  test suite of the kit, the keyboard, the focus frame and the catalogue of
  the texts
- [`audit/2026-10-24-baseline.md`](audit/2026-10-24-baseline.md) — the baseline
  of the first implementation
- [`audit/other-devices-guide.md`](audit/other-devices-guide.md) — the evidence
  behind the guide for other devices: the build, the command line, a real
  start, the dry run of the installer

### The areas that other gates fill

These directories are reserved for the material of the later gates. They stay
empty until the gate that owns them delivers its first file; nothing in the
documentation links to a file that does not exist yet.

- `docs/project/` — the risk register
- `docs/features/` — the granular feature specifications
- `docs/releasing/` — the release process and the build from source
- `docs/reports/` — the reports of the rounds of work
- `docs/decisions/` — the decisions of the project owner

The directories that already carry a first file: `docs/adr/` and `docs/cr/`
(the templates, the records follow), `docs/acceptance/` (the template and the
record of M0), `docs/audit/` (the audit records), `docs/assets/` (the logo and
the icons) and `docs/site/` (the landing page).

## How this set is maintained

The rules are in [`DOCUMENTATION_POLICY.md`](DOCUMENTATION_POLICY.md). The
short version:

1. a number and a rule exist in **exactly one** place; the other documents
   link to it
2. a change that makes a document wrong updates it **in the same change**
3. every internal link resolves — `bash scripts/check-docs-links.sh`
4. the state of the project is machine-readable in
   [`../project/project.yaml`](../project/project.yaml)
