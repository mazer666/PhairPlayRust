# phairplay-dlna

The UPnP / DLNA media renderer of PhairPlayRust. The crate announces the
renderer over SSDP, serves the UPnP description documents and the service
descriptions, answers the SOAP actions of the three services
(`ConnectionManager`, `AVTransport`, `RenderingControl`) and notifies the
subscribers over GENA. It is the most exposed surface of the receiver:
every request comes from an arbitrary device of the local network, and the
GENA callback is a request *out* of the renderer. Both directions are
hardened — the XML reader rejects a document type, and the GENA guard
accepts a callback only from a literal private IPv4 address of the local
subnet.

The crate root documentation is in [`src/lib.rs`](src/lib.rs); the API is
re-exported from there.

The crate is part of the [PhairPlayRust](https://github.com/mazer666/PhairPlayRust)
workspace and is published under the AGPL-3.0-or-later licence. The build
contract that governs it lives in
[`docs/design/BUILD-CONTRACT.md`](https://github.com/mazer666/PhairPlayRust/blob/main/docs/design/BUILD-CONTRACT.md).
