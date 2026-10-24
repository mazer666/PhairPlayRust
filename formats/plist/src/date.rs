// WHAT: conversion between the Apple epoch and RFC 3339 timestamps.
// WHY: the binary property-list format stores dates as a `f64` number of
//      seconds since 2001-01-01 UTC, while the XML format stores them as
//      an ISO 8601 string. The conversion must be exact for the dates a
//      receiver actually handles and must not depend on a calendar crate,
//      because the format crates depend on nothing.
// EXAMPLE: `apple_epoch_to_rfc3339(0.0)` is `"2001-01-01T00:00:00Z"`.

/// The number of seconds between the Unix epoch (1970-01-01) and the
/// Apple epoch (2001-01-01).
pub const APPLE_EPOCH_OFFSET: i64 = 978_307_200;

/// Converts seconds since the Apple epoch into an RFC 3339 timestamp.
///
/// The fractional part is rendered with up to nine digits and without a
/// trailing zero run, so that whole seconds print as `...:00Z`.
///
/// # Errors
///
/// Returns `None` when the value is not finite or lies outside the range
/// that the proleptic Gregorian calendar can represent.
pub fn apple_epoch_to_rfc3339(seconds: f64) -> Option<String> {
    if !seconds.is_finite() {
        return None;
    }
    let total = (seconds * 1_000_000_000.0).round();
    if total.abs() > 9.0e18 {
        return None;
    }
    let total_nanos = total as i64;
    let unix_seconds = total_nanos.div_euclid(1_000_000_000) + APPLE_EPOCH_OFFSET;
    let nanos = total_nanos.rem_euclid(1_000_000_000) as u32;

    let days = unix_seconds.div_euclid(86_400);
    let secs_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    let mut text = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    if nanos != 0 {
        let mut fraction = format!("{nanos:09}");
        while fraction.ends_with('0') {
            fraction.pop();
        }
        text.push('.');
        text.push_str(&fraction);
    }
    text.push('Z');
    Some(text)
}

/// Parses an RFC 3339 timestamp into seconds since the Apple epoch.
///
/// # Errors
///
/// Returns `None` when the text is not a timestamp in the shape
/// `YYYY-MM-DDThh:mm:ss[.fraction][Z|±hh:mm]`.
pub fn rfc3339_to_apple_epoch(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let year: i64 = text.get(0..4)?.parse().ok()?;
    if bytes[4] != b'-' {
        return None;
    }
    let month: u32 = text.get(5..7)?.parse().ok()?;
    if bytes[7] != b'-' {
        return None;
    }
    let day: u32 = text.get(8..10)?.parse().ok()?;
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    let hour: i64 = text.get(11..13)?.parse().ok()?;
    if bytes[13] != b':' {
        return None;
    }
    let minute: i64 = text.get(14..16)?.parse().ok()?;
    if bytes[16] != b':' {
        return None;
    }
    let second: i64 = text.get(17..19)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let mut index = 19;
    let mut fraction = 0.0_f64;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == start {
            return None;
        }
        let digits = &text[start..index];
        let mut scaled = String::with_capacity(digits.len() + 3);
        scaled.push_str(digits);
        while scaled.len() < 3 {
            scaled.push('0');
        }
        fraction = scaled[..3].parse::<f64>().ok()? / 1000.0;
    }

    let offset_seconds = match bytes.get(index) {
        Some(b'Z') | Some(b'z') if index + 1 == bytes.len() => 0,
        Some(sign @ (b'+' | b'-')) => {
            if index + 6 != bytes.len() {
                return None;
            }
            let oh: i64 = text.get(index + 1..index + 3)?.parse().ok()?;
            if bytes[index + 3] != b':' {
                return None;
            }
            let om: i64 = text.get(index + 4..index + 6)?.parse().ok()?;
            if oh > 23 || om > 59 {
                return None;
            }
            let magnitude = oh * 3600 + om * 60;
            if *sign == b'-' { -magnitude } else { magnitude }
        }
        _ => return None,
    };

    let days = days_from_civil(year, month, day);
    let unix_seconds = days * 86_400 + hour * 3600 + minute * 60 + second - offset_seconds;
    Some(unix_seconds as f64 + fraction - APPLE_EPOCH_OFFSET as f64)
}

/// Converts a day number since 1970-01-01 into a civil date.
///
/// This is the well known `civil_from_days` algorithm; it is exact for
/// the whole range of dates that a 64-bit second counter can reach.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Converts a civil date into a day number since 1970-01-01.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = month as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_epoch_is_2001() {
        assert_eq!(
            apple_epoch_to_rfc3339(0.0).as_deref(),
            Some("2001-01-01T00:00:00Z")
        );
    }

    #[test]
    fn unix_epoch_is_before_the_apple_epoch() {
        assert_eq!(
            apple_epoch_to_rfc3339(-(APPLE_EPOCH_OFFSET as f64)).as_deref(),
            Some("1970-01-01T00:00:00Z")
        );
    }

    #[test]
    fn fractional_seconds_are_rendered_without_padding_noise() {
        assert_eq!(
            apple_epoch_to_rfc3339(1.5).as_deref(),
            Some("2001-01-01T00:00:01.5Z")
        );
        assert_eq!(
            apple_epoch_to_rfc3339(1.25).as_deref(),
            Some("2001-01-01T00:00:01.25Z")
        );
    }

    #[test]
    fn round_trip_through_rfc3339_is_stable() {
        for seconds in [0.0_f64, 1.0, -1.0, 1_000_000.0, -1_000_000.0, 86_400.5] {
            let text = apple_epoch_to_rfc3339(seconds).unwrap_or_else(|| panic!("not renderable"));
            let back = rfc3339_to_apple_epoch(&text).unwrap_or_else(|| panic!("not renderable"));
            assert!(
                (back - seconds).abs() < 1e-6,
                "{seconds} -> {text} -> {back}"
            );
        }
    }

    #[test]
    fn leap_day_is_handled() {
        // 2001-01-01 plus 8070 days is 2023-02-05.
        let seconds = 8_070.0_f64 * 86_400.0;
        let text = apple_epoch_to_rfc3339(seconds).unwrap_or_else(|| panic!("not renderable"));
        assert!(text.starts_with("2023-02-05"), "{text}");
    }

    #[test]
    fn non_finite_values_are_rejected() {
        assert!(apple_epoch_to_rfc3339(f64::NAN).is_none());
        assert!(apple_epoch_to_rfc3339(f64::INFINITY).is_none());
    }

    #[test]
    fn malformed_timestamps_are_rejected() {
        assert!(rfc3339_to_apple_epoch("not a date").is_none());
        assert!(rfc3339_to_apple_epoch("2001-01-01").is_none());
        assert!(rfc3339_to_apple_epoch("2001-13-01T00:00:00Z").is_none());
        assert!(rfc3339_to_apple_epoch("2001-01-01T00:00:00").is_none());
        assert!(rfc3339_to_apple_epoch("2001-01-01T00:00:00.Z").is_none());
    }

    #[test]
    fn timezone_offsets_are_applied() {
        let utc = rfc3339_to_apple_epoch("2001-01-01T00:00:00Z")
            .unwrap_or_else(|| panic!("not renderable"));
        let plus_one = rfc3339_to_apple_epoch("2001-01-01T01:00:00+01:00")
            .unwrap_or_else(|| panic!("not renderable"));
        assert!((utc - plus_one).abs() < 1e-9);
    }
}
