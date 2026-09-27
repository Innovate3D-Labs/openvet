//! Integers for power, energy, and money.
//!
//! Prices never go through binary floating point. The formatted major-unit
//! string is decimal, because that is what ends up in a log an operator reads.

use crate::error::{ErrorCode, ProtocolError};
use crate::limits::{MAX_ABS_POWER_W, MAX_ENERGY_WH, MAX_PRICE_SCALE, MIN_PRICE_SCALE};
use serde::{Deserialize, Serialize};

/// Signed watts. Positive charges the vehicle. Negative discharges it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WattBounds {
    pub min: i64,
    pub max: i64,
}

impl WattBounds {
    pub fn check(self) -> Result<(), ProtocolError> {
        if self.min > self.max {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "power minimum is above maximum",
            ));
        }
        if self.min.unsigned_abs() > MAX_ABS_POWER_W as u64
            || self.max.unsigned_abs() > MAX_ABS_POWER_W as u64
        {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "power exceeds 10 GW in either direction",
            ));
        }
        Ok(())
    }

    /// Offers cannot be the point 0 W. A constraint can: it means "no flow".
    pub fn check_offer(self) -> Result<(), ProtocolError> {
        self.check()?;
        if self.min == 0 && self.max == 0 {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "offer power bounds are 0,0",
            ));
        }
        Ok(())
    }

    pub fn contains(self, power: i64) -> bool {
        power >= self.min && power <= self.max
    }
}

/// Non-negative watt-hours. A magnitude, not a signed net.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyBounds {
    pub min: u64,
    pub max: u64,
}

impl EnergyBounds {
    pub fn check(self) -> Result<(), ProtocolError> {
        if self.min > self.max {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "energy minimum is above maximum",
            ));
        }
        if self.max > MAX_ENERGY_WH {
            return Err(ProtocolError::new(ErrorCode::Limit, "energy exceeds 1 TWh"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricePer {
    #[serde(rename = "kWh")]
    Kwh,
}

/// `amount × 10^scale` major currency units per [`PricePer`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub currency: String,
    pub per: PricePer,
    pub scale: i8,
    pub amount: i64,
}

impl Price {
    pub fn check(&self) -> Result<(), ProtocolError> {
        check_currency(&self.currency)?;
        check_scale(self.scale)?;
        Ok(())
    }

    /// Decimal major units, no binary float. `1500` at scale `-4` is `0.1500`.
    pub fn format_major(&self) -> String {
        format_scaled(self.amount, self.scale)
    }
}

pub(crate) fn check_currency(currency: &str) -> Result<(), ProtocolError> {
    let bytes = currency.as_bytes();
    if bytes.len() == 3 && bytes.iter().all(|c| c.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ErrorCode::Malformed,
            "currency must be three uppercase letters",
        ))
    }
}

pub(crate) fn check_scale(scale: i8) -> Result<(), ProtocolError> {
    if (MIN_PRICE_SCALE..=MAX_PRICE_SCALE).contains(&scale) {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ErrorCode::Limit,
            "price scale must be between -9 and 0",
        ))
    }
}

pub(crate) fn format_scaled(amount: i64, scale: i8) -> String {
    if scale >= 0 {
        return amount.to_string();
    }
    let frac = (-scale) as u32;
    let base = 10u64.pow(frac);
    let negative = amount < 0;
    let abs = amount.unsigned_abs();
    let whole = abs / base;
    let part = abs % base;
    let sign = if negative { "-" } else { "" };
    format!("{sign}{whole}.{part:0width$}", width = frac as usize)
}

pub(crate) fn check_power_value(power: i64) -> Result<(), ProtocolError> {
    if power.unsigned_abs() > MAX_ABS_POWER_W as u64 {
        Err(ProtocolError::new(
            ErrorCode::Limit,
            "power exceeds 10 GW in either direction",
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn check_energy_value(energy: u64) -> Result<(), ProtocolError> {
    if energy > MAX_ENERGY_WH {
        Err(ProtocolError::new(ErrorCode::Limit, "energy exceeds 1 TWh"))
    } else {
        Ok(())
    }
}
