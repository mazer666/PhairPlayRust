#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;

fn probe(sequence: u16, transmit: u64) -> [u8; 32] {
    let mut request = [0u8; 32];
    request[0] = 0x80;
    request[1] = REQUEST_TYPE;
    request[2] = (sequence >> 8) as u8;
    request[3] = sequence as u8;
    write_u32(&mut request[24..28], (transmit >> 32) as u32);
    write_u32(&mut request[28..32], (transmit & 0xFFFF_FFFF) as u32);
    request
}

#[test]
fn a_probe_is_answered_with_the_sender_sequence() {
    let request = probe(0x1234, 0x0000_0000_1000_0000);
    let response =
        TimingResponder::build_response(&request, 0x0000_0000_2000_0000).expect("valid probe");
    assert_eq!(response[0], 0x80);
    assert_eq!(response[1], RESPONSE_TYPE);
    assert_eq!(response[2], 0x12);
    assert_eq!(response[3], 0x34);
}

#[test]
fn the_response_echoes_the_sender_transmit_time_as_the_reference() {
    let request = probe(1, 0x0000_0000_1000_0000);
    let response =
        TimingResponder::build_response(&request, 0x0000_0000_2000_0000).expect("valid probe");
    assert_eq!(read_u32(&response[8..12]), 0x0000_0000);
    assert_eq!(read_u32(&response[12..16]), 0x1000_0000);
}

#[test]
fn the_response_carries_the_receive_timestamp() {
    let request = probe(1, 0);
    let response =
        TimingResponder::build_response(&request, 0x0000_0000_2000_0000).expect("valid probe");
    assert_eq!(read_u32(&response[16..20]), 0x0000_0000);
    assert_eq!(read_u32(&response[20..24]), 0x2000_0000);
}

#[test]
fn a_short_probe_is_ignored() {
    let responder = TimingResponder::new();
    let result = responder.handle_probe(&[0u8; 8], 0).expect("no error");
    assert!(result.is_none());
}

#[test]
fn a_packet_of_another_type_is_ignored() {
    let responder = TimingResponder::new();
    let mut request = probe(1, 0);
    request[1] = 0x99;
    let result = responder.handle_probe(&request, 0).expect("no error");
    assert!(result.is_none());
}

#[test]
fn a_probe_updates_the_clock_offset() {
    let responder = TimingResponder::new();
    // The sender claims it sent the packet 1 second after the NTP epoch.
    let request = probe(1, 0);
    let _ = responder.handle_probe(&request, 0).expect("no error");
    assert_eq!(responder.rtp_clock_offset_us(), 0);

    let request = probe(2, 1u64 << 32);
    let _ = responder.handle_probe(&request, 0).expect("no error");
    // The receiver's clock is at the epoch, the sender's is one second
    // ahead: the offset is negative.
    assert_eq!(responder.rtp_clock_offset_us(), -1_000_000);
}

#[test]
fn the_ntp_timestamp_is_in_the_ntp_epoch() {
    let timestamp = TimingResponder::current_ntp_timestamp();
    let seconds = timestamp >> 32;
    assert!(
        seconds > NTP_EPOCH_OFFSET,
        "the timestamp must be in the NTP epoch, got {seconds}"
    );
}

#[test]
fn a_fraction_is_converted_into_microseconds() {
    let half = 0x8000_0000u64;
    assert_eq!(TimingResponder::ntp_to_us(NTP_EPOCH_OFFSET, half), 500_000);
}
