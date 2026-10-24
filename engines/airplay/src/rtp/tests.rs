
use super::*;

use crate::error::AirPlayError;

fn rtp_frame(payload: &[u8], timestamp: u32) -> Vec<u8> {
    let mut frame = vec![0x80, 0x60, 0x00, 0x01];
    frame.extend_from_slice(&timestamp.to_be_bytes());
    frame.extend_from_slice(&[0, 0, 0, 0]);
    frame.extend_from_slice(payload);
    frame
}

#[test]
fn an_rtp_frame_is_split_into_header_and_payload() {
    let frame = rtp_frame(&[0x65, 0xAA], 90_000);
    let (header, payload) = parse_rtp(&frame).expect("valid frame");
    assert_eq!(header.payload_type, 96);
    assert_eq!(header.timestamp, 90_000);
    assert_eq!(payload, &[0x65, 0xAA]);
}

#[test]
fn a_short_frame_is_rejected() {
    let error = parse_rtp(&[0x80, 0x60]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_truncated_header_extension_is_rejected() {
    let mut frame = vec![0x90, 0x60, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0];
    frame.extend_from_slice(&[0xBE, 0xDE, 0x00, 0x02]);
    let error = parse_rtp(&frame).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_header_extension_is_skipped() {
    let mut frame = vec![0x90, 0x60, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0];
    frame.extend_from_slice(&[0xBE, 0xDE, 0x00, 0x01]);
    frame.extend_from_slice(&[0, 0, 0, 0]);
    frame.extend_from_slice(&[0x65, 0xAA]);
    let (_, payload) = parse_rtp(&frame).expect("valid frame");
    assert_eq!(payload, &[0x65, 0xAA]);
}

#[test]
fn a_timestamp_becomes_microseconds() {
    assert_eq!(timestamp_to_micros(90_000), 1_000_000);
}

#[test]
fn a_complete_nal_unit_is_delivered_unchanged() {
    let mut reassembler = FuAReassembler::new();
    let nal = reassembler.push(&[0x65, 0xAA], 42).expect("a NAL unit");
    assert_eq!(nal.data, vec![0x65, 0xAA]);
    assert_eq!(nal.pts_us, 42);
}

#[test]
fn a_fragmented_nal_unit_is_reassembled() {
    let mut reassembler = FuAReassembler::new();
    assert!(reassembler.push(&[0x7C, 0x85, 0xAA], 7).is_none());
    assert!(reassembler.is_accumulating());
    assert!(reassembler.push(&[0x7C, 0x05, 0xBB], 7).is_none());
    let nal = reassembler.push(&[0x7C, 0x45, 0xCC], 7).expect("complete");
    assert_eq!(nal.data, vec![0x65, 0xAA, 0xBB, 0xCC]);
    assert!(!reassembler.is_accumulating());
}

#[test]
fn a_fragment_without_a_start_bit_is_dropped() {
    let mut reassembler = FuAReassembler::new();
    assert!(reassembler.push(&[0x7C, 0x45, 0xAA], 0).is_none());
    assert!(!reassembler.is_accumulating());
}

#[test]
fn a_new_fragment_discards_an_incomplete_one() {
    let mut reassembler = FuAReassembler::new();
    assert!(reassembler.push(&[0x7C, 0x85, 0xAA], 0).is_none());
    assert!(reassembler.push(&[0x7C, 0x85, 0xBB], 0).is_none());
    let nal = reassembler.push(&[0x7C, 0x45, 0xCC], 0).expect("complete");
    assert_eq!(nal.data, vec![0x65, 0xBB, 0xCC]);
}

#[test]
fn a_complete_nal_unit_drops_an_unfinished_fragment() {
    let mut reassembler = FuAReassembler::new();
    assert!(reassembler.push(&[0x7C, 0x85, 0xAA], 0).is_none());
    let nal = reassembler.push(&[0x65, 0xBB], 0).expect("a NAL unit");
    assert_eq!(nal.data, vec![0x65, 0xBB]);
    assert!(!reassembler.is_accumulating());
}

#[test]
fn an_empty_payload_is_ignored() {
    let mut reassembler = FuAReassembler::new();
    assert!(reassembler.push(&[], 0).is_none());
}

#[tokio::test]
async fn an_interleaved_frame_is_read() {
    let (client, server) = tokio::io::duplex(1024);
    tokio::spawn(async move {
        let mut client = client;
        use tokio::io::AsyncWriteExt;
        let _ = client.write_all(&[0x24, 0x00, 0x00, 0x03, 0xAA, 0xBB, 0xCC]).await;
    });
    let mut stream = StreamBuffer::new(server);
    let frame = read_interleaved_frame(&mut stream, 1024, Duration::from_secs(1))
        .await
        .expect("read")
        .expect("a frame");
    assert_eq!(frame.channel, 0);
    assert_eq!(frame.payload, vec![0xAA, 0xBB, 0xCC]);
}

#[tokio::test]
async fn an_oversized_frame_is_rejected() {
    let (client, server) = tokio::io::duplex(1024);
    tokio::spawn(async move {
        let mut client = client;
        use tokio::io::AsyncWriteExt;
        let _ = client.write_all(&[0x24, 0x00, 0xFF, 0xFF]).await;
    });
    let mut stream = StreamBuffer::new(server);
    let error = read_interleaved_frame(&mut stream, 1024, Duration::from_secs(1))
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}

#[tokio::test]
async fn a_clean_end_of_stream_yields_no_frame() {
    let (client, server) = tokio::io::duplex(1024);
    drop(client);
    let mut stream = StreamBuffer::new(server);
    let frame = read_interleaved_frame(&mut stream, 1024, Duration::from_secs(1))
        .await
        .expect("read");
    assert!(frame.is_none());
}
