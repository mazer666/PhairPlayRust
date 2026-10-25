//! The lookup of the device profile files.
//!
//! WHAT: The search for the file of a device profile and the load of the
//! configuration it carries.
//!
//! WHY: The values of a profile live in a file of the configuration
//! directory (`config/profiles/raspberry-pi.toml`), not in the program. The
//! program therefore has to *find* that file — and it has to find it in an
//! installation as well as in a checkout. The search therefore knows three
//! kinds of places: the directory the environment names, the directory of
//! the working directory (the checkout) and the directory of the
//! installation (`/etc/phairplayrust/profiles`).
//!
//! WHY the search is not a single path: a receiver that is installed on a
//! NAS and a receiver that is started from a checkout must both be able to
//! say `--profile nas`. A program that only knew one path would work in one
//! of the two cases.
//!
//! EXAMPLE:
//!
//! ```no_run
//! use phairplay_web::profiles;
//! use phairplay_core::config::profiles::DeviceProfile;
//!
//! // The configuration of a profile, or the reason why there is none.
//! let configuration = profiles::load(DeviceProfile::RaspberryPi);
//! ```

use std::path::PathBuf;

use phairplay_core::config::AppConfig;
use phairplay_core::config::profiles::DeviceProfile;

/// The name of the environment variable that names the directory of the
/// profiles.
pub const DIRECTORY_VARIABLE: &str = "PHAIRPLAY_PROFILE_DIR";

/// The directories that are searched for the profile files, in order.
///
/// WHY the order: the explicit wish of the user comes first, then the
/// working directory, then the installation. The first directory that
/// carries the file of the profile wins.
#[must_use]
pub fn directories() -> Vec<PathBuf> {
    let mut directories = Vec::new();

    if let Some(directory) = std::env::var_os(DIRECTORY_VARIABLE) {
        directories.push(PathBuf::from(directory));
    }

    if let Ok(current) = std::env::current_dir() {
        directories.push(current.join("config").join("profiles"));
        directories.push(current.join("profiles"));
    }

    if let Some(executable) = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|directory| directory.to_path_buf()))
    {
        directories.push(
            executable
                .join("..")
                .join("share")
                .join("phairplayrust")
                .join("profiles"),
        );
        directories.push(executable.join("..").join("config").join("profiles"));
        directories.push(executable.join("..").join("profiles"));
    }

    // WHY: the checkout of the repository carries the profiles next to the
    // crate. The path is compiled into the binary, so that a receiver that
    // is started from anywhere finds the profiles of its own checkout.
    directories.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("config")
            .join("profiles"),
    );

    directories.push(PathBuf::from("/etc/phairplayrust/profiles"));
    directories
}

/// The file of a profile, if one of the directories carries it.
#[must_use]
pub fn locate(profile: DeviceProfile) -> Option<PathBuf> {
    directories()
        .into_iter()
        .map(|directory| directory.join(profile.file_name()))
        .find(|path| path.is_file())
}

/// Loads the configuration of a device profile.
///
/// # Errors
/// Returns a message that names the profile, the file that was looked for
/// and the directories that were searched when there is no such file, and a
/// message that names the file and the reason when the file cannot be read.
pub fn load(profile: DeviceProfile) -> Result<AppConfig, String> {
    load_from(profile, &directories())
}

/// Loads the configuration of a device profile out of the given directories.
///
/// WHY the directories are a parameter: the search is a pure function of a
/// list of directories, and a pure function can be tested without a
/// filesystem full of profiles.
fn load_from(profile: DeviceProfile, directories: &[PathBuf]) -> Result<AppConfig, String> {
    for directory in directories {
        let path = directory.join(profile.file_name());
        if path.is_file() {
            return profile.load(directory).map_err(|error| {
                format!(
                    "the profile file `{}` could not be loaded: {error}",
                    path.display()
                )
            });
        }
    }
    let searched = directories
        .iter()
        .map(|directory| directory.display().to_string())
        .collect::<Vec<String>>()
        .join(", ");
    Err(format!(
        "the profile `{}` was not found: the file `{}` is missing in {searched}",
        profile.name(),
        profile.file_name()
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_core::protocol::ProtocolKind;

    #[test]
    fn the_search_carries_the_checkout_of_the_repository() {
        let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("config")
            .join("profiles");
        assert!(
            directories()
                .iter()
                .any(|directory| directory == &development),
            "the checkout of the repository must be searched"
        );
    }

    #[test]
    fn the_search_carries_the_directory_of_the_installation() {
        assert!(
            directories()
                .iter()
                .any(|directory| directory == &PathBuf::from("/etc/phairplayrust/profiles")),
            "the directory of the installation must be searched"
        );
    }

    #[test]
    fn every_profile_of_the_repository_is_found_and_loaded() {
        // WHY: the program promises five profiles. A profile that the
        // program does not know is a profile the user cannot use.
        for profile in DeviceProfile::ALL {
            let path = locate(profile)
                .unwrap_or_else(|| panic!("the profile {profile} is not in {:?}", directories()));
            assert!(path.is_file(), "{}", path.display());

            let configuration = load(profile)
                .unwrap_or_else(|error| panic!("the profile {profile} must load: {error}"));
            assert!(
                configuration.validate().is_ok(),
                "the profile {profile} is not a valid configuration"
            );
        }
    }

    #[test]
    fn every_profile_of_the_repository_describes_the_five_protocols() {
        // WHY: the receiver speaks five protocols. A profile that does not
        // mention one of them hands the engine of that protocol the default
        // of the core instead of the value the profile owner reviewed — and
        // a profile that mentions a protocol twice is a configuration that
        // says two different things about the same protocol.
        //
        // WHY the switch is not checked here: a profile may switch a
        // protocol off on purpose. The Raspberry Pi switches Miracast off
        // because the device has no Wi-Fi Direct radio.
        for profile in DeviceProfile::ALL {
            let configuration = load(profile)
                .unwrap_or_else(|error| panic!("the profile {profile} must load: {error}"));
            for kind in ProtocolKind::ALL {
                assert!(
                    configuration.protocols.contains_key(kind.as_str()),
                    "the profile {profile} does not describe the protocol {kind}"
                );
            }
            assert_eq!(
                configuration.protocols.len(),
                ProtocolKind::ALL.len(),
                "the profile {profile} does not describe exactly the five protocols"
            );
        }
    }

    #[test]
    fn a_profile_that_is_missing_is_reported_with_the_searched_directories() {
        // WHY: the message has to name the file and the directories —
        // otherwise the user does not know where to put the file.
        let error = load_from(DeviceProfile::Generic, &[PathBuf::from("/nowhere")])
            .expect_err("the profile must be missing");
        assert!(error.contains("generic.toml"), "{error}");
        assert!(error.contains("/nowhere"), "{error}");
        assert!(error.contains("was not found"), "{error}");
    }

    #[test]
    fn a_profile_that_cannot_be_read_is_reported_with_its_file() {
        // WHY: a profile that is broken is a file the user has to fix. The
        // message has to name the file and the reason.
        let directory =
            std::env::temp_dir().join(format!("phairplay-profile-{}-broken", std::process::id()));
        std::fs::create_dir_all(&directory).expect("the directory must be writable");
        let path = directory.join(DeviceProfile::Generic.file_name());
        std::fs::write(&path, "[device\nname = \"Broken\"").expect("the file must be writable");

        let error = load_from(DeviceProfile::Generic, std::slice::from_ref(&directory))
            .expect_err("the file must be refused");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&directory);

        assert!(error.contains("generic.toml"), "{error}");
        assert!(error.contains("could not be loaded"), "{error}");
    }
}
