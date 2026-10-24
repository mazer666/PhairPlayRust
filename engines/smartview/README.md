# phairplay-smartview

The Samsung Smart View engine of PhairPlayRust. The crate announces the
receiver over SSDP with the search target
`urn:samsung.com:service:MultiScreenService:1`, serves the description
document that the `LOCATION` header points at, answers the HTTP API below
`/api/v2/` and accepts the WebSocket upgrade of the channel.

**The honest limit:** the message format of the Smart View channel is
proprietary and not publicly documented, so it is **not** implemented. What
the crate implements is the discovery and the handshake — the part that
makes a device a Smart View target. A Galaxy device finds the device and
connects to it, but no picture flows yet. Smart View needs no HDCP
protection and no decryption: it is an open Samsung feature, not a DRM
pipeline.

The crate root documentation is in [`src/lib.rs`](src/lib.rs); the API is
re-exported from there.

The crate is part of the [PhairPlayRust](https://github.com/mazer666/PhairPlayRust)
workspace and is published under the AGPL-3.0-or-later licence. The build
contract that governs it lives in
[`docs/design/BUILD-CONTRACT.md`](https://github.com/mazer666/PhairPlayRust/blob/main/docs/design/BUILD-CONTRACT.md).
