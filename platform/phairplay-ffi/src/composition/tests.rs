#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;

#[test]
fn the_engines_are_registered_with_the_supervisor() {
    let receiver = build(&AppConfig::default()).expect("the receiver must start");
    for protocol in [
        ProtocolKind::AirPlay,
        ProtocolKind::Miracast,
        ProtocolKind::Cast,
        ProtocolKind::Dlna,
        ProtocolKind::SmartView,
    ] {
        assert!(
            receiver.engine(protocol).is_some(),
            "{protocol} has no engine"
        );
    }
}

#[test]
fn a_protocol_that_is_switched_off_is_not_started() {
    // WHY: a protocol that the configuration switches off must not
    // listen on a socket, even though its engine is registered.
    let mut config = AppConfig::default();
    config
        .protocols
        .entry("miracast".to_string())
        .or_default()
        .enabled = false;
    let receiver = build(&config).expect("the receiver must start");
    let startup = start(&receiver);

    let outcome = startup
        .outcomes
        .iter()
        .find(|outcome| outcome.protocol == ProtocolKind::Miracast)
        .expect("an outcome for every protocol");
    assert!(!outcome.is_running());
    assert!(outcome.reason.is_some());
    stop(&receiver);
}

#[test]
fn every_protocol_is_answered_for() {
    // WHY: the start is a total judgement. Every protocol of the
    // configuration gets an outcome — started, failed or switched off.
    let receiver = build(&AppConfig::default()).expect("the receiver must start");
    let startup = start(&receiver);
    assert_eq!(startup.outcomes.len(), ProtocolKind::ALL.len());
    stop(&receiver);
}

#[test]
fn the_start_reports_the_protocols_that_run() {
    let receiver = build(&AppConfig::default()).expect("the receiver must start");
    let startup = start(&receiver);
    // WHY: on a device without multicast the protocols that need it
    // fail. The others still run — that is the whole point of the
    // total judgement.
    assert!(startup.running() + startup.failed() <= ProtocolKind::ALL.len());
    for outcome in &startup.outcomes {
        if outcome.reason.is_some() {
            assert!(!outcome.is_running(), "{:?}", outcome.protocol);
        }
    }
    stop(&receiver);
}
