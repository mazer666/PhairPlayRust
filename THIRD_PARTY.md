# Third-party components

> **Status:** binding inventory. Every third-party component that the
> workspace links, vendors or derives from is listed here with its
> licence and its origin. The list is checked by the licence audit.

The workspace itself is licensed under **AGPL-3.0-or-later**; see
[`LICENSE`](LICENSE) and [`LICENSES/`](LICENSES/). The components below
keep their own licence.

---

## Vendored or derived code

### Apple Lossless Audio Codec (ALAC) reference decoder

| Field | Value |
|---|---|
| Component | ALAC reference decoder (`ALACDecoder.cpp`, `ALACBitUtilities.c`, `ag_dec.c`, `dp_dec.c`, `matrix_dec.c`, `ALACAudioTypes.h`) |
| Origin | <https://developer.apple.com/downloads/> — Apple Inc. |
| Copyright | Copyright (c) 2001-2011 Apple Inc. |
| Licence | Apache License 2.0 ([`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt)) |
| Used by | `formats/alac` (`phairplay-alac`) |
| Form | **Derived work — native Rust port** |

The Rust crate `phairplay-alac` is a port of the reference decoder. The
port was written for this project; **no C source file was copied into the
tree**. The port keeps the observable behaviour of the reference
implementation, because the format is defined by it, and it replaces the
raw pointer arithmetic of the reference by slices with explicit lengths
and by real errors instead of unchecked reads.

Every file that is derived from the reference implementation carries an
attribution header that names the origin and the licence:

| Rust file | Derived from |
|---|---|
| `formats/alac/src/bits.rs` | `ALACBitUtilities.c`, `ALACBitUtilities.h` |
| `formats/alac/src/golomb.rs` | `ag_dec.c`, `aglib.h` |
| `formats/alac/src/predictor.rs` | `dp_dec.c`, `dplib.h` |
| `formats/alac/src/matrix.rs` | `matrix_dec.c`, `matrixlib.h` |
| `formats/alac/src/config.rs` | `ALACAudioTypes.h`, `ALACDecoder.cpp` |
| `formats/alac/src/decoder.rs` | `ALACDecoder.cpp`, `ALACDecoder.h` |
| `formats/alac/src/elements.rs` | `ALACDecoder.cpp` |
| `formats/alac/src/framing.rs` | `ALACDecoder.cpp` |

The reference implementation is the origin of the *format*, not of the
*code* in this repository. The Apache License 2.0 permits the derivation
and requires that the origin and the licence are mentioned; both are
mentioned in the attribution headers of the files listed above and in
this inventory.

The reference implementation was **not** vendored, so no copy of the
Apache License 2.0 has to travel with the source tree of the port; the
licence text is nevertheless kept in [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt)
because the port is a derived work.

---

## Rust dependencies

The Rust dependencies are declared in the root `Cargo.toml` and are
checked by the `cargo-deny` gate in the continuous integration. Their
licences are not repeated here; the gate is the inventory for them.
