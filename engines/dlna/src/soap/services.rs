//! The actions of the three UPnP services.
//!
//! WHY: the control plane of a renderer is a state machine per service.
//! Keeping the three of them in their own module makes it obvious that
//! `AVTransport` is the service that owns the transport, and that
//! `RenderingControl` only owns the volume and the mute state.
//!
//! HOW: the handlers are methods of the dispatcher; they are `pub(super)`
//! so that the dispatcher in the parent module can call them.

use std::net::SocketAddr;

use super::values::{argument, format_duration, parse_seek_target};
use super::{
    SINK_PROTOCOL_INFO, SoapDispatcher, SoapResponse, error_code, fault, invalid_action, ok,
    session_rejected,
};
use crate::description::{
    EVENT_PATH_AV_TRANSPORT, EVENT_PATH_RENDERING_CONTROL, SERVICE_AV_TRANSPORT,
    SERVICE_CONNECTION_MANAGER, SERVICE_RENDERING_CONTROL,
};
use crate::state::TransportState;

impl SoapDispatcher {
    pub(super) fn connection_manager(&self, action: &str) -> SoapResponse {
        match action {
            "GetProtocolInfo" => ok(
                action,
                SERVICE_CONNECTION_MANAGER,
                &[("Source", ""), ("Sink", SINK_PROTOCOL_INFO)],
            ),
            "GetCurrentConnectionIDs" => {
                let ids = match self.state.session() {
                    Some(session) => session.id.to_string(),
                    None => "0".to_string(),
                };
                ok(
                    action,
                    SERVICE_CONNECTION_MANAGER,
                    &[("ConnectionIDs", &ids)],
                )
            }
            _ => invalid_action(action),
        }
    }

    /// The AVTransport service.
    pub(super) fn av_transport(
        &self,
        action: &str,
        arguments: &[(String, String)],
        peer: SocketAddr,
    ) -> SoapResponse {
        match action {
            "SetAVTransportURI" => {
                let uri = argument(arguments, "CurrentURI").unwrap_or_default();
                if uri.is_empty() {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "CurrentURI is empty",
                    );
                }
                if let Err(error) = self.state.claim_session(peer) {
                    return session_rejected(&error);
                }
                let metadata = argument(arguments, "CurrentURIMetaData").unwrap_or_default();
                self.state.set_media(&uri, &metadata);
                self.publish_transport_state();
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "SetNextAVTransportURI" => {
                let uri = argument(arguments, "NextURI").unwrap_or_default();
                if uri.is_empty() {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "NextURI is empty",
                    );
                }
                let metadata = argument(arguments, "CurrentURIMetaData").unwrap_or_default();
                self.state.set_next_media(&uri, &metadata);
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "GetMediaInfo" => {
                let uri = self.state.current_uri();
                let tracks = if uri.is_empty() { "0" } else { "1" };
                ok(
                    action,
                    SERVICE_AV_TRANSPORT,
                    &[
                        ("NrTracks", tracks),
                        ("MediaDuration", &format_duration(self.state.duration())),
                        ("CurrentURI", &uri),
                        ("CurrentURIMetaData", &self.state.current_uri_metadata()),
                        ("NextURI", &self.state.next_uri()),
                    ],
                )
            }
            "GetTransportInfo" => ok(
                action,
                SERVICE_AV_TRANSPORT,
                &[
                    (
                        "CurrentTransportState",
                        self.state.transport_state().as_str(),
                    ),
                    ("CurrentTransportStatus", "OK"),
                    ("CurrentSpeed", "1"),
                ],
            ),
            "GetPositionInfo" => {
                let position = format_duration(self.state.position());
                ok(
                    action,
                    SERVICE_AV_TRANSPORT,
                    &[
                        ("Track", "1"),
                        ("TrackDuration", &format_duration(self.state.duration())),
                        ("TrackMetaData", &self.state.current_uri_metadata()),
                        ("TrackURI", &self.state.current_uri()),
                        ("RelTime", &position),
                        ("AbsTime", &position),
                        ("RelCount", "0"),
                        ("AbsCount", "0"),
                    ],
                )
            }
            "Play" => {
                if self.state.current_uri().is_empty() {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::TRANSITION_NOT_AVAILABLE,
                        "no media has been set",
                    );
                }
                if let Err(error) = self.state.claim_session(peer) {
                    return session_rejected(&error);
                }
                self.state.set_transport_state(TransportState::Playing);
                self.publish_transport_state();
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "Pause" => {
                if !self.state.transport_state().is_active() {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::TRANSITION_NOT_AVAILABLE,
                        "the renderer is not playing",
                    );
                }
                self.state
                    .set_transport_state(TransportState::PausedPlayback);
                self.publish_transport_state();
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "Stop" => {
                self.state.stop();
                self.publish_transport_state();
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "Seek" => {
                let unit = argument(arguments, "Unit").unwrap_or_default();
                let target = argument(arguments, "Target").unwrap_or_default();
                let position = match parse_seek_target(&unit, &target) {
                    Ok(position) => position,
                    Err(error) => return error,
                };
                self.state.set_position(position);
                ok(action, SERVICE_AV_TRANSPORT, &[])
            }
            "GetCurrentTransportActions" => {
                let actions = match self.state.transport_state() {
                    TransportState::Playing => "Play,Stop,Pause,Seek",
                    TransportState::PausedPlayback => "Play,Stop,Pause,Seek",
                    _ => "Play,Stop",
                };
                ok(action, SERVICE_AV_TRANSPORT, &[("Actions", actions)])
            }
            "GetDeviceCapabilities" => ok(
                action,
                SERVICE_AV_TRANSPORT,
                &[("DeviceCapabilities", "PlayMedia,Seek,Play,Stop,Pause")],
            ),
            _ => invalid_action(action),
        }
    }

    /// The RenderingControl service.
    pub(super) fn rendering_control(
        &self,
        action: &str,
        arguments: &[(String, String)],
    ) -> SoapResponse {
        match action {
            "GetVolume" => {
                let volume = self.state.volume().to_string();
                ok(
                    action,
                    SERVICE_RENDERING_CONTROL,
                    &[("CurrentVolume", &volume)],
                )
            }
            "SetVolume" => {
                let Some(raw) = argument(arguments, "DesiredVolume") else {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "DesiredVolume is missing",
                    );
                };
                let Ok(value) = raw.trim().parse::<i32>() else {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "DesiredVolume is not a number",
                    );
                };
                if !(0..=100).contains(&value) {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "DesiredVolume is out of range",
                    );
                }
                let value = value as u8;
                self.state.set_volume(value);
                let volume = self.state.volume().to_string();
                self.events.publish(
                    EVENT_PATH_RENDERING_CONTROL,
                    &format!(
                        "<Event xmlns=\"urn:schemas-upnp-org:metadata-1-0/RCS/\"><InstanceID val=\"0\">\
                         <Volume channel=\"Master\" val=\"{volume}\"/></InstanceID></Event>"
                    ),
                );
                ok(action, SERVICE_RENDERING_CONTROL, &[])
            }
            "GetMute" => {
                let mute = if self.state.muted() { "1" } else { "0" };
                ok(action, SERVICE_RENDERING_CONTROL, &[("CurrentMute", mute)])
            }
            "SetMute" => {
                let Some(raw) = argument(arguments, "DesiredMute") else {
                    return fault(
                        500,
                        "Internal Server Error",
                        error_code::INVALID_ARGS,
                        "DesiredMute is missing",
                    );
                };
                let muted = match raw.trim() {
                    "1" | "true" | "TRUE" | "True" => true,
                    "0" | "false" | "FALSE" | "False" => false,
                    _ => {
                        return fault(
                            500,
                            "Internal Server Error",
                            error_code::INVALID_ARGS,
                            "DesiredMute is not a boolean",
                        );
                    }
                };
                self.state.set_muted(muted);
                let mute = if self.state.muted() { "1" } else { "0" };
                self.events.publish(
                    EVENT_PATH_RENDERING_CONTROL,
                    &format!(
                        "<Event xmlns=\"urn:schemas-upnp-org:metadata-1-0/RCS/\"><InstanceID val=\"0\">\
                         <Mute channel=\"Master\" val=\"{mute}\"/></InstanceID></Event>"
                    ),
                );
                ok(action, SERVICE_RENDERING_CONTROL, &[])
            }
            "ListChannels" => ok(
                action,
                SERVICE_RENDERING_CONTROL,
                &[("Channel", "0=Master")],
            ),
            "GetChannelInfo" => ok(action, SERVICE_RENDERING_CONTROL, &[("Info", "0=Master")]),
            _ => invalid_action(action),
        }
    }

    /// Publishes the transport state to the event subscribers.
    pub(super) fn publish_transport_state(&self) {
        let state = self.state.transport_state().as_str();
        self.events.publish(
            EVENT_PATH_AV_TRANSPORT,
            &format!(
                "<Event xmlns=\"urn:schemas-upnp-org:metadata-1-0/AVT/\"><InstanceID val=\"0\">\
                 <TransportState val=\"{state}\"/></InstanceID></Event>"
            ),
        );
    }
}
