# PhairPlayRust Project Management

PhairPlayRust is managed as an engineering product, not as an accumulation of
features.

---

## 1. The sources of truth

| Topic | Source of truth |
|---|---|
| The product behaviour | [`REQUIREMENTS.md`](REQUIREMENTS.md) and the build contract |
| The architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) and the ADRs in `docs/adr/` |
| The current milestone | [`ROADMAP.md`](ROADMAP.md) and [`project/project.yaml`](project/project.yaml) |
| The quality and the acceptance | [`QUALITY.md`](QUALITY.md) and the records in `docs/acceptance/` |
| The agent process | [`AGENTS.md`](AGENTS.md) |
| The defaults, the budgets, the policy | [`config/`](config/README.md) |
| The build interfaces between the crates | [`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) |
| The historical baseline | `archive/` |

When two documents contradict each other, the conflict is fixed immediately.
Two living versions of a rule are not allowed to exist.

---

## 2. The work hierarchy

```text
Product vision
  ↓
Requirements
  ↓
Milestones
  ↓
Sub-milestones
  ↓
Acceptance criteria
  ↓
Implementation tasks
  ↓
Tests + evidence
  ↓
Release
```

A task must be traceable upwards to a requirement. A task without a
requirement is a wish; it needs a change request before it becomes work.

---

## 3. The milestone states

- `planned`
- `in_progress`
- `blocked`
- `acceptance`
- `done`

**"Done" requires evidence, not confidence.** A milestone is `done` when the
gates of the milestone in [`ROADMAP.md`](ROADMAP.md) are green and the
acceptance record names the command that produced every statement.

---

## 4. The states of a requirement

- `specified`
- `implementing`
- `testing`
- `accepted`
- `deferred`
- `superseded`

A deferred requirement says **why**, which change request deferred it, and to
which milestone it was moved. A requirement that is silently forgotten is a
defect in the project management, not a detail.

The state of the requirements is not tracked per requirement in the
requirements document itself; it is tracked per milestone in the roadmap and
in the machine-readable project state.

---

## 5. The change requests

A material product change requires a `CR-XXXX` in `docs/cr/`. The template is
[`docs/cr/CR_TEMPLATE.md`](docs/cr/CR_TEMPLATE.md).

A change request contains:

- the problem
- the requested change
- the reason
- the impacted requirement identifiers
- the impacted architecture
- the impacted platforms
- the impact on the milestone
- the changes of the acceptance and of the tests
- the risks
- the decision

Agents may **propose** a change request. They may not enact the product change
first and write the document afterwards.

A change request is needed when:

- a requirement is added, removed or weakened
- a default, a limit or a budget is changed
- the scope of a milestone changes
- a protocol leaves or enters the product
- an honest limit of the product is removed

A change request is **not** needed for:

- a bug fix that restores the documented behaviour
- a test that proves an existing requirement
- a documentation fix of a typo
- a refactoring that does not change a behaviour

---

## 6. The architecture decisions

A material architecture decision uses an ADR in `docs/adr/`. The template is
[`docs/adr/ADR_TEMPLATE.md`](docs/adr/ADR_TEMPLATE.md).

**ADRs are never deleted.** When a decision changes:

- the old ADR becomes `superseded`
- the new ADR links to the old ADR
- the historical reasoning stays readable

The build contract
[`docs/design/BUILD-CONTRACT.md`](docs/design/BUILD-CONTRACT.md) is a
binding document with the same discipline: a rule that blocks a correct
implementation is a wrong rule, and it is changed there first — through a
change request, never in passing.

---

## 7. The freshness of the documentation

Every meaningful change asks:

- Are the requirements stale?
- Is the architecture stale?
- Is the roadmap or the project state stale?
- Are the acceptance criteria stale?
- Are the configuration files stale?
- Is `WHERE_TO_CHANGE.md` stale?
- Is the build contract stale?

If yes, the documentation is updated **in the same change**.

A change that only touches the code and knowingly makes a document wrong is
incomplete. The rule is written down in
[`docs/DOCUMENTATION_POLICY.md`](docs/DOCUMENTATION_POLICY.md).

---

## 8. The archive policy

`archive/` contains the immutable historical baselines and the material that
was superseded.

A file in the archive is not a source of truth and is never edited.

The creation of a baseline:

1. update the current documents
2. run the validation
3. assign the baseline version
4. copy the required documents into `archive/baselines/<version>/`
5. write the manifest
6. tag the baseline in Git when the repository is live

---

## 9. The risk register

The significant risks are maintained in `docs/project/RISKS.md`.

A risk has:

- an identifier
- a description
- a probability
- an impact
- a mitigation
- an owning role
- a state

The risks that are known today are dominated by the threat model: a hostile
device in the local network, a sender that sends a flood, a parser that
misreads a length field. The mitigations are the limits of the
configuration, the negative tests and the bounded queues.

---

## 10. The discipline of the decisions

A decision is better when it is:

- explicit
- documented
- reversible where possible
- testable
- measurable

Speculative features are not designed in detail, but the agreed extension
points stay intact: a new protocol is a new engine crate, a new codec is a
new format crate, a new platform is a new shell.

---

## 11. The best-in-class reviews

At the acceptance of a milestone, the benchmark role compares the relevant
workflows with the reference products. The framework is
[`docs/BEST_IN_CLASS.md`](docs/BEST_IN_CLASS.md).

The review records:

- the reference product, the version and the date
- the task
- the steps
- the speed
- the clarity
- the failure behaviour
- the advantage and the disadvantage of PhairPlayRust
- the follow-up

A feature-count comparison is not a review. A measurement is.

---

## 12. The release readiness

A release requires:

- the scope of the milestone complete
- the acceptance criteria green
- no unresolved critical security issue
- no unresolved data-loss issue
- the documentation current
- the licenses and the notices current
- the release notes reproducible
- the known limitations documented
- the upgrade and the migration behaviour tested where it is relevant
- the honest limits of the product visible to the user

A release that hides a limitation is a defect with a release date.
