use super::*;
use crate::binary::encode::encode;

fn limits() -> PlistLimits {
    PlistLimits::default()
}

/// Builds a minimal but valid binary property list around `objects`.
///
/// `objects` holds the raw bytes of every object; the helper writes
/// the header, the offset table and the trailer.
fn build(objects: &[Vec<u8>], top: u64) -> Vec<u8> {
    let mut offsets = Vec::new();
    let mut cursor = MAGIC.len() as u64;
    for object in objects {
        offsets.push(cursor);
        cursor += object.len() as u64;
    }
    let table_offset = cursor;
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    for object in objects {
        out.extend_from_slice(object);
    }
    for offset in &offsets {
        out.push(*offset as u8);
    }
    let mut trailer = [0_u8; TRAILER_LEN];
    trailer[6] = 1;
    trailer[7] = 1;
    trailer[8..16].copy_from_slice(&(objects.len() as u64).to_be_bytes());
    trailer[16..24].copy_from_slice(&top.to_be_bytes());
    trailer[24..32].copy_from_slice(&table_offset.to_be_bytes());
    out.extend_from_slice(&trailer);
    out
}

#[test]
fn decodes_a_hand_written_dictionary() {
    // A dictionary with the single key "a" and the value "b".
    let bytes = build(
        &[vec![0xd1, 0x01, 0x02], vec![0x51, b'a'], vec![0x51, b'b']],
        0,
    );
    let decoded = decode(&bytes, &limits()).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        decoded,
        Value::dictionary([("a", Value::String("b".into()))])
    );
}

#[test]
fn agrees_with_the_encoder() {
    let value = Value::dictionary([("a", Value::String("b".into())), ("c", Value::Integer(3))]);
    let encoded = encode(&value, &limits()).unwrap_or_default();
    let decoded = decode(&encoded, &limits()).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(decoded, value);
}

#[test]
fn rejects_a_wrong_magic() {
    let mut bytes = encode(&Value::Dictionary(Vec::new()), &limits()).unwrap_or_default();
    bytes[0] = b'x';
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::BadBinaryMagic)
    ));
}

#[test]
fn rejects_a_truncated_file() {
    assert!(matches!(
        decode(b"bplist", &limits()),
        Err(PlistError::Truncated { .. })
    ));
}

#[test]
fn rejects_an_absurd_object_count() {
    let mut bytes = encode(&Value::Dictionary(Vec::new()), &limits()).unwrap_or_default();
    let trailer = bytes.len() - TRAILER_LEN;
    bytes[trailer + 8..trailer + 16].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::TooManyObjects { .. })
    ));
}

#[test]
fn rejects_an_illegal_offset_width() {
    let mut bytes = encode(&Value::Dictionary(Vec::new()), &limits()).unwrap_or_default();
    let trailer = bytes.len() - TRAILER_LEN;
    bytes[trailer + 6] = 0;
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::BadWidth { what: "offset", .. })
    ));
}

#[test]
fn rejects_an_illegal_reference_width() {
    let mut bytes = encode(&Value::Dictionary(Vec::new()), &limits()).unwrap_or_default();
    let trailer = bytes.len() - TRAILER_LEN;
    bytes[trailer + 7] = 9;
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::BadWidth {
            what: "object reference",
            ..
        })
    ));
}

#[test]
fn rejects_a_length_field_that_exceeds_the_input() {
    // A data object that claims 0xFFFFFFFF bytes: the marker is 0x4F,
    // the inline integer is a four byte value 0xFFFFFFFF.
    let bytes = build(&[vec![0x4f, 0x12, 0xff, 0xff, 0xff, 0xff]], 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the length check to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::DataTooLarge { length, .. } if length == 0xffff_ffff),
        "{error:?}"
    );
}

#[test]
fn rejects_a_data_blob_that_reaches_past_the_buffer() {
    // The length fits into the budget of the decoder, but the buffer
    // does not hold the announced bytes.
    let mut object = vec![0x4f, 0x10, 100];
    object.extend(std::iter::repeat_n(0x00, 3));
    let bytes = build(&[object], 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the length check to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::LengthExceedsInput { length: 100, .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_a_data_blob_of_one_gigabyte() {
    // The count field announces 1 GB, which is inside the file only
    // if the decoder believes the length without checking it.
    let bytes = build(&[vec![0x4f, 0x13, 0, 0, 0, 0, 0x40, 0, 0, 0]], 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the length check to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(
            error,
            PlistError::DataTooLarge {
                length: 0x4000_0000,
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn rejects_a_cycle() {
    // A dictionary whose value refers back to the dictionary itself.
    let bytes = build(&[vec![0xd1, 0x01, 0x00], vec![0x51, b'a']], 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the cycle to be detected"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::CyclicReference { .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_an_object_offset_outside_the_object_region() {
    let mut bytes = build(&[vec![0xd1, 0x01, 0x01], vec![0x51, b'a']], 0);
    // The offset table starts right after the objects.
    let table_offset = bytes.len() - TRAILER_LEN - 2;
    bytes[table_offset] = 0xff;
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::BadTrailer { .. })
    ));
}

#[test]
fn rejects_an_object_reference_outside_the_table() {
    let bytes = build(&[vec![0xa1, 0x09], vec![0x51, b'a']], 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the reference check to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::ObjectIndexOutOfRange { .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_an_array_that_is_too_large() {
    // An array that announces 4294967295 elements.
    let objects = vec![vec![0xaf, 0x12, 0xff, 0xff, 0xff, 0xff], vec![0x51, b'a']];
    let bytes = build(&objects, 0);
    let error = match decode(&bytes, &limits()) {
        Ok(_) => panic!("expected the container check to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::ContainerTooLarge { .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_an_object_graph_that_expands_too_far() {
    // An array of hundred references to the same dictionary. The
    // dictionary is decoded once, but the expansion counts every copy
    // of it.
    let mut array = vec![0xaf, 0x12, 0x00, 0x00, 0x00, 0x64];
    array.extend(std::iter::repeat_n(0x01, 100));
    let objects = vec![
        array,
        vec![0xd1, 0x02, 0x03],
        vec![0x51, b'a'],
        vec![0x10, 0x01],
    ];
    let bytes = build(&objects, 0);

    let mut limits = limits();
    limits.max_expanded_nodes = 50;
    let error = match decode(&bytes, &limits) {
        Ok(_) => panic!("expected the node budget to bite"),
        Err(err) => err,
    };
    assert!(
        matches!(error, PlistError::NodeBudgetExceeded { .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_a_null_object() {
    let bytes = build(&[vec![0x00]], 0);
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::UnsupportedObject { .. })
    ));
}

#[test]
fn rejects_an_unknown_object_marker() {
    let bytes = build(&[vec![0xe1, 0x00]], 0);
    assert!(matches!(
        decode(&bytes, &limits()),
        Err(PlistError::UnknownObjectType { .. })
    ));
}
