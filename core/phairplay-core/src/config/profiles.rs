//! The device profiles of the receiver.
//!
//! WHAT: [`DeviceProfile`] is the **vocabulary** of the profiles: the five
//! names a user may write on the command line, the file each name stands
//! for and one sentence about the device the profile is for.
//!
//! WHY: The values of a profile live in the profile file — `raspberry-pi`
//! is `config/profiles/raspberry-pi.toml` — and not in the code. A decision
//! that a user can read, change and compare belongs in a file; a constant
//! in the program is invisible to the user and drifts away from the
//! documentation the moment somebody edits one of the two.
//!
//! WHY the names nevertheless live in the core: the name of a profile is a
//! word of the product, like the name of a protocol. The core owns the
//! vocabulary of the product; the configuration directory owns the values.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::profiles::DeviceProfile;
//!
//! let profile = DeviceProfile::RaspberryPi;
//! assert_eq!(profile.name(), "raspberry-pi");
//! assert_eq!(profile.file_name(), "raspberry-pi.toml");
//! ```

use std::fmt;

use crate::config::AppConfig;
use crate::error::ConfigError;
use std::path::Path;

/// The device profiles of the receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceProfile {
    /// A single-board computer such as a Raspberry Pi.
    RaspberryPi,
    /// A network attached storage device.
    Nas,
    /// A container on a host.
    Docker,
    /// A Linux desktop.
    Desktop,
    /// The neutral profile: the fallback for a device nobody described.
    Generic,
}

impl DeviceProfile {
    /// Every profile, in a stable order.
    pub const ALL: [DeviceProfile; 5] = [
        DeviceProfile::RaspberryPi,
        DeviceProfile::Nas,
        DeviceProfile::Docker,
        DeviceProfile::Desktop,
        DeviceProfile::Generic,
    ];

    /// The name of the profile on the command line.
    ///
    /// EXAMPLE:
    ///
    /// ```
    /// use phairplay_core::config::profiles::DeviceProfile;
    ///
    /// assert_eq!(DeviceProfile::RaspberryPi.name(), "raspberry-pi");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            DeviceProfile::RaspberryPi => "raspberry-pi",
            DeviceProfile::Nas => "nas",
            DeviceProfile::Docker => "docker",
            DeviceProfile::Desktop => "desktop",
            DeviceProfile::Generic => "generic",
        }
    }

    /// The name of the file that carries the profile.
    ///
    /// WHY: the file name is the name of the profile plus the extension of
    /// the configuration format. The rule is written down once, here, so
    /// that the program and the directory of the profiles cannot drift
    /// apart.
    #[must_use]
    pub const fn file_name(self) -> &'static str {
        match self {
            DeviceProfile::RaspberryPi => "raspberry-pi.toml",
            DeviceProfile::Nas => "nas.toml",
            DeviceProfile::Docker => "docker.toml",
            DeviceProfile::Desktop => "desktop.toml",
            DeviceProfile::Generic => "generic.toml",
        }
    }

    /// One sentence about the device the profile is for.
    ///
    /// WHY: the name of a profile is not self-explanatory — "nas" could be
    /// anything. The sentence is what the `--help` of the program prints.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            DeviceProfile::RaspberryPi => {
                "a single-board computer such as a Raspberry Pi: little memory, no Wi-Fi Direct"
            }
            DeviceProfile::Nas => "a network attached storage device: a server without a screen",
            DeviceProfile::Docker => {
                "a container on a host: the default budget, no radio of its own"
            }
            DeviceProfile::Desktop => "a Linux desktop: memory to spare and every protocol",
            DeviceProfile::Generic => "the defaults of the product, for a device that is unknown",
        }
    }

    /// Loads the configuration of the profile out of a directory.
    ///
    /// # Errors
    /// Returns [`ConfigError::NotFound`] when the directory does not carry
    /// the file of the profile, [`ConfigError::Parse`] when the file is not
    /// valid TOML, and any error from [`AppConfig::validate`].
    pub fn load(self, directory: impl AsRef<Path>) -> Result<AppConfig, ConfigError> {
        AppConfig::from_path(directory.as_ref().join(self.file_name()))
    }
}

impl fmt::Display for DeviceProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for DeviceProfile {
    type Err = String;

    /// Parses the name of a profile.
    ///
    /// # Errors
    /// Returns a message that names the known profiles when the name is not
    /// one of them.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let name = value.trim().to_ascii_lowercase();
        for profile in DeviceProfile::ALL {
            if profile.name() == name {
                return Ok(profile);
            }
        }
        let known = DeviceProfile::ALL
            .iter()
            .map(|profile| profile.name())
            .collect::<Vec<&str>>()
            .join(", ");
        Err(format!(
            "unknown profile `{value}` — the known profiles are: {known}"
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn every_profile_has_a_name_a_file_and_a_description() {
        for profile in DeviceProfile::ALL {
            assert!(!profile.name().is_empty());
            assert!(!profile.file_name().is_empty());
            assert!(!profile.description().is_empty());
        }
    }

    #[test]
    fn the_names_are_unique() {
        let mut names = DeviceProfile::ALL
            .iter()
            .map(|profile| profile.name())
            .collect::<Vec<&str>>();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two profiles share a name");
    }

    #[test]
    fn the_file_of_a_profile_carries_its_name() {
        // WHY: the file name is the rule of the directory of the profiles.
        // A profile whose file does not carry its name is a profile that
        // the program cannot find.
        for profile in DeviceProfile::ALL {
            assert_eq!(profile.file_name(), format!("{}.toml", profile.name()));
        }
    }

    #[test]
    fn a_profile_round_trips_through_its_name() {
        for profile in DeviceProfile::ALL {
            let parsed: DeviceProfile = profile.name().parse().expect("a known profile");
            assert_eq!(parsed, profile);
        }
    }

    #[test]
    fn an_unknown_profile_names_the_known_ones() {
        let error = "toaster".parse::<DeviceProfile>().expect_err("must fail");
        assert!(error.contains("toaster"), "{error}");
        assert!(error.contains("raspberry-pi"), "{error}");
    }

    #[test]
    fn a_profile_that_is_not_in_the_directory_is_reported() {
        // WHY: the failure has to name the file that was looked for —
        // otherwise the user does not know which directory is missing.
        let error = DeviceProfile::Nas
            .load("/definitely/not/a/directory")
            .expect_err("the file must be missing");
        assert!(error.to_string().contains("nas.toml"), "{error}");
    }
}
