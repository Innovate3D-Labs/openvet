//! Numeric limits from `docs/spec/04-data-model.md`.
//!
//! These are part of the draft, not local configuration. Change the spec
//! table in the same commit as this file.

/// Largest envelope, in bytes. 256 KiB.
pub const MAX_MESSAGE_BYTES: usize = 256 * 1024;

/// Longest interval. Seven 86400-second days.
pub const MAX_INTERVAL_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Absolute power cap. 10 GW. A unit-error guard, not a feeder rating.
pub const MAX_ABS_POWER_W: i64 = 10_000_000_000;

/// Largest energy value in one field. 1 TWh.
pub const MAX_ENERGY_WH: u64 = 1_000_000_000_000;

pub const MIN_PRICE_SCALE: i8 = -9;
pub const MAX_PRICE_SCALE: i8 = 0;

pub const MAX_PRICE_POINTS: usize = 2048;

/// Pool status older than this is a new message, not a replay of this morning.
pub const MAX_STATUS_AGE_SECONDS: i64 = 900;

/// A setpoint may start this long before the sender timestamp. Clock skew.
pub const DISPATCH_GRACE_SECONDS: i64 = 60;

pub const MAX_EXT_KEYS: usize = 8;
pub const MAX_EXT_KEY_LEN: usize = 64;
pub const MAX_DETAIL_CHARS: usize = 256;
pub const MAX_VEHICLES: u32 = 1_000_000;

pub const MIN_YEAR: i32 = 2020;
pub const MAX_YEAR: i32 = 2100;

/// Wire version. Not Cargo semver. `"0.1.0"` is a different string.
pub const PROTOCOL_VERSION: &str = "0.1";
