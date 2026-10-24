# Troubleshooting

A symptom, the probable cause, the check. The order is deliberate: the
cheapest check comes first.

---

## 1. The sender does not see the receiver

| Probable cause | The check |
|---|---|
| The multicast of the network is filtered | `ping` the sender from the receiver; a router with "AP isolation" or a filtered multicast blocks the discovery |
| The service type is wrong | AirPlay needs **both** `_airplay._tcp` and `_raop._tcp`; check the advertisement of the receiver |
| The sender and the receiver are in different subnets | the peer must be inside the permitted subnet; the receiver refuses the rest with `NetworkError::PeerOutsideSubnet` |
| The protocol is switched off | the `enabled` flag of the protocol in the configuration, and the state in the dashboard |
| The platform lacks the permission | Miracast needs the WFD feature of the platform; a device without it does not advertise the service (`FR-035`) |

The check of the traffic: the capture filters and the ports of the four
protocols are in [`PROTOCOL_OVERVIEW.md`](PROTOCOL_OVERVIEW.md) §7.

---

## 2. The device appears, the picture does not

| Probable cause | The check |
|---|---|
| The sender requires a codec the device cannot decode | the advertised MIME types of the capability model against the decoders of the platform |
| The session is refused by the protocol baseline | the baseline checks of the core: `supports_airplay_baseline`, `supports_cast_baseline`, `supports_miracast_baseline` |
| The sender expects HDCP and the platform cannot protect the session | the session is refused on purpose; the log names the reason (`EXC-003`) |
| The content is FairPlay protected | it is not decryptable, and the receiver says so (`EXC-001`) |

---

## 3. The picture stutters

| Probable cause | The check |
|---|---|
| The decoder cannot keep up | the drop counter of the core: `video_frames_dropped` against `video_frames` |
| The queue is bounded and the sender is faster | the queue budgets in the configuration; a raise needs a change request and a measurement |
| The network is saturated | the measurement of the frame rate over the whole window, not the average of the first seconds |
| The CPU is at the budget | the CPU measurement of the process over the window |

---

## 4. The receiver does not appear in the dashboard

| Probable cause | The check |
|---|---|
| The dashboard is not running | the bind address of the configuration; the backend binds on the loopback interface only |
| The browser is on another machine | the dashboard is a local panel; it is not reachable from the network, by design |
| The event channel is not connected | the event endpoint of the web backend |

---

## 5. A peer is refused

| Probable cause | The check |
|---|---|
| A session is already active on that protocol | one peer at a time; the second connection is refused, not queued |
| The access control requires a PIN | the `require_pin` flag of the protocol; the number of failed attempts is the `max_pairing_attempts` key |
| A limit was reached | the `limits` section of the configuration: the message size, the header count, the header length |
| The peer is outside the subnet | `NetworkError::PeerOutsideSubnet` |

---

## 6. The log does not show what I need

The log masks the names and the addresses of the senders on purpose. That is
the privacy promise of the project, and it is not a setting that a user
should be able to switch off casually.

For the development of a protocol, the traffic is the better source anyway:

```bash
sudo tcpdump -i any -n -A
```

---

## 7. The build fails

| Message | The cause | The check |
|---|---|---|
| `unsafe` is denied | the workspace lint | an ADR is needed, and the override is local to the crate, never global |
| `unwrap` / `expect` is denied | the workspace lint | a missing value is an error, not a panic |
| `missing_docs` warns | the workspace lint | every public item is documented |
| a crate does not compile after a change of the core | the contract of the workspace | a signature of the core needs a change request |

---

## 8. A check did not run

A check that could not be executed is reported as **NOT RUN** with the exact
reason: a missing tool, a missing device, a missing credential. It is never
reported as a success, and it is never quietly left out of the acceptance
record.
