//! The command line of the receiver.
//!
//! WHAT: The parser of the arguments of `phairplay-web`. It turns a list of
//! strings into a [`Command`]: start the receiver, print the help or print
//! the version.
//!
//! WHY: The command line is the first thing a user meets, and it is the
//! only part of the program that a user on a headless device sees. It
//! therefore has to answer three questions without a manual: what can I do,
//! what does this option do, and — when I mistyped something — what did I
//! get wrong. A parser that guesses is a parser that starts the receiver
//! with a configuration the user did not ask for.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::cli::{Command, parse};
//!
//! let command = parse(&["--profile".to_string(), "raspberry-pi".to_string()])
//!     .unwrap_or_else(|error| panic!("the arguments must be understood: {error}"));
//! match command {
//!     Command::Run(options) => {
//!         assert_eq!(options.profile.map(|profile| profile.name()), Some("raspberry-pi"));
//!     }
//!     other => panic!("unexpected command: {other:?}"),
//! }
//! ```

use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use phairplay_core::config::profiles::DeviceProfile;

use crate::server::DEFAULT_PORT;

/// The name of the program.
pub const PROGRAM: &str = "phairplay-web";

/// The interface the dashboard binds to when the user names none.
///
/// WHY: the loopback interface is the only one that a browser on the same
/// machine can reach. The dashboard is a local control panel and not a
/// service of the network.
pub const LOOPBACK: &str = "127.0.0.1";

/// The smallest port a user may name.
const LOWEST_PORT: u16 = 1;

/// The largest port a user may name.
const HIGHEST_PORT: u16 = 65535;

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Start the receiver with these options.
    Run(Options),
    /// Print the help and leave.
    Help,
    /// Print the version of the build and leave.
    Version,
}

/// The options of the receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The configuration file the receiver reads.
    pub config: Option<PathBuf>,
    /// The device profile the configuration is based on.
    pub profile: Option<DeviceProfile>,
    /// The port the dashboard binds to.
    pub port: u16,
    /// The interface the dashboard binds to.
    pub bind: IpAddr,
    /// Whether the files of the dashboard are served.
    pub dashboard: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            config: None,
            profile: None,
            port: DEFAULT_PORT,
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            dashboard: true,
        }
    }
}

/// A failure of the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    /// Builds a failure with the given explanation.
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// The explanation of the failure.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

/// Parses the arguments of the program.
///
/// # Errors
/// Returns a [`CliError`] with a sentence that names the mistake when an
/// option is unknown, when a value is missing, when a value has the wrong
/// shape or when a value is outside its range.
pub fn parse(arguments: &[String]) -> Result<Command, CliError> {
    let mut options = Options::default();
    let mut index = 0;

    while index < arguments.len() {
        let argument = arguments[index].clone();
        index += 1;

        let (name, value) = match argument.split_once('=') {
            Some((name, value)) => (name.to_string(), Some(value.to_string())),
            None => (argument.clone(), None),
        };

        match name.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            "--no-dashboard" => options.dashboard = false,
            "--config" => {
                options.config = Some(PathBuf::from(take_value(
                    &name, value, arguments, &mut index,
                )?));
            }
            "--profile" => {
                let raw = take_value(&name, value, arguments, &mut index)?;
                options.profile = Some(raw.parse::<DeviceProfile>().map_err(CliError::new)?);
            }
            "--port" | "-p" => {
                let raw = take_value(&name, value, arguments, &mut index)?;
                options.port = parse_port(&raw)?;
            }
            "--bind" => {
                let raw = take_value(&name, value, arguments, &mut index)?;
                options.bind = parse_address(&raw)?;
            }
            other => {
                return Err(CliError::new(format!(
                    "unknown option `{other}` — try `{PROGRAM} --help`"
                )));
            }
        }
    }

    Ok(Command::Run(options))
}

/// Takes the value of an option, either from `--option=value` or from the
/// next argument.
fn take_value(
    name: &str,
    inline: Option<String>,
    arguments: &[String],
    index: &mut usize,
) -> Result<String, CliError> {
    if let Some(value) = inline {
        return Ok(value);
    }
    match arguments.get(*index) {
        Some(value) => {
            *index += 1;
            Ok(value.clone())
        }
        None => Err(CliError::new(format!("the option `{name}` needs a value"))),
    }
}

/// Reads a port number.
///
/// WHY: the port is a number between 1 and 65535. A port of zero would
/// ask the kernel for a free port, and the user would have no way to find
/// the dashboard — so it is refused with an explanation instead of being
/// silently accepted.
fn parse_port(raw: &str) -> Result<u16, CliError> {
    let port: u32 = raw
        .parse()
        .map_err(|_| CliError::new(format!("`{raw}` is not a port number")))?;
    if port < u32::from(LOWEST_PORT) || port > u32::from(HIGHEST_PORT) {
        return Err(CliError::new(format!(
            "the port must be between {LOWEST_PORT} and {HIGHEST_PORT}"
        )));
    }
    Ok(port as u16)
}

/// Reads an IP address.
fn parse_address(raw: &str) -> Result<IpAddr, CliError> {
    raw.parse::<IpAddr>()
        .map_err(|_| CliError::new(format!("`{raw}` is not an IP address")))
}

/// The help of the program.
///
/// WHY: every option gets one sentence. A user who reads the help has to
/// know what an option does without opening a manual — the program is the
/// documentation on a device without a screen.
#[must_use]
pub fn help() -> String {
    let profiles = DeviceProfile::ALL
        .iter()
        .map(|profile| format!("    {:<14}{}", profile.name(), profile.description()))
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        "{PROGRAM} — the receiver of PhairPlayRust\n\
         \n\
         USAGE:\n    \
         {PROGRAM} [--config <path>] [--profile <name>] [--port <port>] [--bind <address>] \
         [--no-dashboard] [--version] [--help]\n\
         \n\
         OPTIONS:\n    \
         --config <path>     Reads the configuration of the receiver from a TOML file.\n    \
         --profile <name>    Uses the device profile <name> — the file config/profiles/<name>.toml — as the base of the configuration.\n    \
         --port <port>       Binds the dashboard to <port> on the chosen interface.\n    \
         --bind <address>    Binds the dashboard to <address> instead of the loopback interface.\n    \
         --no-dashboard      Runs the receiver without the dashboard; the API stays available.\n    \
         --version           Prints the version of this build and exits.\n    \
         --help              Prints this help and exits.\n\
         \n\
         THE PROFILES:\n\
         {profiles}\n\
         \n\
         The receiver starts every protocol that the configuration enables. A protocol that\n\
         cannot start — because the device has no multicast, for example — is reported as an\n\
         error and does not stop the other protocols.\n\
         \n\
         THE PROTOCOLS:\n\
         Every protocol has its own switch in the configuration, and the switch of a protocol is\n\
         the key `enabled` in its section:\n\
         \n\
         [protocols.dlna]\n\
         enabled = false\n\
         \n\
         The sections are `airplay`, `miracast`, `cast`, `dlna` and `smartview`. The receiver\n\
         starts every protocol whose switch is on, and the protocols that share a discovery\n\
         port — DLNA and Smart View — are served by ONE listener, so switching one of them off\n\
         does not take the other one with it.\n\
         \n\
         The default interface is {LOOPBACK}: the dashboard is a local control panel. A\n\
         different interface puts the control panel of the receiver into the network, and the\n\
         program says so on stderr.\n\
         \n\
         A configuration file is laid over the profile: what the file names wins."
    )
}

/// The version of the build.
///
/// WHY: a bug report without a version is not reproducible. The line names
/// the version, the target and the compiler that built the binary.
#[must_use]
pub fn version() -> String {
    let info = phairplay_core::build_info::BuildInfo::current();
    format!(
        "{PROGRAM} {} ({}, {}, {})",
        info.version, info.target, info.rustc_version, info.license
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn options(values: &[&str]) -> Options {
        match parse(&arguments(values)).expect("the arguments must be understood") {
            Command::Run(options) => options,
            other => panic!("unexpected command: {other:?}"),
        }
    }

    fn failure(values: &[&str]) -> CliError {
        match parse(&arguments(values)) {
            Ok(other) => panic!("the arguments must be refused: {other:?}"),
            Err(error) => error,
        }
    }

    #[test]
    fn without_arguments_the_defaults_are_used() {
        let options = options(&[]);
        assert_eq!(options.config, None);
        assert_eq!(options.profile, None);
        assert_eq!(options.port, DEFAULT_PORT);
        assert!(options.bind.is_loopback());
        assert!(options.dashboard);
    }

    #[test]
    fn the_default_interface_is_the_loopback_interface() {
        // WHY: the whole privacy promise of the receiver depends on this
        // one line. A dashboard that binds to 0.0.0.0 would be reachable
        // from the network of the user.
        let options = options(&[]);
        assert!(options.bind.is_loopback(), "{}", options.bind);
    }

    #[test]
    fn a_configuration_file_is_named() {
        let options = options(&["--config", "/etc/phairplayrust/receiver.toml"]);
        assert_eq!(
            options.config,
            Some(PathBuf::from("/etc/phairplayrust/receiver.toml"))
        );
    }

    #[test]
    fn a_configuration_file_is_named_with_an_equal_sign() {
        let options = options(&["--config=/etc/phairplayrust/receiver.toml"]);
        assert_eq!(
            options.config,
            Some(PathBuf::from("/etc/phairplayrust/receiver.toml"))
        );
    }

    #[test]
    fn a_profile_is_named() {
        let options = options(&["--profile", "raspberry-pi"]);
        assert_eq!(options.profile, Some(DeviceProfile::RaspberryPi));
    }

    #[test]
    fn every_profile_is_understood() {
        for profile in DeviceProfile::ALL {
            let options = options(&["--profile", profile.name()]);
            assert_eq!(options.profile, Some(profile));
        }
    }

    #[test]
    fn a_port_is_named() {
        assert_eq!(options(&["--port", "8080"]).port, 8080);
        assert_eq!(options(&["--port=8080"]).port, 8080);
        assert_eq!(options(&["-p", "8080"]).port, 8080);
    }

    #[test]
    fn an_address_is_named() {
        let options = options(&["--bind", "0.0.0.0"]);
        assert_eq!(options.bind.to_string(), "0.0.0.0");
        assert!(!options.bind.is_loopback());
    }

    #[test]
    fn the_dashboard_can_be_switched_off() {
        assert!(!options(&["--no-dashboard"]).dashboard);
    }

    #[test]
    fn the_help_is_asked_for() {
        assert_eq!(parse(&arguments(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&arguments(&["-h"])), Ok(Command::Help));
    }

    #[test]
    fn the_version_is_asked_for() {
        assert_eq!(parse(&arguments(&["--version"])), Ok(Command::Version));
        assert_eq!(parse(&arguments(&["-V"])), Ok(Command::Version));
    }

    #[test]
    fn an_unknown_option_is_refused() {
        let error = failure(&["--turbo"]);
        assert!(error.message().contains("--turbo"), "{error}");
        assert!(error.message().contains("--help"), "{error}");
    }

    #[test]
    fn a_positional_argument_is_refused() {
        // WHY: a stray word is a typo, and a receiver that starts with a
        // typo is a receiver that does something the user did not ask for.
        let error = failure(&["receiver.toml"]);
        assert!(error.message().contains("receiver.toml"), "{error}");
    }

    #[test]
    fn a_missing_value_is_refused() {
        for option in ["--config", "--profile", "--port", "--bind"] {
            let error = failure(&[option]);
            assert!(
                error.message().contains("needs a value"),
                "{option}: {error}"
            );
        }
    }

    #[test]
    fn a_port_out_of_the_range_is_refused() {
        // WHY: a port of 65536 does not exist, and a port of zero asks the
        // kernel for a free port that the user cannot find.
        let error = failure(&["--port", "65536"]);
        assert!(error.message().contains("between 1 and 65535"), "{error}");
        let error = failure(&["--port", "0"]);
        assert!(error.message().contains("between 1 and 65535"), "{error}");
    }

    #[test]
    fn a_port_that_is_not_a_number_is_refused() {
        let error = failure(&["--port", "http"]);
        assert!(error.message().contains("not a port number"), "{error}");
    }

    #[test]
    fn an_address_that_is_not_an_address_is_refused() {
        let error = failure(&["--bind", "localhost"]);
        assert!(error.message().contains("not an IP address"), "{error}");
    }

    #[test]
    fn an_unknown_profile_is_refused() {
        let error = failure(&["--profile", "toaster"]);
        assert!(error.message().contains("toaster"), "{error}");
        assert!(error.message().contains("raspberry-pi"), "{error}");
    }

    #[test]
    fn the_help_names_every_option() {
        // WHY: the help is the only documentation a user on a headless
        // device has. Every option of the program must be in it.
        let help = help();
        for option in [
            "--config",
            "--profile",
            "--port",
            "--bind",
            "--no-dashboard",
            "--version",
            "--help",
        ] {
            assert!(help.contains(option), "the help does not name {option}");
        }
    }

    #[test]
    fn the_help_names_every_profile() {
        let help = help();
        for profile in DeviceProfile::ALL {
            assert!(
                help.contains(profile.name()),
                "the help does not name {profile}"
            );
        }
    }

    #[test]
    fn the_version_names_the_build() {
        let version = version();
        assert!(version.contains(PROGRAM), "{version}");
        assert!(
            version.contains(phairplay_core::build_info::VERSION),
            "{version}"
        );
    }
}
