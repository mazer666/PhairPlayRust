# phairplay-alac

Apple Lossless (ALAC) decoder for
[PhairPlayRust](https://github.com/mazer666/PhairPlayRust).

The crate is pure: it transforms bytes into bytes and never touches the
network, the file system or the clock. It decodes a lossless frame into
the samples of the stream.

## What it does

| Feature | Detail |
|---|---|
| Magic cookie | The configuration of the stream, with and without the `frma` and `alac` atoms |
| Frame | The single channel element, the channel pair element, the data stream element, the fill element and the end of the frame |
| Residuals | The adaptive Golomb decoder of the format, with its running mean |
| Prediction | The dynamic predictor with its four, eight and general coefficient paths |
| Mixing | The undoing of the channel mixing, including the shifted out low bytes |
| Hardening | Every read is checked, the announced sample count is compared with the configuration, the magic cookie is validated before a buffer is allocated |

## Port and licence

The decoder is a **native Rust port** of the reference decoder by Apple
Inc. (`ALACDecoder.cpp`, `ALACBitUtilities.c`, `ag_dec.c`, `dp_dec.c`,
`matrix_dec.c`), Copyright (c) 2001-2011 Apple Inc., licensed under the
Apache License, Version 2.0. **No C code was copied into the tree.** Every
derived file carries an attribution header, and the reference
implementation is listed in [`THIRD_PARTY.md`](../../THIRD_PARTY.md).

The port replaces the raw pointer arithmetic of the reference by slices
with explicit lengths and by real errors, and it needs no `unsafe`.

## Example

```rust
use phairplay_alac::{AlacConfig, AlacDecoder};

let config = AlacConfig {
    frame_length: 4096,
    bit_depth: 16,
    num_channels: 2,
    ..AlacConfig::default()
};
let decoder = AlacDecoder::new(config)?;
```

The crate documentation lives in [`src/lib.rs`](src/lib.rs).
