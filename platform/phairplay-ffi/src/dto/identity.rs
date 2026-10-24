//! The identity of the receiver and the identification of the build.
//!
//! WHAT: The name, the model and the identifier the receiver advertises to
//! senders, and the version of the binary that carries the bridge.
//!
//! WHY: Senders identify the receiver by its name; a name that changes on
//! every start makes the device churn through the remembered-device list of
//! the sender. And a bug report without a version is not reproducible — the
//! build information is compiled into the binary so that it can be reported
//! verbatim from the settings screen of a shipped app.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiDeviceIdentity;
//! use phairplay_core::identity::DeviceIdentity;
//!
//! let identity = FfiDeviceIdentity::from(&DeviceIdentity::with_name("Living Room"));
//! assert_eq!(identity.name, "Living Room");
//! assert_eq!(identity.service_instance_name, "Living Room");
//! ```

use phairplay_core::build_info::BuildInfo;
use phairplay_core::identity::DeviceIdentity;

/// The identity the receiver advertises to senders.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiDeviceIdentity {
    /// The user-visible device name, for example "Living Room".
    pub name: String,
    /// The hardware model, for example "PhairPlay Rust Receiver".
    pub model: String,
    /// The manufacturer string reported to senders.
    pub manufacturer: String,
    /// The protocol version reported to senders.
    pub protocol_version: String,
    /// The stable, opaque identifier of this installation.
    pub device_id: String,
    /// The sanitised name under which the service is advertised.
    pub service_instance_name: String,
}

impl From<&DeviceIdentity> for FfiDeviceIdentity {
    fn from(identity: &DeviceIdentity) -> Self {
        FfiDeviceIdentity {
            name: identity.name.clone(),
            model: identity.model.clone(),
            manufacturer: identity.manufacturer.clone(),
            protocol_version: identity.protocol_version.clone(),
            device_id: identity.device_id.clone(),
            service_instance_name: identity.service_instance_name(),
        }
    }
}

impl From<DeviceIdentity> for FfiDeviceIdentity {
    fn from(identity: DeviceIdentity) -> Self {
        FfiDeviceIdentity::from(&identity)
    }
}

/// The identification of the build that carries the bridge.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiBuildInfo {
    /// The version of the workspace.
    pub version: String,
    /// The name of the workspace.
    pub name: String,
    /// The licence identifier of the product.
    pub license: String,
    /// The repository the source lives in.
    pub repository: String,
    /// The Rust compiler version the binary was compiled with.
    pub rustc_version: String,
    /// The target triple the binary was compiled for.
    pub target: String,
}

impl From<&BuildInfo> for FfiBuildInfo {
    fn from(info: &BuildInfo) -> Self {
        FfiBuildInfo {
            version: info.version.to_string(),
            name: info.name.to_string(),
            license: info.license.to_string(),
            repository: info.repository.to_string(),
            rustc_version: info.rustc_version.to_string(),
            target: info.target.to_string(),
        }
    }
}

impl From<BuildInfo> for FfiBuildInfo {
    fn from(info: BuildInfo) -> Self {
        FfiBuildInfo::from(&info)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_identity_survives_the_conversion() {
        let identity = DeviceIdentity::with_name("Living Room");
        let converted = FfiDeviceIdentity::from(&identity);
        assert_eq!(converted.name, identity.name);
        assert_eq!(converted.model, identity.model);
        assert_eq!(converted.manufacturer, identity.manufacturer);
        assert_eq!(converted.protocol_version, identity.protocol_version);
        assert_eq!(converted.device_id, identity.device_id);
        assert_eq!(converted.service_instance_name, "Living Room");
    }

    #[test]
    fn the_service_name_is_sanitised() {
        // A control character in the name would make the mDNS daemon
        // reject the registration; the bridge reports the sanitised name.
        let identity = DeviceIdentity::with_name("Kitchen\u{0} TV");
        let converted = FfiDeviceIdentity::from(&identity);
        assert_eq!(converted.service_instance_name, "Kitchen TV");
    }

    #[test]
    fn the_build_info_is_not_empty() {
        let converted = FfiBuildInfo::from(&BuildInfo::current());
        assert!(!converted.version.is_empty());
        assert!(!converted.target.is_empty());
        assert!(!converted.rustc_version.is_empty());
    }
}
