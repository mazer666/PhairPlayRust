// WHAT: the XML side of the property-list codec.
// WHY: the module exists so that `lib.rs` can stay a thin façade and so
//      that the XML reader, decoder and encoder each stay under the file
//      length limit of the build contract.
// EXAMPLE: `phairplay_plist::xml::decode` turns a `<plist>` document
//          into a [`Value`](crate::Value).

mod decode;
mod encode;
mod reader;

pub(crate) use decode::decode;
pub(crate) use encode::encode;
