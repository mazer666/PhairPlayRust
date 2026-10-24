# Central Configuration

PhairPlayRust keeps product defaults, limits and policy in one obvious place.

The source code uses typed settings models that correspond to these documented
values. The mapping is:

| Documented file | Typed model in the Rust core |
|---|---|
| `config/app-defaults.toml` | `DeviceConfig`, `ProtocolConfig` in `core/phairplay-core/src/config.rs`; `DeviceIdentity` in `core/phairplay-core/src/identity.rs` |
| `config/profiles/*.toml` | `AppConfig` in `core/phairplay-core/src/config.rs` — a profile is a complete configuration of one class of devices |
| `config/performance-budgets.toml` | `PerformanceConfig`, `LimitsConfig` in `core/phairplay-core/src/config.rs` |
| `config/security-policy.toml` | `AppConfig` in `core/phairplay-core/src/config.rs` plus the parser, callback and pairing guards of the engines |

## Who owns which budget

Every budget of `performance-budgets.toml` has exactly one owner. The owner
is the place a change has to be made — and the only place the value is
written down.

| Budget | Owner | Machine-checked? |
|---|---|---|
| `latency.first_frame_ms` | `PerformanceConfig::first_frame_latency_ms` | yes — compared with the typed model |
| `latency.idle_timeout_ms` | `PerformanceConfig::idle_timeout_ms` | yes |
| `throughput.minimum_fps` | `PerformanceConfig::minimum_fps` | yes |
| `resources.max_memory_mb` | `PerformanceConfig::max_memory_mb` | yes |
| `resources.max_cpu_percent` | `PerformanceConfig::max_cpu_percent` | yes |
| `queues.video_queue_frames` | `LimitsConfig::video_queue_frames` | yes |
| `queues.video_queue_bytes` | `LimitsConfig::video_queue_bytes` | yes |
| `queues.audio_queue_packets` | `LimitsConfig::audio_queue_packets` | yes |
| `queues.audio_queue_bytes` | `LimitsConfig::audio_queue_bytes` | yes |
| `limits.max_message_bytes` | `LimitsConfig::max_message_bytes` | yes |
| `limits.max_header_count` | `LimitsConfig::max_header_count` | yes |
| `limits.max_header_bytes` | `LimitsConfig::max_header_bytes` | yes |
| `limits.max_pairing_attempts` | `LimitsConfig::max_pairing_attempts` | yes |
| `limits.max_artwork_bytes` | `LimitsConfig::max_artwork_bytes` | yes |
| `limits.max_photo_bytes` | `LimitsConfig::max_photo_bytes` | yes |
| `limits.max_media_bytes` | `LimitsConfig::max_media_bytes` | yes |
| `latency.audio_video_sync_ms` | this file — a measurement target, not a default | yes — must exist and must not be copied |
| `throughput.measurement_window_seconds` | this file — a measurement target | yes |
| `jobs.max_concurrent_workers` | this file — a bound of the job executor | yes |
| `jobs.max_queued_jobs` | this file — a bound of the job executor | yes |
| `jobs.max_queue_wait_ms` | this file — a bound of the job executor | yes |
| `code_limits.target_max_lines_per_file` | this file — a rule of the code style | yes |
| `code_limits.hard_max_lines_per_file` | this file — a rule of the code style | yes |

"Mirrored" means that the value also exists in the typed Rust model and that
`scripts/check-docs-links.sh` compares the two. "Owned by this file" means
that the value has no typed counterpart — it is a measurement target, a job
bound or a rule of the code style. The script checks that such a value exists
and that it is not written down a second time.

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
- [`profiles/`](profiles/README.md) — the device profiles: one TOML file per
  class of devices.

## Device profiles

A **profile** is the configuration of one class of devices. The numbers of a
profile live in the profile itself and are **not** repeated here — the table
below links to them.

| Profile | Owner of the values | Purpose | How it is used |
|---|---|---|---|
| [`raspberry-pi`](profiles/raspberry-pi.toml) | the profile file — the device integration, reviewed by Product + Architect + Performance | a Raspberry Pi 3B/4/5 or an Orange Pi as a service: small memory budget, no Miracast without a Wi-Fi Direct radio | `phairplay-web --config config/profiles/raspberry-pi.toml` or `--profile raspberry-pi` |
| [`nas`](profiles/nas.toml) | the profile file — the device integration, reviewed by Product + Architect + Performance | a Synology, QNAP or TrueNAS box without a display: the protocol is the product, the dashboard is a control panel reached over SSH | `phairplay-web --config config/profiles/nas.toml` or `--profile nas` |
| [`docker`](profiles/docker.toml) | the profile file — the device integration, reviewed by Product + Architect + Performance | the receiver in a container: the network mode of the container decides whether a sender finds the receiver at all | `phairplay-web --config config/profiles/docker.toml` or `--profile docker` |
| [`desktop`](profiles/desktop.toml) | the profile file — the device integration, reviewed by Product + Architect + Performance | a Linux desktop with a screen: all four protocols, the product-wide budgets | `phairplay-web --config config/profiles/desktop.toml` or `--profile desktop` |
| [`generic`](profiles/generic.toml) | the profile file — the device integration, reviewed by Product + Architect + Performance | the fallback for a device nobody described: the product-wide behaviour, written down explicitly | `phairplay-web --config config/profiles/generic.toml` or `--profile generic` |

The rules of a profile — what it may decide, what it may not, and why every
number in it is justified — are in [`profiles/README.md`](profiles/README.md).

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
