//! Build and version metadata.
//!
//! WHY: a bug report without a version is not reproducible. The build info is
//! compiled into the binary so it can be reported verbatim from the settings
//! screen of a shipped app.

/// The version of the workspace, taken from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the workspace.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The licence the workspace is published under.
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// The repository the workspace lives in.
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The Rust toolchain the workspace was compiled with.
pub const RUSTC_VERSION: &str = env!("PH_RUSTC_VERSION");

/// The target triple the binary was compiled for.
pub const TARGET: &str = env!("PH_TARGET");

/// The metadata of this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    /// The workspace version.
    pub version: &'static str,
    /// The workspace name.
    pub name: &'static str,
    /// The licence identifier.
    pub license: &'static str,
    /// The repository URL.
    pub repository: &'static str,
    /// The Rust compiler version.
    pub rustc_version: &'static str,
    /// The target triple.
    pub target: &'static str,
}

impl BuildInfo {
    /// The build info of this binary.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            version: VERSION,
            name: NAME,
            license: LICENSE,
            repository: REPOSITORY,
            rustc_version: RUSTC_VERSION,
            target: TARGET,
        }
    }
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}, rustc {}, {})",
            self.name, self.version, self.target, self.rustc_version, self.license
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_is_not_empty() {
        let info = BuildInfo::current();
        assert!(!info.version.is_empty());
        assert!(!info.target.is_empty());
        assert!(!info.rustc_version.is_empty());
    }

    #[test]
    fn build_info_renders_a_readable_line() {
        let text = BuildInfo::current().to_string();
        assert!(text.contains("phairplay-core"), "{text}");
    }
}
