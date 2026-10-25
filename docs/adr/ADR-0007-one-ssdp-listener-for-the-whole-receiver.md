# ADR-0007 — One SSDP listener for the whole receiver

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Chief Architect
**Related requirement IDs:** FR-009, FR-011, FR-100
**Amends:** ADR-0001 (the core may not depend on another crate — this ADR gives
the core a service of its own, but still no dependency).

---

## Context

The receiver announces **two** services over SSDP:

- the DLNA renderer, under `upnp:rootdevice` and
  `urn:schemas-upnp-org:device:MediaRenderer:1`, and
- the Samsung Smart View receiver, under
  `urn:samsung.com:service:MultiScreenService:1`.

Both live on the same multicast group, `239.255.255.250:1900`.

**Linux allows only one socket per multicast group.** `IP_ADD_MEMBERSHIP`
returns `EADDRINUSE` when the same (group, interface, local address) triple
is already joined — and unlike a TCP port, `SO_REUSEADDR` does not lift that
restriction. Two responders, one group: the second one fails to start.

The first version of the receiver had exactly that. The acceptance record of
M5 caught it, because the receiver runs all five protocols at the same time:

```text
DLNA / UPnP      not started — the SSDP multicast group could not be joined:
                  Address already in use (os error 98)
Samsung Smart View advertising
```

The receiver degraded — it ran with four of five protocols and reported the
reason — but a receiver that loses a protocol because of a socket option is
not a receiver that is finished.

## Decision

1. **The core owns one SSDP listener** — `core/phairplay-core/src/ssdp.rs`.
   It binds the multicast socket once, joins the group once, and reads the
   datagrams on one thread.
2. **Every service registers itself** with the listener. A service carries:
   - the type it is (`primary_target`) — the answer to a query for
     `ssdp:all` names the type of the service, not the wildcard, because the
     answer has to tell the control point *what* it found;
   - the targets it answers for;
   - the targets it announces itself under;
   - the address of its description document, its unique name and its server
     string;
   - **the rule that decides whether a message is a search it answers**.
3. **The rule belongs to the service, not to the listener.** A UPnP control
   point always sends the `MAN` header; some Samsung senders omit it; a
   datagram that announces a `Content-Length` it does not carry is malformed
   and is refused. These rules differ per protocol, so they live with the
   protocol.
4. **A service that stops leaves the registry; the listener stays.** The
   listener belongs to the process, not to one protocol. A receiver that
   switches DLNA off keeps announcing Smart View.
5. **The registry is keyed by the service, not by the device.** The two
   services belong to the **same** device and therefore carry the **same**
   unique name — `uuid:<the device id>` names the device, not the service.
   A registry that is keyed by the unique name alone holds only one of the
   two services: the second one pushes the first out of it. Both protocols
   would then report "advertising", and in the network the renderer would
   answer for the framework and the framework for the renderer. The key is
   therefore the unique name **and** the type of the service
   (`SsdpService::key`).

## Consequences

- All five protocols run at the same time. The proof is in the acceptance
  record of M5 and in the output of the receiver itself.
- The core now owns a network service. It still depends on no other crate of
  the workspace — the rule of ADR-0001 is untouched; what changed is that
  the core is not only a vocabulary but also the place where the shared
  infrastructure of the process lives.
- The two engines lost their own responder. What is left in each of them is
  the part that is actually different: the search targets, the server string
  and the rule of the protocol.
- The tests of the responder moved with it. There is one implementation and
  one place that proves it behaves.
- **The first version of the registry was keyed by the unique name alone.**
  The two services of the device share that name, so one of them pushed the
  other out of the registry — and the defect was invisible in the status of
  the protocols, because both of them reported "advertising". It was found
  by a network probe, not by a unit test: the test of the registry used two
  *different* unique names, which is not what the receiver produces. The
  test now uses the name that the receiver really produces, and the
  registry is keyed by `SsdpService::key`.

## Alternatives Considered

- **Two sockets with `SO_REUSEADDR`.** Rejected: the option does not lift
  the restriction of the kernel on the multicast membership.
- **Two sockets with different local addresses.** Rejected: a socket that is
  bound to a specific unicast address does not receive the datagrams of the
  group. The receiver would announce itself and never hear a query.
- **The core stays free of sockets; the bridge owns the listener.** Rejected:
  the bridge may depend on the engines, but the engines may not depend on
  the bridge — a listener in the bridge could not be reached by the engines
  that have to register themselves with it.
- **One protocol wins, the other is switched off.** Rejected: the user
  decides which protocols run, not the order in which the receiver starts
  them.

## Verification

- `cargo test -p phairplay-core --offline` — the tests of the listener: the
  search is recognised, the target is read, the answer carries the location,
  the target and the unique name, two services share one listener, and a
  service that is started twice answers once.
- `cargo test -p phairplay-dlna --offline` and
  `cargo test -p phairplay-smartview --offline` — the protocols keep their
  own rules and their own answers.
- `cargo test -p phairplay-core --offline ssdp` — the test
  `two_services_of_the_same_device_stay_in_the_registry` uses the unique
  name that the receiver really produces and proves that both services stay
  in the registry, that each of them answers for its own target, and that
  the removal of one does not take the other with it.
- The receiver itself: `phairplay-web --profile generic --no-dashboard`
  reports **5 of 5 protocols are running**.
- The network: an `M-SEARCH` for
  `urn:schemas-upnp-org:device:MediaRenderer:1` is answered by the renderer
  and by nothing else; an `M-SEARCH` for
  `urn:samsung.com:service:MultiScreenService:1` is answered by the
  framework and by nothing else; an `M-SEARCH` for `ssdp:all` is answered
  by both, once per service type, as the UPnP device architecture asks.
