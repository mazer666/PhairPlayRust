//! The DACP reverse control channel.
//!
//! WHY: the remote control of the TV has to drive the *sender*: play, pause,
//! skip, volume. The AirPlay sender offers this through DACP (Digital Audio
//! Control Protocol): it announces a `_dacp._tcp` service and hands the
//! receiver two RTSP headers — `DACP-ID` and `Active-Remote`. The receiver
//! then sends `GET /ctrl-int/1/<command>` to that service, with the
//! `Active-Remote` token as the authorisation.
//!
//! The discovery of the `_dacp._tcp` service belongs to the discovery engine;
//! this module owns the *protocol* side: the command vocabulary, the request
//! and the answer.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::dacp::{DacpClient, DacpCommand};
//!
//! let mut client = DacpClient::new();
//! client.configure(Some("0A1B2C3D"), Some("token"));
//! assert!(client.is_available());
//! assert_eq!(DacpCommand::PlayPause.path(), "/ctrl-int/1/playpause");
//! ```

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::AirPlayError;

/// The service type of the DACP control channel.
pub const DACP_SERVICE_TYPE: &str = "_dacp._tcp";

/// The prefix of the service name of a sender.
pub const DACP_SERVICE_PREFIX: &str = "iTunes_Ctrl_";

/// The timeout of a DACP request.
pub const DACP_TIMEOUT: Duration = Duration::from_secs(2);

/// The commands of the DACP protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacpCommand {
    /// Toggle between play and pause.
    PlayPause,
    /// Skip to the next item.
    NextItem,
    /// Skip to the previous item.
    PreviousItem,
    /// Start the fast-forward.
    BeginFastForward,
    /// Start the rewind.
    BeginRewind,
    /// Raise the volume.
    VolumeUp,
    /// Lower the volume.
    VolumeDown,
    /// Toggle the mute.
    MuteToggle,
}

impl DacpCommand {
    /// The wire name of the command.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlayPause => "playpause",
            Self::NextItem => "nextitem",
            Self::PreviousItem => "previtem",
            Self::BeginFastForward => "beginff",
            Self::BeginRewind => "beginrew",
            Self::VolumeUp => "volumeup",
            Self::VolumeDown => "volumedown",
            Self::MuteToggle => "mutetoggle",
        }
    }

    /// The path of the command on the control service.
    #[must_use]
    pub fn path(self) -> String {
        format!("/ctrl-int/1/{}", self.as_str())
    }
}

/// The identity of the sending device for the reverse channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DacpIdentity {
    /// The identifier of the DACP service.
    pub dacp_id: String,
    /// The token that authorises the commands.
    pub active_remote: String,
}

impl DacpIdentity {
    /// The name of the DACP service of the sender.
    #[must_use]
    pub fn service_name(&self) -> String {
        format!("{DACP_SERVICE_PREFIX}{}", self.dacp_id)
    }
}

/// The client of the DACP reverse channel.
#[derive(Debug, Default)]
pub struct DacpClient {
    identity: Option<DacpIdentity>,
    endpoint: Option<(String, u16)>,
}

impl DacpClient {
    /// Creates a client without an identity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores the identity of the sender.
    ///
    /// Returns `true` when the identity changed. Repeated calls with the
    /// same identity are cheap and do not reset the resolved endpoint.
    pub fn configure(&mut self, dacp_id: Option<&str>, active_remote: Option<&str>) -> bool {
        let (Some(dacp_id), Some(active_remote)) = (dacp_id, active_remote) else {
            return false;
        };
        if dacp_id.trim().is_empty() || active_remote.trim().is_empty() {
            return false;
        }
        let identity = DacpIdentity {
            dacp_id: dacp_id.trim().to_string(),
            active_remote: active_remote.trim().to_string(),
        };
        if self.identity.as_ref() == Some(&identity) {
            return false;
        }
        self.endpoint = None;
        self.identity = Some(identity);
        true
    }

    /// Sets the resolved address of the DACP service.
    pub fn set_endpoint(&mut self, host: &str, port: u16) {
        self.endpoint = Some((host.to_string(), port));
    }

    /// Whether the client has an identity.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.identity.is_some()
    }

    /// The identity of the sender.
    #[must_use]
    pub fn identity(&self) -> Option<&DacpIdentity> {
        self.identity.as_ref()
    }

    /// The resolved endpoint of the DACP service.
    #[must_use]
    pub fn endpoint(&self) -> Option<(&str, u16)> {
        self.endpoint
            .as_ref()
            .map(|(host, port)| (host.as_str(), *port))
    }

    /// Builds the HTTP request of a command.
    ///
    /// # Errors
    /// Returns [`AirPlayError::InvalidState`] when the client has no
    /// identity.
    pub fn build_request(&self, command: DacpCommand) -> Result<Vec<u8>, AirPlayError> {
        let identity = self.identity.as_ref().ok_or_else(|| {
            AirPlayError::invalid_state("the DACP client has no identity of the sender")
        })?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost\r\nActive-Remote: {}\r\nUser-Agent: AirTunes/220.68\r\n\r\n",
            command.path(),
            identity.active_remote
        );
        Ok(request.into_bytes())
    }

    /// Sends a command to the sender.
    ///
    /// # Errors
    /// Returns [`AirPlayError::InvalidState`] when the identity or the
    /// endpoint is missing and [`AirPlayError::Io`] when the request fails.
    pub async fn send_command(&self, command: DacpCommand) -> Result<u16, AirPlayError> {
        let request = self.build_request(command)?;
        let (host, port) = self.endpoint().ok_or_else(|| {
            AirPlayError::invalid_state("the DACP service of the sender is not resolved yet")
        })?;
        let mut stream = tokio::time::timeout(DACP_TIMEOUT, TcpStream::connect((host, port)))
            .await
            .map_err(|_| AirPlayError::timeout("the DACP service did not answer"))??;
        stream.write_all(&request).await?;
        let mut response = Vec::new();
        let mut chunk = [0u8; 512];
        let read = tokio::time::timeout(DACP_TIMEOUT, stream.read(&mut chunk))
            .await
            .map_err(|_| AirPlayError::timeout("the DACP service did not answer"))??;
        response.extend_from_slice(&chunk[..read]);
        let text = String::from_utf8_lossy(&response);
        Ok(parse_status(&text))
    }
}

/// Extracts the status code of an HTTP answer.
#[must_use]
pub fn parse_status(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
