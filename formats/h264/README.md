# phairplay-h264

H.264 bitstream parsing and RTP FU-A reassembly for
[PhairPlayRust](https://github.com/mazer666/PhairPlayRust).

The crate is pure: it transforms bytes into bytes and never touches the
network, the file system or the clock. It is the part of the video
receiver that runs before a decoder.

## What it does

| Feature | Detail |
|---|---|
| NAL units | The Annex-B split (three and four byte start codes), the length-prefixed (AVCC) container in both directions |
| Parameter sets | The sequence parameter set with its VUI, the picture parameter set, the cropping and the picture size |
| RTP | The reassembly of a fragmented NAL unit (FU-A), the single NAL unit packet and the aggregation packet (STAP-A) |
| Hardening | The Exp-Golomb code counts in `u64` and clamps, the picture size, the number of slice groups, the number of reference frames and the length of a reassembled NAL unit are bounded |

The Exp-Golomb reader is the reason the crate exists: a sequence
parameter set with 31 leading zero bits must not overflow the shift that
computes its code number.

## Example

```rust
use phairplay_h264::{split_annex_b, SequenceParameterSet};

let units = split_annex_b(&[0, 0, 0, 1, 0x67, 0x42, 0xc0, 0x1e]);
let sps = SequenceParameterSet::from_nal_payload(&units[0].payload)?;
assert_eq!(sps.width_in_pixels(), 704);
```

The crate documentation lives in [`src/lib.rs`](src/lib.rs).
