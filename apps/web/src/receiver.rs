//! The configuration of the receiver.
//!
//! WHAT: the one place that turns the command line and the device profile
//! into the configuration of the receiver.
//!
//! WHY the composition root does not live here: the native shells and the
//! local dashboard reach the receiver through the bridge. The place that
//! knows which engines exist is the bridge — see
//! [`phairplay_ffi::composition`]. This module only decides *how* the
//! receiver is configured; the bridge decides *what* it consists of.
//!
//! EXAMPLE:
//!
//! ```text
//! let config = phairplay_web::receiver::configuration(&options)?;
//! let core = phairplay_ffi::composition::build(&config)?;
//! ```

use phairplay_core::config::AppConfig;

use crate::cli::Options;
use crate::profiles;

/// The composition root of the receiver, re-exported from the bridge.
///
/// WHY the re-export: the dashboard and the native shells start the receiver
/// in the same way, and a caller should not have to know which crate holds
/// the composition root.
pub use phairplay_ffi::composition::{ProtocolOutcome, Startup, build, start, stop};

/// Builds the configuration of the receiver out of the options.
///
/// WHY: the profile is the base and the configuration file is the explicit
/// wish of the user. The file is laid over the profile, so that what the
/// file names wins and what it does not mention stays as the profile
/// guessed it.
///
/// # Errors
/// Returns a message that names the file and the reason when the profile
/// or the file cannot be read, is not valid TOML or does not fit the model.
pub fn configuration(options: &Options) -> Result<AppConfig, String> {
    let base = match options.profile {
        Some(profile) => profiles::load(profile)?,
        None => AppConfig::default(),
    };
    match &options.config {
        Some(path) => {
            let table = base
                .to_table()
                .map_err(|error| format!("the configuration could not be built: {error}"))?;
            AppConfig::from_path_over(path, &table).map_err(|error| {
                format!(
                    "the configuration file `{}` could not be loaded: {error}",
                    path.display()
                )
            })
        }
        None => Ok(base),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_core::protocol::ProtocolKind;

    /// The options of a receiver without a file and without a profile.
    fn options() -> Options {
        Options::default()
    }

    #[test]
    fn the_configuration_of_a_profile_is_the_base_of_a_file() {
        // WHY: what the file names wins over what the profile guessed.
        let mut options = options();
        options.profile = Some(phairplay_core::config::profiles::DeviceProfile::RaspberryPi);
        let config = configuration(&options).expect("the configuration must be built");
        assert_eq!(config.device.name, "PhairPlay Pi");
        assert!(!config.is_enabled(ProtocolKind::Miracast));
    }

    #[test]
    fn a_file_lays_over_the_profile() {
        // WHY: the profile is the base and the file is the explicit wish of
        // the user. What the file names wins.
        let mut path = std::env::temp_dir();
        path.push(format!("phairplay-web-overlay-{}.toml", std::process::id()));
        std::fs::write(&path, "[device]\nname = \"Attic\"").expect("the file must be writable");

        let mut options = options();
        options.profile = Some(phairplay_core::config::profiles::DeviceProfile::RaspberryPi);
        options.config = Some(path.clone());
        let config = configuration(&options).expect("the configuration must be built");
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.device.name, "Attic");
        // WHY: what the file does not mention stays as the profile said it.
        assert!(!config.is_enabled(ProtocolKind::Miracast));
    }

    #[test]
    fn a_broken_configuration_file_names_the_file_and_the_reason() {
        let mut path = std::env::temp_dir();
        path.push(format!("phairplay-web-broken-{}.toml", std::process::id()));
        std::fs::write(&path, "[device\nname = \"Living Room\"")
            .expect("the file must be writable");

        let mut options = options();
        options.config = Some(path.clone());
        let error = configuration(&options).expect_err("the file must be refused");
        let _ = std::fs::remove_file(&path);

        assert!(error.contains("phairplay-web-broken"), "{error}");
        assert!(error.contains("could not be loaded"), "{error}");
    }

    #[test]
    fn a_missing_configuration_file_names_the_file() {
        let mut path = std::env::temp_dir();
        path.push(format!("phairplay-web-absent-{}.toml", std::process::id()));

        let mut options = options();
        options.config = Some(path.clone());
        let error = configuration(&options).expect_err("the file must be refused");

        assert!(error.contains("phairplay-web-absent"), "{error}");
    }
}
