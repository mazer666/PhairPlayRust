# phairplay-airplay

The AirPlay 2 receiver engine of [PhairPlayRust](https://github.com/mazer666/PhairPlayRust).

This crate implements the *receiver* side of AirPlay 2: the RTSP control
channel with its HTTP-like handshake routes, the SDP negotiation, the
interleaved RTP transport, the pairing handshakes (Ed25519/X25519 and the
legacy SRP-6a PIN flow), the FairPlay key exchange, the DAAP/DMAP now-playing
metadata, the DACP reverse control channel, the Apple NTP timing service and
the photo endpoint.

The crate is a protocol *state machine*: it owns the sockets, it accepts
exactly one sender at a time (a second one is answered with
`503 Service Unavailable`), and it reports what happens through the
`phairplay_core::protocol::ProtocolEngine` trait.

Every byte that arrives from the network is treated as hostile input: the
reader enforces a request-line, header-count, header-line, body and message
limit, every blocking read has a deadline, and every parser rejects malformed
input instead of repairing it.

See the crate documentation (`cargo doc -p phairplay-airplay`) for the module
map, and `deliverable.md` in the task output directory for the feature
matrix, the hardening evidence and the known boundaries.

Licensed under AGPL-3.0-or-later, like the rest of the workspace.
