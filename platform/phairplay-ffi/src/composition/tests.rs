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
fn switching_one_protocol_off_does_not_take_the_other_one_with_it() {
    // WHY: the two protocols that share a discovery port — DLNA and Samsung
    // Smart View — are served by ONE listener. A user who switches one of
    // them off expects the other one to keep working; a shared listener
    // that takes both down with it would make the switch a switch for two
    // protocols.
    let mut config = AppConfig::default();
    config
        .protocols
        .entry("smartview".to_string())
        .or_default()
        .enabled = false;
    let receiver = build(&config).expect("the receiver must start");
    let startup = start(&receiver);

    let smartview = startup
        .outcomes
        .iter()
        .find(|outcome| outcome.protocol == ProtocolKind::SmartView)
        .expect("an outcome for every protocol");
    assert!(
        !smartview.is_running(),
        "the switched-off protocol runs: {smartview:?}"
    );
    assert!(
        smartview.reason.is_some(),
        "the switched-off protocol has no reason: {smartview:?}"
    );

    // WHY the other one: the DLNA renderer shares the listener with the
    // framework. It has to keep running.
    let dlna = startup
        .outcomes
        .iter()
        .find(|outcome| outcome.protocol == ProtocolKind::Dlna)
        .expect("an outcome for every protocol");
    assert!(
        dlna.is_running(),
        "the switched-off protocol took the other one with it: {dlna:?}"
    );
    stop(&receiver);
}

#[test]
fn both_protocols_of_the_shared_listener_run_together() {
    // WHY: the two services of the shared listener must be able to run at
    // the same time — that is the reason the listener lives in the core.
    // Without this, one of the two protocols is lost to a socket option.
    let receiver = build(&AppConfig::default()).expect("the receiver must start");
    let startup = start(&receiver);
    for protocol in [ProtocolKind::Dlna, ProtocolKind::SmartView] {
        let outcome = startup
            .outcomes
            .iter()
            .find(|outcome| outcome.protocol == protocol)
            .expect("an outcome for every protocol");
        assert!(
            outcome.is_running(),
            "{protocol} does not run next to its neighbour: {outcome:?}"
        );
    }
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
