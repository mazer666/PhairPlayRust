// WHAT: the binary side of the property-list codec.
// WHY: the module exists so that `lib.rs` can stay a thin façade and so
//      that the binary reader and writer each stay under the file length
//      limit of the build contract.
// EXAMPLE: `phairplay_plist::binary::decode` turns a `bplist00` file
//          into a [`Value`](crate::Value).

mod decode;
mod encode;
mod objects;

pub(crate) use decode::decode;
pub(crate) use encode::encode;

/// The eight bytes every binary property list starts with.
pub(crate) const MAGIC: &[u8; 8] = b"bplist00";
