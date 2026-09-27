//! Timestamps are 20 characters, `YYYY-MM-DDTHH:MM:SSZ`.
//!
//! The grammar is deliberately smaller than RFC 3339. One encoding per
//! instant keeps interval overlap the same in every language. Conversion
//! to Unix seconds uses Howard Hinnant's public-domain civil calendar
//! algorithms (`days_from_civil` / `civil_from_days`). Leap seconds are
//! rejected. Days are 86400 seconds long.

use crate::error::{ErrorCode, ProtocolError};
use crate::limits::{MAX_INTERVAL_SECONDS, MAX_YEAR, MIN_YEAR};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A canonical UTC instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn parse(input: &str) -> Result<Self, ProtocolError> {
        let bytes = input.as_bytes();
        if bytes.len() != 20
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes[10] != b'T'
            || bytes[13] != b':'
            || bytes[16] != b':'
            || bytes[19] != b'Z'
        {
            return Err(bad_ts());
        }

        let year = four(&bytes[0..4])?;
        let month = two(&bytes[5..7])?;
        let day = two(&bytes[8..10])?;
        let hour = two(&bytes[11..13])?;
        let minute = two(&bytes[14..16])?;
        let second = two(&bytes[17..19])?;

        if !(MIN_YEAR..=MAX_YEAR).contains(&year) || !(1..=12).contains(&month) {
            return Err(bad_ts());
        }
        if day < 1 || day > days_in_month(year, month) {
            return Err(bad_ts());
        }
        if hour > 23 || minute > 59 || second > 59 {
            return Err(bad_ts());
        }

        let unix = unix_from_civil(year, month, day, hour, minute, second);
        Ok(Self(unix))
    }

    pub fn unix(self) -> i64 {
        self.0
    }

    pub fn to_canonical(self) -> String {
        let days = self.0.div_euclid(86_400);
        let sod = self.0.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let hour = sod / 3_600;
        let minute = (sod % 3_600) / 60;
        let second = sod % 60;
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_canonical())
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_canonical())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Timestamp::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// Half-open interval `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl Interval {
    pub fn check(self) -> Result<(), ProtocolError> {
        if self.end <= self.start {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "interval end must be after start",
            ));
        }
        let duration = self.end.unix() - self.start.unix();
        if duration > MAX_INTERVAL_SECONDS {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "interval longer than 7 days",
            ));
        }
        Ok(())
    }

    pub fn overlaps(self, other: Interval) -> bool {
        self.start < other.end && other.start < self.end
    }

    pub fn contains_interval(self, inner: Interval) -> bool {
        inner.start >= self.start && inner.end <= self.end
    }
}

fn bad_ts() -> ProtocolError {
    ProtocolError::new(
        ErrorCode::Malformed,
        "timestamp must be YYYY-MM-DDTHH:MM:SSZ between 2020 and 2100",
    )
}

fn four(bytes: &[u8]) -> Result<i32, ProtocolError> {
    digits(bytes)
}

fn two(bytes: &[u8]) -> Result<i32, ProtocolError> {
    digits(bytes)
}

fn digits(bytes: &[u8]) -> Result<i32, ProtocolError> {
    let mut value = 0i32;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(bad_ts());
        }
        value = value * 10 + i32::from(byte - b'0');
    }
    Ok(value)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: i32) -> i32 {
    const DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if month == 2 && is_leap(year) {
        29
    } else {
        DAYS[(month - 1) as usize]
    }
}

fn unix_from_civil(year: i32, month: i32, day: i32, hour: i32, minute: i32, second: i32) -> i64 {
    let days = days_from_civil(year, month as u32, day as u32);
    days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second)
}

/// Days since 1970-01-01. Howard Hinnant, public domain.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut y = i64::from(year);
    if month <= 2 {
        y -= 1;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let month_prime = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * u64::from(month_prime) + 2) / 5 + u64::from(day) - 1;
    let day_of_era = yoe * 365 + yoe / 4 - yoe / 100 + day_of_year;
    era * 146_097 + day_of_era as i64 - 719_468
}

/// Inverse of [`days_from_civil`]. Howard Hinnant, public domain.
fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year as i32, month as i32, day as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_anchors_the_calendar() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn canonical_timestamps_round_trip() {
        for sample in [
            "2020-01-01T00:00:00Z",
            "2024-02-29T12:00:00Z",
            "2026-09-26T16:00:00Z",
            "2100-02-28T23:59:59Z",
        ] {
            let parsed = Timestamp::parse(sample).unwrap();
            assert_eq!(parsed.to_canonical(), sample);
        }
    }

    #[test]
    fn rejects_offsets_leaps_and_impossible_days() {
        for sample in [
            "2019-12-31T00:00:00Z",
            "2101-01-01T00:00:00Z",
            "2100-02-29T00:00:00Z",
            "2023-02-29T00:00:00Z",
            "2026-09-26T16:00:00.1Z",
            "2026-09-26T16:00:00+00:00",
            "2026-09-26T18:00:00+02:00",
            "2026-13-01T00:00:00Z",
            "2026-09-26T16:00:60Z",
            "2026-09-26t16:00:00Z",
        ] {
            assert!(Timestamp::parse(sample).is_err(), "{sample}");
        }
    }
}
