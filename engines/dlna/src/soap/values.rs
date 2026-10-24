//! The values of the SOAP arguments.
//!
//! WHY: an argument arrives as text and has to become a number, a boolean
//! or a duration. The conversion rules are the same for every action, so
//! they live in one place — and a parser that rejects is a parser that
//! cannot be talked into a wrong value.
//!
//! HOW: every conversion returns an error instead of a default. A missing
//! volume is not "the previous volume" and an unparsable time is not zero.

use std::time::Duration;

use super::{SoapResponse, error_code, fault};

pub(super) fn collect_arguments(action: &crate::xml::XmlElement) -> Vec<(String, String)> {
    action
        .children()
        .map(|child| {
            (
                child.local_name().to_string(),
                child.text().trim().to_string(),
            )
        })
        .collect()
}

/// The value of an argument.
pub(super) fn argument(arguments: &[(String, String)], name: &str) -> Option<String> {
    arguments
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

/// Parses a seek target.
///
/// # Errors
/// Returns a SOAP fault when the unit is unknown or the target is
/// malformed.
pub(super) fn parse_seek_target(unit: &str, target: &str) -> Result<Duration, SoapResponse> {
    match unit.trim().to_ascii_uppercase().as_str() {
        "REL_TIME" | "ABS_TIME" => {
            let duration = parse_time(target).ok_or_else(|| {
                fault(
                    500,
                    "Internal Server Error",
                    error_code::INVALID_ARGS,
                    "the seek target is not a time",
                )
            })?;
            Ok(duration)
        }
        "TRACK_NR" => {
            let track = target.trim().parse::<u32>().map_err(|_| {
                fault(
                    500,
                    "Internal Server Error",
                    error_code::INVALID_ARGS,
                    "the seek target is not a track number",
                )
            })?;
            Ok(Duration::from_secs(u64::from(track)))
        }
        _ => Err(fault(
            500,
            "Internal Server Error",
            error_code::SEEK_MODE_NOT_SUPPORTED,
            "the seek mode is not supported",
        )),
    }
}

/// Parses a `H:MM:SS[.fraction]` time.
#[must_use]
pub fn parse_time(text: &str) -> Option<Duration> {
    let text = text.trim();
    let (main, fraction) = match text.split_once('.') {
        Some((main, fraction)) => (main, fraction),
        None => (text, ""),
    };
    let mut parts = main.split(':');
    let hours: u64 = parts.next()?.trim().parse().ok()?;
    let minutes: u64 = parts.next()?.trim().parse().ok()?;
    let seconds: u64 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() || minutes > 59 || seconds > 59 {
        return None;
    }
    let mut millis = 0u64;
    if !fraction.is_empty() {
        if fraction.len() > 3 || !fraction.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let padded = format!("{fraction:0<3}");
        millis = padded.parse().ok()?;
    }
    let total = hours * 3600 + minutes * 60 + seconds;
    Some(Duration::new(
        total,
        u32::try_from(millis * 1_000_000).unwrap_or(u32::MAX),
    ))
}

/// Formats a duration as `HH:MM:SS`.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}
