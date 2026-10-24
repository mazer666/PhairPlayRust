# phairplay-plist

Apple property list codec for [PhairPlayRust](https://github.com/mazer666/PhairPlayRust):
the XML form and the binary form (`bplist00`), both directions.

The crate is pure: it transforms bytes into bytes and never touches the
network, the file system or the clock. It is the codec that the AirPlay 2
handshake and the Google Cast protocol need.

## What it does

| Feature | Detail |
|---|---|
| XML decode | `plist`, `dict`, `array`, `key`, `string`, `integer`, `real`, `true`, `false`, `data`, `date` |
| XML encode | The same eight value types, with the escaping of the five predefined entities |
| Binary decode | Trailer, offset table, object references, integer, real, bool, date, data, string (ASCII and UTF-16), array, dictionary |
| Binary encode | The same object types, with the smallest integer width and the inline count escape of the format |
| Hardening | Object count, nesting depth, container length, string and data length, node budget, reference cycles, internal DTD subsets |

The limits live in `PlistLimits` and can be tightened by the engine from
`AppConfig::limits`.

## Example

```rust
use phairplay_plist::{decode, encode_binary, Value};

let value = Value::dictionary([
    ("name", Value::String("Living Room".into())),
    ("volume", Value::Integer(42)),
]);
let bytes = encode_binary(&value)?;
assert_eq!(decode(&bytes)?, value);
```

The crate documentation lives in [`src/lib.rs`](src/lib.rs).
