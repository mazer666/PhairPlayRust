"""The roundtrip of the bridge through the real C ABI.

WHAT: The proof that the *binding* works and not only the Rust code. The
script loads the generated Python bindings, which call the shared library
through the C ABI, and pushes a value through it: in, through, out — and
compares the value that comes out with the value that went in.

WHY: A Rust test proves that the conversions are correct. Only a foreign
caller proves that the *wire format* is correct: the buffer layout, the
lifting of the optional values, the numbers that are too large for a
32-bit integer, the error that has to arrive as a structured failure and
not as a crash. Those are the bugs that a Rust test cannot see.

The script is driven by ``tests/foreign_abi_roundtrip.rs``; it is a test,
not an example.

EXAMPLE: ``python3 roundtrip.py`` — with the generated bindings and the
shared library in the same directory.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import phairplay_ffi as ffi  # noqa: E402  (the path has to be set first)


def a_configuration():
    """A configuration with a value in every field."""
    return ffi.FfiAppConfig(
        device=ffi.FfiDeviceConfig(
            name="Attic Receiver",
            model="PhairPlay Rust Receiver",
            manufacturer="PhairPlayRust",
            protocol_version="2.5",
        ),
        protocols=[
            ffi.FfiProtocolConfig(
                name="airplay",
                enabled=False,
                port=7000,
                require_pin=True,
                max_message_bytes=4096,
            ),
            ffi.FfiProtocolConfig(
                name="cast",
                enabled=True,
                port=8009,
                require_pin=False,
                max_message_bytes=65536,
            ),
        ],
        limits=ffi.FfiLimitsConfig(
            video_queue_frames=42,
            video_queue_bytes=12345678,
            audio_queue_packets=7,
            audio_queue_bytes=7654321,
            max_pairing_attempts=3,
            max_artwork_bytes=111,
            max_photo_bytes=222,
            max_media_bytes=333,
            max_header_count=44,
            max_header_bytes=555,
        ),
        performance=ffi.FfiPerformanceConfig(
            first_frame_latency_ms=66,
            minimum_fps=45,
            max_memory_mb=123,
            max_cpu_percent=55,
            idle_timeout_ms=9999,
        ),
    )


class EventSink:
    """A host that records the events the bridge delivers."""

    def __init__(self):
        self.events = []

    def on_event(self, event):
        self.events.append(event)


def the_configuration_survives_the_bridge():
    """Wert rein, durch die Brücke, Wert wieder raus — identisch."""
    config = a_configuration()
    receiver = ffi.Receiver(config)
    read_back = receiver.config()

    assert read_back == config, "the configuration changed on the way: {0} != {1}".format(
        read_back, config
    )
    assert read_back.device.name == "Attic Receiver"
    assert read_back.device.protocol_version == "2.5"
    assert len(read_back.protocols) == 2
    assert read_back.protocols[0].name == "airplay"
    assert read_back.protocols[0].port == 7000
    assert read_back.protocols[0].require_pin is True
    assert read_back.limits.video_queue_frames == 42
    assert read_back.limits.max_header_bytes == 555
    assert read_back.limits.audio_queue_bytes == 7654321
    assert read_back.performance.idle_timeout_ms == 9999
    print("ok  the configuration survives the bridge through the C ABI")


def the_status_survives_the_bridge():
    """The status of every protocol crosses the boundary unchanged."""
    receiver = ffi.Receiver(a_configuration())
    status = receiver.status()

    assert len(status) == 4, status
    assert [entry.protocol for entry in status] == [
        ffi.FfiProtocolKind.AIR_PLAY,
        ffi.FfiProtocolKind.MIRACAST,
        ffi.FfiProtocolKind.CAST,
        ffi.FfiProtocolKind.DLNA,
    ], status
    # The configuration of the proof disables the AirPlay engine and says
    # nothing about Miracast, which therefore keeps the default.
    assert status[0].enabled is False, status[0]
    assert status[1].enabled is True, status[1]
    assert status[0].peer_count == 0
    assert status[0].tcp_port is None
    assert status[0].state == ffi.FfiProtocolState.IDLE, status[0]
    print("ok  the status survives the bridge through the C ABI")


def the_identity_survives_the_bridge():
    """The identity crosses the boundary unchanged."""
    receiver = ffi.Receiver(a_configuration())
    identity = receiver.identity()

    assert identity.name == "Attic Receiver", identity
    assert identity.manufacturer == "PhairPlayRust", identity
    assert identity.service_instance_name == "Attic Receiver", identity
    assert identity.device_id != "", identity
    print("ok  the identity survives the bridge through the C ABI")


def the_counters_survive_the_bridge():
    """The counters cross the boundary unchanged."""
    receiver = ffi.Receiver(a_configuration())
    stats = receiver.stats("airplay")

    assert stats.video_frames == 0, stats
    assert stats.drop_ratio == 0.0, stats
    assert stats.bytes_received == 0, stats
    print("ok  the counters survive the bridge through the C ABI")


def an_error_arrives_as_a_structured_failure():
    """A failure arrives with its code — and it does not crash the host."""
    receiver = ffi.Receiver(a_configuration())

    try:
        receiver.protocol_status("telepathy")
    except ffi.FfiError.Failed as error:
        # WHY: the number is the contract of the core.
        assert error.code == 1, error.code
        assert "telepathy" in error.details, error.details
    else:
        raise AssertionError("the unknown protocol must be rejected")

    # A protocol that is disabled in the configuration is refused with the
    # code of the core for a wrong state.
    try:
        receiver.start_protocol("airplay")
    except ffi.FfiError.Failed as error:
        assert error.code == 7, error.code
        assert "airplay" in error.details, error.details
    else:
        raise AssertionError("the disabled protocol must be rejected")

    try:
        receiver.start_protocol("cast")
    except ffi.FfiError.Failed as error:
        # No engine is registered in the library of this proof, so the
        # receiver must say so instead of pretending that it advertises.
        assert error.code == 6, error.code
        assert "cast" in error.details, error.details
    else:
        raise AssertionError("the protocol without an engine must be rejected")
    print("ok  a failure arrives as a structured failure through the C ABI")


def an_event_reaches_the_callback_of_the_host():
    """The callback interface of the host is called across the boundary."""
    receiver = ffi.Receiver(a_configuration())
    sink = EventSink()
    receiver.set_event_sink(sink)

    try:
        receiver.start_protocol("cast")
    except ffi.FfiError.Failed:
        pass  # the failure is the subject of the other test

    assert len(sink.events) >= 1, "the host callback was never called"
    event = sink.events[0]
    assert event.is_PROTOCOL_STATUS_CHANGED(), event
    assert event.status.protocol == ffi.FfiProtocolKind.CAST, event
    print("ok  the callback of the host is called across the C ABI")


def the_build_info_is_reported():
    """The version of the binary is reported verbatim."""
    receiver = ffi.Receiver(a_configuration())
    info = receiver.build_info()

    assert info.version != "", info
    assert info.target != "", info
    assert info.rustc_version != "", info
    print("ok  the build info is reported through the C ABI")


def main():
    the_configuration_survives_the_bridge()
    the_status_survives_the_bridge()
    the_identity_survives_the_bridge()
    the_counters_survive_the_bridge()
    an_error_arrives_as_a_structured_failure()
    an_event_reaches_the_callback_of_the_host()
    the_build_info_is_reported()
    print("the roundtrip through the C ABI succeeded")


if __name__ == "__main__":
    main()
