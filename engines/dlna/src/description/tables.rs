//! The tables of the UPnP services.
//!
//! WHY: the action list and the state variables of the three services are
//! data, not logic. Keeping them in their own module makes the renderer of
//! the documents short enough to be read at a glance.
//!
//! HOW: every item is `pub(crate)` — the tables are an implementation
//! detail of the description module and are not part of the public API.

use super::arguments::*;
use super::{SERVICE_AV_TRANSPORT, SERVICE_CONNECTION_MANAGER, SERVICE_RENDERING_CONTROL};

/// The direction of an action argument.
pub(crate) const ARG_IN: &str = "in";

/// The direction of an action argument.
pub(crate) const ARG_OUT: &str = "out";

/// One argument of an action.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ArgumentSpec {
    /// The name of the argument.
    pub(crate) name: &'static str,
    /// `in` or `out`.
    pub(crate) direction: &'static str,
    /// The state variable the argument is bound to.
    pub(crate) variable: &'static str,
}

/// One action of a service.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActionSpec {
    /// The name of the action.
    pub(crate) name: &'static str,
    /// The arguments of the action.
    pub(crate) arguments: &'static [ArgumentSpec],
}

/// One state variable of a service.
#[derive(Debug, Clone, Copy)]
pub(crate) struct VariableSpec {
    /// The name of the state variable.
    pub(crate) name: &'static str,
    /// The UPnP data type.
    pub(crate) data_type: &'static str,
    /// The allowed values, if the variable is an enumeration.
    pub(crate) allowed: &'static [&'static str],
    /// Whether the variable is reported through GENA.
    pub(crate) send_events: bool,
}

/// One service of the renderer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ServiceSpec {
    /// The name of the service.
    pub(crate) name: &'static str,
    /// The actions the service offers.
    pub(crate) actions: &'static [ActionSpec],
    /// The state variables of the service.
    pub(crate) variables: &'static [VariableSpec],
}

/// The services of the renderer.
pub(crate) const SERVICES: &[ServiceSpec] = &[
    ServiceSpec {
        name: SERVICE_CONNECTION_MANAGER,
        actions: &[
            ActionSpec {
                name: "GetProtocolInfo",
                arguments: &[ARG_PROTOCOL_INFO_SOURCE, ARG_PROTOCOL_INFO_SINK],
            },
            ActionSpec {
                name: "GetCurrentConnectionIDs",
                arguments: &[ARG_CONNECTION_IDS],
            },
        ],
        variables: &[
            VariableSpec {
                name: "A_ARG_TYPE_ProtocolInfo",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_ConnectionIDs",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
        ],
    },
    ServiceSpec {
        name: SERVICE_AV_TRANSPORT,
        actions: &[
            ActionSpec {
                name: "SetAVTransportURI",
                arguments: &[ARG_INSTANCE_ID, ARG_CURRENT_URI, ARG_CURRENT_URI_METADATA],
            },
            ActionSpec {
                name: "SetNextAVTransportURI",
                arguments: &[ARG_INSTANCE_ID, ARG_NEXT_URI, ARG_CURRENT_URI_METADATA],
            },
            ActionSpec {
                name: "GetMediaInfo",
                arguments: &[
                    ARG_INSTANCE_ID,
                    ARG_NR_TRACKS,
                    ARG_MEDIA_DURATION,
                    ARG_CURRENT_URI_OUT,
                    ARG_CURRENT_URI_METADATA_OUT,
                    ARG_NEXT_URI_OUT,
                ],
            },
            ActionSpec {
                name: "GetTransportInfo",
                arguments: &[
                    ARG_INSTANCE_ID,
                    ARG_TRANSPORT_STATE,
                    ARG_TRANSPORT_STATUS,
                    ARG_CURRENT_SPEED,
                ],
            },
            ActionSpec {
                name: "GetPositionInfo",
                arguments: &[
                    ARG_INSTANCE_ID,
                    ARG_TRACK,
                    ARG_TRACK_DURATION,
                    ARG_TRACK_METADATA,
                    ARG_TRACK_URI,
                    ARG_REL_TIME,
                    ARG_ABS_TIME,
                    ARG_REL_COUNT,
                    ARG_ABS_COUNT,
                ],
            },
            ActionSpec {
                name: "Play",
                arguments: &[ARG_INSTANCE_ID, ARG_SPEED],
            },
            ActionSpec {
                name: "Pause",
                arguments: &[ARG_INSTANCE_ID],
            },
            ActionSpec {
                name: "Stop",
                arguments: &[ARG_INSTANCE_ID],
            },
            ActionSpec {
                name: "Seek",
                arguments: &[ARG_INSTANCE_ID, ARG_UNIT, ARG_TARGET],
            },
            ActionSpec {
                name: "GetCurrentTransportActions",
                arguments: &[ARG_INSTANCE_ID, ARG_TRANSPORT_ACTIONS],
            },
            ActionSpec {
                name: "GetDeviceCapabilities",
                arguments: &[ARG_INSTANCE_ID, ARG_DEVICE_CAPABILITIES],
            },
        ],
        variables: &[
            VariableSpec {
                name: "TransportState",
                data_type: "string",
                allowed: &[
                    "STOPPED",
                    "PLAYING",
                    "PAUSED_PLAYBACK",
                    "TRANSITIONING",
                    "NO_MEDIA_PRESENT",
                ],
                send_events: true,
            },
            VariableSpec {
                name: "TransportStatus",
                data_type: "string",
                allowed: &["OK", "ERROR_OCCURRED"],
                send_events: true,
            },
            VariableSpec {
                name: "A_ARG_TYPE_InstanceID",
                data_type: "ui4",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Speed",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_CurrentURI",
                data_type: "uri",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_NextURI",
                data_type: "uri",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_CurrentURIMetaData",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_SeekMode",
                data_type: "string",
                allowed: &["TRACK_NR", "REL_TIME", "ABS_TIME"],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_SeekTarget",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_NrTracks",
                data_type: "ui4",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Duration",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Track",
                data_type: "ui4",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_RelTime",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_AbsTime",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_RelCount",
                data_type: "i4",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_AbsCount",
                data_type: "i4",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_CurrentTransportActions",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_DeviceCapabilities",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
        ],
    },
    ServiceSpec {
        name: SERVICE_RENDERING_CONTROL,
        actions: &[
            ActionSpec {
                name: "GetVolume",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL, ARG_CURRENT_VOLUME],
            },
            ActionSpec {
                name: "SetVolume",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL, ARG_DESIRED_VOLUME],
            },
            ActionSpec {
                name: "GetMute",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL, ARG_CURRENT_MUTE],
            },
            ActionSpec {
                name: "SetMute",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL, ARG_DESIRED_MUTE],
            },
            ActionSpec {
                name: "ListChannels",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL],
            },
            ActionSpec {
                name: "GetChannelInfo",
                arguments: &[ARG_INSTANCE_ID, ARG_CHANNEL, ARG_CHANNEL_INFO],
            },
        ],
        variables: &[
            VariableSpec {
                name: "Volume",
                data_type: "ui2",
                allowed: &[],
                send_events: true,
            },
            VariableSpec {
                name: "Mute",
                data_type: "boolean",
                allowed: &[],
                send_events: true,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Channel",
                data_type: "ui2",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Volume",
                data_type: "ui2",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_Mute",
                data_type: "boolean",
                allowed: &[],
                send_events: false,
            },
            VariableSpec {
                name: "A_ARG_TYPE_ChannelInfo",
                data_type: "string",
                allowed: &[],
                send_events: false,
            },
        ],
    },
];
