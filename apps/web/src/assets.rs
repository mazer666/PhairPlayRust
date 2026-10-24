//! The files of the dashboard.
//!
//! WHAT: The directory the dashboard is served from and the way it is
//! found.
//!
//! WHY: The binary serves the dashboard as well, so that the user needs
//! one program. The directory is the one of the crate during the
//! development and the one of the installation afterwards. The environment
//! variable `PHAIRPLAY_WEB_ROOT` overrides both, so that a packaged
//! dashboard can be used without rebuilding the server.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::assets::static_root;
//!
//! // The directory exists in the development; in an installation it is the
//! // one of the package.
//! let _ = static_root();
//! ```

use std::path::PathBuf;

/// The name of the environment variable that overrides the directory of the
/// dashboard.
pub const STATIC_ROOT_VARIABLE: &str = "PHAIRPLAY_WEB_ROOT";

/// The directory the dashboard is served from.
///
/// WHY: the order matters. An explicit environment variable wins, then the
/// directory of the crate — which is the one the dashboard was built in —
/// and finally the directory the process was started in.
#[must_use]
pub fn static_root() -> PathBuf {
    if let Some(directory) = std::env::var_os(STATIC_ROOT_VARIABLE) {
        return PathBuf::from(directory);
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if development.join("index.html").exists() {
        return development;
    }
    std::env::current_dir().unwrap_or(development)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_directory_of_the_dashboard_is_found() {
        // WHY: the dashboard is served from the directory of the crate
        // during the development.
        let root = static_root();
        assert!(root.is_absolute(), "{}", root.display());
    }
}
