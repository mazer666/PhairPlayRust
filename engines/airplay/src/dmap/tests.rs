#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

fn tag(name: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = name.to_vec();
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

#[test]
fn a_flat_document_yields_the_metadata() {
    let mut body = tag(b"minm", b"Song");
    body.extend(tag(b"asar", b"Artist"));
    body.extend(tag(b"asal", b"Album"));
    let metadata = parse_now_playing(&body).expect("valid dmap");
    assert_eq!(metadata.title.as_deref(), Some("Song"));
    assert_eq!(metadata.artist.as_deref(), Some("Artist"));
    assert_eq!(metadata.album.as_deref(), Some("Album"));
    assert!(metadata.has_metadata());
}

#[test]
fn a_nested_listing_item_yields_the_metadata() {
    let mut inner = tag(b"minm", b"Nested");
    inner.extend(tag(b"asar", b"Nested Artist"));
    let body = tag(b"mlit", &inner);
    let metadata = parse_now_playing(&body).expect("valid dmap");
    assert_eq!(metadata.title.as_deref(), Some("Nested"));
    assert_eq!(metadata.artist.as_deref(), Some("Nested Artist"));
}

#[test]
fn a_length_that_overruns_the_buffer_is_rejected() {
    let mut body = Vec::new();
    body.extend_from_slice(b"minm");
    body.extend_from_slice(&u32::MAX.to_be_bytes());
    body.extend_from_slice(b"short");
    let error = parse_now_playing(&body).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_body_that_is_not_a_dmap_document_is_rejected() {
    let body = vec![0x00u8; 16];
    let error = parse_now_playing(&body).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_too_short_body_is_rejected() {
    let error = parse_now_playing(b"minm").expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_deeply_nested_document_does_not_recurse_forever() {
    let mut body = tag(b"minm", b"Deep");
    for _ in 0..12 {
        body = tag(b"mlit", &body);
    }
    let metadata = parse_now_playing(&body).expect("valid dmap");
    // The document nests deeper than the limit, so the innermost tag is
    // never reached — but the parser must return instead of recursing.
    assert_eq!(metadata.title, None);
}

#[test]
fn an_empty_value_is_treated_as_absent() {
    let body = tag(b"minm", b"   ");
    let metadata = parse_now_playing(&body).expect("valid dmap");
    assert_eq!(metadata.title, None);
    assert!(!metadata.has_metadata());
}
