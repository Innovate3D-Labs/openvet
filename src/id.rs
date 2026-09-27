//! Party ids are `role:name`. Names are lowercase on purpose.
//!
//! Case-folding looks friendly until two implementations fold different
//! sets of characters and disagree about whether a pool is already reserved.

use crate::error::{ErrorCode, ProtocolError};
use crate::limits::MAX_DETAIL_CHARS;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A pool id, offer id, contract id, or similar token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    pub fn parse(input: &str) -> Result<Self, ProtocolError> {
        if is_name(input) {
            Ok(Self(input.to_string()))
        } else {
            Err(ProtocolError::new(
                ErrorCode::Malformed,
                "name must be 1..=63 chars, start with a lowercase letter, then lowercase, digits, '_' or '-'",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Name {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Name::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// `csms`, `agg`, `grid`, or `hub`, plus a name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartyId(String);

impl PartyId {
    pub fn parse(input: &str) -> Result<Self, ProtocolError> {
        let Some((role, name)) = input.split_once(':') else {
            return Err(bad_party());
        };
        if !matches!(role, "csms" | "agg" | "grid" | "hub") || !is_name(name) {
            return Err(bad_party());
        }
        Ok(Self(input.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PartyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for PartyId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PartyId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        PartyId::parse(&text).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn check_message_id(input: &str) -> Result<(), ProtocolError> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "message id length must be 1..=64",
        ));
    }
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "message id must start with a lowercase letter or digit",
        ));
    }
    if bytes
        .iter()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_' || *c == b'-')
    {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ErrorCode::Malformed,
            "message id contains a character outside lowercase, digits, '_' and '-'",
        ))
    }
}

/// Machine label: `feeder_thermal`, `profile_unsupported`. Not a party name.
pub(crate) fn check_label(input: &str) -> Result<(), ProtocolError> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !bytes[0].is_ascii_lowercase() {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "label must be 1..=64 chars starting with a lowercase letter",
        ));
    }
    if bytes
        .iter()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
    {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ErrorCode::Malformed,
            "label must be lowercase, digits, or '_'",
        ))
    }
}

pub(crate) fn check_detail(input: &str) -> Result<(), ProtocolError> {
    let count = input.chars().count();
    if count == 0 || count > MAX_DETAIL_CHARS {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "detail must be 1..=256 characters",
        ));
    }
    if input.chars().any(|c| c.is_control()) {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "detail must not contain control characters",
        ));
    }
    Ok(())
}

fn is_name(input: &str) -> bool {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() > 63 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_' || *c == b'-')
}

fn bad_party() -> ProtocolError {
    ProtocolError::new(
        ErrorCode::Malformed,
        "party id must be csms|agg|grid|hub, a colon, and a lowercase name",
    )
}
