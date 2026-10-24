//! The arguments of the UPnP actions.
//!
//! WHY: an action argument is bound to a state variable, and the binding
//! is the same for every service that uses the argument. The constants
//! therefore live in one place, and the tables refer to them by name.
//!
//! HOW: every constant is `pub(crate)` — the arguments are an
//! implementation detail of the description module.

use super::tables::{ARG_IN, ARG_OUT, ArgumentSpec};

pub(crate) const ARG_INSTANCE_ID: ArgumentSpec = ArgumentSpec {
    name: "InstanceID",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_InstanceID",
};
pub(crate) const ARG_SPEED: ArgumentSpec = ArgumentSpec {
    name: "Speed",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_Speed",
};
pub(crate) const ARG_CURRENT_URI: ArgumentSpec = ArgumentSpec {
    name: "CurrentURI",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_CurrentURI",
};
pub(crate) const ARG_CURRENT_URI_METADATA: ArgumentSpec = ArgumentSpec {
    name: "CurrentURIMetaData",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_CurrentURIMetaData",
};
pub(crate) const ARG_NEXT_URI: ArgumentSpec = ArgumentSpec {
    name: "NextURI",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_NextURI",
};
pub(crate) const ARG_UNIT: ArgumentSpec = ArgumentSpec {
    name: "Unit",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_SeekMode",
};
pub(crate) const ARG_TARGET: ArgumentSpec = ArgumentSpec {
    name: "Target",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_SeekTarget",
};
pub(crate) const ARG_DESIRED_VOLUME: ArgumentSpec = ArgumentSpec {
    name: "DesiredVolume",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_Volume",
};
pub(crate) const ARG_DESIRED_MUTE: ArgumentSpec = ArgumentSpec {
    name: "DesiredMute",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_Mute",
};
pub(crate) const ARG_CHANNEL: ArgumentSpec = ArgumentSpec {
    name: "Channel",
    direction: ARG_IN,
    variable: "A_ARG_TYPE_Channel",
};
pub(crate) const ARG_PROTOCOL_INFO_SOURCE: ArgumentSpec = ArgumentSpec {
    name: "Source",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_ProtocolInfo",
};
pub(crate) const ARG_PROTOCOL_INFO_SINK: ArgumentSpec = ArgumentSpec {
    name: "Sink",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_ProtocolInfo",
};
pub(crate) const ARG_CONNECTION_IDS: ArgumentSpec = ArgumentSpec {
    name: "ConnectionIDs",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_ConnectionIDs",
};
pub(crate) const ARG_TRANSPORT_STATE: ArgumentSpec = ArgumentSpec {
    name: "CurrentTransportState",
    direction: ARG_OUT,
    variable: "TransportState",
};
pub(crate) const ARG_TRANSPORT_STATUS: ArgumentSpec = ArgumentSpec {
    name: "CurrentTransportStatus",
    direction: ARG_OUT,
    variable: "TransportStatus",
};
pub(crate) const ARG_CURRENT_SPEED: ArgumentSpec = ArgumentSpec {
    name: "CurrentSpeed",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_Speed",
};
pub(crate) const ARG_TRACK: ArgumentSpec = ArgumentSpec {
    name: "Track",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_Track",
};
pub(crate) const ARG_TRACK_DURATION: ArgumentSpec = ArgumentSpec {
    name: "TrackDuration",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_Duration",
};
pub(crate) const ARG_TRACK_METADATA: ArgumentSpec = ArgumentSpec {
    name: "TrackMetaData",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_CurrentURIMetaData",
};
pub(crate) const ARG_TRACK_URI: ArgumentSpec = ArgumentSpec {
    name: "TrackURI",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_CurrentURI",
};
pub(crate) const ARG_REL_TIME: ArgumentSpec = ArgumentSpec {
    name: "RelTime",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_RelTime",
};
pub(crate) const ARG_ABS_TIME: ArgumentSpec = ArgumentSpec {
    name: "AbsTime",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_AbsTime",
};
pub(crate) const ARG_REL_COUNT: ArgumentSpec = ArgumentSpec {
    name: "RelCount",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_RelCount",
};
pub(crate) const ARG_ABS_COUNT: ArgumentSpec = ArgumentSpec {
    name: "AbsCount",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_AbsCount",
};
pub(crate) const ARG_NR_TRACKS: ArgumentSpec = ArgumentSpec {
    name: "NrTracks",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_NrTracks",
};
pub(crate) const ARG_MEDIA_DURATION: ArgumentSpec = ArgumentSpec {
    name: "MediaDuration",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_Duration",
};
pub(crate) const ARG_CURRENT_URI_OUT: ArgumentSpec = ArgumentSpec {
    name: "CurrentURI",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_CurrentURI",
};
pub(crate) const ARG_CURRENT_URI_METADATA_OUT: ArgumentSpec = ArgumentSpec {
    name: "CurrentURIMetaData",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_CurrentURIMetaData",
};
pub(crate) const ARG_NEXT_URI_OUT: ArgumentSpec = ArgumentSpec {
    name: "NextURI",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_NextURI",
};
pub(crate) const ARG_CURRENT_VOLUME: ArgumentSpec = ArgumentSpec {
    name: "CurrentVolume",
    direction: ARG_OUT,
    variable: "Volume",
};
pub(crate) const ARG_CURRENT_MUTE: ArgumentSpec = ArgumentSpec {
    name: "CurrentMute",
    direction: ARG_OUT,
    variable: "Mute",
};
pub(crate) const ARG_TRANSPORT_ACTIONS: ArgumentSpec = ArgumentSpec {
    name: "Actions",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_CurrentTransportActions",
};
pub(crate) const ARG_DEVICE_CAPABILITIES: ArgumentSpec = ArgumentSpec {
    name: "DeviceCapabilities",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_DeviceCapabilities",
};
pub(crate) const ARG_CHANNEL_INFO: ArgumentSpec = ArgumentSpec {
    name: "Info",
    direction: ARG_OUT,
    variable: "A_ARG_TYPE_ChannelInfo",
};
