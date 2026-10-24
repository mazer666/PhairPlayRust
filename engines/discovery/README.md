# phairplay-discovery

Service discovery for the PhairPlayRust receiver stack. This crate
publishes the receiver on the local network: it registers the four DNS-SD
services the protocols of PhairPlayRust need — `_airplay._tcp`, `_raop._tcp`,
`_googlecast._tcp` and `_miracast._tcp` — with the TXT records that tell a
sender what the receiver can do, and it can browse the network over SSDP to
find the devices that are out there. The AirPlay pair is registered as a
unit: a receiver that publishes only one of the two is invisible to some
senders.

The crate root documentation is in
[`src/lib.rs`](src/lib.rs); the API is re-exported from there.

The crate is part of the [PhairPlayRust](https://github.com/mazer666/PhairPlayRust)
workspace and is published under the AGPL-3.0-or-later licence. The build
contract that governs it lives in
[`docs/design/BUILD-CONTRACT.md`](https://github.com/mazer666/PhairPlayRust/blob/main/docs/design/BUILD-CONTRACT.md).
