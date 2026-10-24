use super::*;

const JPEG: [u8; 6] = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
const PNG: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[test]
fn a_jpeg_is_accepted() {
    let result = validate_photo(&JPEG, Some("image/jpeg"), 1024);
    assert_eq!(
        result,
        PhotoValidation::Valid {
            image_type: PhotoImageType::Jpeg
        }
    );
}

#[test]
fn a_png_is_accepted() {
    let result = validate_photo(&PNG, Some("image/png"), 1024);
    assert_eq!(
        result,
        PhotoValidation::Valid {
            image_type: PhotoImageType::Png
        }
    );
}

#[test]
fn a_content_type_with_a_charset_is_accepted() {
    let result = validate_photo(&JPEG, Some("image/jpeg; charset=binary"), 1024);
    assert!(matches!(result, PhotoValidation::Valid { .. }));
}

#[test]
fn an_empty_payload_is_rejected() {
    let result = validate_photo(&[], None, 1024);
    assert!(matches!(result, PhotoValidation::Invalid { .. }));
}

#[test]
fn an_oversized_payload_is_rejected() {
    let payload = vec![0xFFu8, 0xD8, 0xFF];
    let result = validate_photo(&payload, None, 2);
    match result {
        PhotoValidation::Invalid { reason } => assert!(reason.contains("exceeds"), "{reason}"),
        PhotoValidation::Valid { .. } => panic!("must be rejected"),
    }
}

#[test]
fn random_bytes_are_rejected() {
    let payload = vec![0x42u8; 64];
    let result = validate_photo(&payload, None, 1024);
    assert!(matches!(result, PhotoValidation::Invalid { .. }));
}

#[test]
fn a_mismatched_content_type_is_rejected() {
    let result = validate_photo(&JPEG, Some("image/png"), 1024);
    match result {
        PhotoValidation::Invalid { reason } => {
            assert!(reason.contains("does not match"), "{reason}")
        }
        PhotoValidation::Valid { .. } => panic!("must be rejected"),
    }
}

#[test]
fn an_octet_stream_content_type_is_tolerated() {
    let result = validate_photo(&PNG, Some("application/octet-stream"), 1024);
    assert!(matches!(result, PhotoValidation::Valid { .. }));
}

#[test]
fn the_mime_types_are_stable() {
    assert_eq!(PhotoImageType::Jpeg.mime_type(), "image/jpeg");
    assert_eq!(PhotoImageType::Png.mime_type(), "image/png");
}
