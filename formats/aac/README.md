# phairplay-aac

AAC and AAC-ELD framing for
[PhairPlayRust](https://github.com/mazer666/PhairPlayRust).

The crate is pure: it transforms bytes into bytes and never touches the
network, the file system or the clock. It tells the audio sink how many
samples a packet carries and how many bits per second the stream uses.

## What it does

| Feature | Detail |
|---|---|
| ADTS | The seven byte header, the optional CRC, the sample rate, the channel configuration and the frame length |
| AAC-ELD | The 42 bit frame header with its sync word, the flags and the channel count |
| Samples | The number of samples per frame (1024, or 960 for the low delay profile), per packet and per stream |
| Bit rate | The bit rate of the format and the bit rate that a measured packet size implies |
| Hardening | The frame length is checked against the bytes that are present, the sample rate and the channel configuration are checked against the tables of the format |

## Example

```rust
use phairplay_aac::{split_adts, total_samples, AdtsHeader};

let stream = [0xff, 0xf1, 0x50, 0x80, 0x01, 0x1f, 0xfc, 0x00];
let frames = split_adts(&stream)?;
assert_eq!(frames[0].sample_rate, 44_100);
assert_eq!(total_samples(&frames), 1024);
```

The crate documentation lives in [`src/lib.rs`](src/lib.rs).
