# How the receiver is configured

A guide for the developer and for the user who wants to change one value.

The **source of truth** for every value is the file in
[`../../config/`](../../config/README.md) that owns it. This guide only says
*which* file owns *which* value and *how* a value reaches the code.

---

## 1. The three files

| File | Owns |
|---|---|
| [`../../config/app-defaults.toml`](../../config/app-defaults.toml) | the advertised identity, the per-protocol switches, the session rules, the dashboard bind address, the log policy |
| [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml) | the latency, the throughput, the resource, the queue, the input-limit and the job budgets, the code-size limits |
| [`../../config/security-policy.toml`](../../config/security-policy.toml) | the threat model, the privacy promise, the parser rules, the XML rules, the callback guard, the pairing policy, the copy-protection policy |

The typed mirror of all three lives in
`core/phairplay-core/src/config.rs` (`AppConfig`, `DeviceConfig`,
`ProtocolConfig`, `LimitsConfig`, `PerformanceConfig`).

---

## 2. How a value reaches the code

```rust
// 1. read the file
let config = AppConfig::from_path("config/app-defaults.toml")?;

// 2. validate it — a wrong value is an error, a missing key is not
config.validate()?;

// 3. read a limit
let max_message_bytes = config.protocol(ProtocolKind::AirPlay).max_message_bytes;
let idle_timeout = config.performance.idle_timeout_ms;
```

`AppConfig::validate()` rejects a value that is syntactically valid but
semantically wrong — a zero queue budget, a zero latency budget, an unknown
protocol name — with `ConfigError::Invalid { field, reason }`. The message
names the dotted path of the offending key, so a user can find it.

Because the model is derived with `deny_unknown_fields`, a **typo in a key is
an error** and not a silently ignored line.

---

## 3. The rules of a change

1. **A value is written down once.** The TOML file owns it; the other
   documents link to it.
2. **A change touches the TOML file and the typed model in the same change.**
3. **An engine never hard-codes a limit.** It reads it from the model.
4. **A budget that has to move goes through a change request**, with the
   measurement that shows why the old value was wrong.
5. **A secret never belongs into a configuration file.** Pairing keys, PINs
   and certificates belong into the secure storage of the platform.

---

## 4. The most used values

| Question | The key |
|---|---|
| How long may the first picture take? | `latency.first_frame_ms` |
| How long may a silent peer hold a slot? | `latency.idle_timeout_ms` |
| How much may the process occupy? | `resources.max_memory_mb` |
| How much CPU may the process use? | `resources.max_cpu_percent` |
| How large may a single message be? | `limits.max_message_bytes` |
| How large may a queue become? | `queues.video_queue_bytes`, `queues.audio_queue_bytes` |
| How many headers may a request carry? | `limits.max_header_count` |
| How large may an artwork be? | `limits.max_artwork_bytes` |
| Is a PIN required? | `protocols.<name>.require_pin` |
| May a protocol serve several peers at the same time? | `session.multi_peer_protocols` |
| Which service types does a protocol advertise? | `protocols.<name>.mdns_service_types` |
| May the dashboard be reached from the network? | `web.allow_remote_access` (it may not) |
| Are the names of the senders masked in the log? | `privacy.mask_device_names` (in `security-policy.toml`) |

---

## 4a. The rule of the peers

```toml
[session]
multi_peer_protocols = ["cast"]
```

Most protocols serve **one** peer at a time: a second sender is refused,
not queued — a queued sender waits for a session that may never end, and
the sender sees a device that hangs. Google Cast is the documented
exception: a Cast receiver may serve several senders at the same time.

**The list is the one owner of the rule.** Every engine asks the
configuration, and the rule travels over the bridge to the native shells.
The list is read by the core (`SessionConfig`), handed to every engine in
the `EngineContext`, and read again by the shells from the configuration of
the receiver.

**The numbers of the peers of a protocol are not configurable.** The list
says *whether* a protocol may serve several peers; the number of peers that
the machine carries is a property of the protocol. Google Cast serves eight.

---

## 5. The limits of the configuration

- The configuration is read **once** at the start. A change of the file needs
  a restart of the receiver.
- The configuration is not a remote control. There is no endpoint that
  rewrites a file on the disk.
- The web backend may switch a protocol on and off in the running process.
  That is a change of the *state*, not of the *file*.

---

## 6. What is not configurable, and why

| Thing | Why |
|---|---|
| The dependency direction of the crates | It is a property of the build, not a setting |
| The rules of the parsers | They are security properties; a value that weakens them is a defect |
| The numeric error codes of the bridge | They are the public contract of the bridge |
| The exclusion list of the product | It is a product boundary, not a preference |
