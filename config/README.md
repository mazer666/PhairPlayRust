# Central Configuration

PhairPlayRust keeps product defaults, limits and policy in one obvious place.

The source code uses typed settings models that correspond to these documented
values. The mapping is:

| Documented file | Typed model in the Rust core |
|---|---|
| `config/app-defaults.toml` | `DeviceConfig`, `ProtocolConfig` in `core/phairplay-core/src/config.rs`; `DeviceIdentity` in `core/phairplay-core/src/identity.rs` |
| `config/performance-budgets.toml` | `PerformanceConfig`, `LimitsConfig` in `core/phairplay-core/src/config.rs` |
| `config/security-policy.toml` | `AppConfig` in `core/phairplay-core/src/config.rs` plus the parser, callback and pairing guards of the engines |

## The single-source rule

**Every number and every rule in the documentation tree is written down
exactly once.**

- Numeric performance budgets and resource limits live in
  `config/performance-budgets.toml`.
- User-visible default behaviour lives in `config/app-defaults.toml`.
- Security and privacy policy lives in `config/security-policy.toml`.
- Requirement identifiers and their verification live in `REQUIREMENTS.md`.
- The machine-readable project state lives in `project/project.yaml`.

Every other document links to these files instead of repeating their content.
A budget that appears in two documents is a budget that will drift.

The Rust constants in `core/phairplay-core/src/config.rs` are the **typed
mirror** of the TOML files, not a second documentation source. A value that
changes in a TOML file changes in the Rust default in the same change; the
change is incomplete otherwise. See
[`docs/DOCUMENTATION_POLICY.md`](../docs/DOCUMENTATION_POLICY.md).

## Files

- [`app-defaults.toml`](app-defaults.toml) — user-visible default behaviour: the
  advertised identity, the per-protocol switches, the session rules, the
  dashboard bind address, the log policy.
- [`performance-budgets.toml`](performance-budgets.toml) — latency, throughput,
  resource, queue, input-limit and job budgets, plus the code-size limits.
- [`security-policy.toml`](security-policy.toml) — the threat model, the
  privacy promise, the parser rules, the XML rules, the callback guard, the
  pairing policy and the copy-protection policy.

## How a value is read at runtime

The engines never hard-code a limit. They read it from the typed
configuration model, which is filled from these files:

```rust
let config = AppConfig::from_path("config/app-defaults.toml")?;      // AppConfig
let limits = &config.limits;                                         // LimitsConfig
let budget = &config.performance;                                    // PerformanceConfig
```

`AppConfig::validate()` rejects a value that is syntactically valid but
semantically wrong — a zero queue budget, a zero latency budget, an unknown
protocol name — with `ConfigError::Invalid { field, reason }`. A missing key
is never an error: every field has a default.

## Secrets

Never store in repository config:

- pairing private keys
- PIN codes
- device certificates
- any credential of a sender

The pairing material belongs into the secure storage of the platform and is
referenced by path at most. The repository holds policy, never secrets.
