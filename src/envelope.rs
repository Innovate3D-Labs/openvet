//! One JSON object in, one JSON object out.
//!
//! Field order on the way out is the order in the spec. Equality after a
//! round trip is structural, not byte-for-byte: objects do not have a
//! canonical key order on the way in.

use crate::error::{ErrorCode, ProtocolError};
use crate::id::{check_label, check_message_id, PartyId};
use crate::limits::{MAX_EXT_KEYS, MAX_EXT_KEY_LEN, MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
use crate::messages::{
    known_type, type_requires_corr, ContractActivate, ContractCancel, ContractRef, DispatchAck,
    DispatchSetpoint, ErrorBody, Message, MeterInterval, OfferCounter, OfferDecision,
    OfferFlexibility, SettlementClaim, SettlementDispute, SignalConstraint, SignalPrice,
    StatusPool,
};
use crate::time::Timestamp;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// A parsed envelope. Invalid combinations do not survive [`Envelope::from_slice`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub v: String,
    pub id: String,
    pub from: PartyId,
    pub to: PartyId,
    pub ts: Timestamp,
    pub corr: Option<String>,
    pub ext: BTreeMap<String, Value>,
    pub msg: Message,
}

impl Envelope {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "message exceeds 256 KiB",
            ));
        }
        if bytes.is_empty() {
            return Err(ProtocolError::new(ErrorCode::Malformed, "empty message"));
        }

        let raw: RawEnvelope = serde_json::from_slice(bytes)
            .map_err(|err| ProtocolError::new(ErrorCode::Malformed, trim_serde(err.to_string())))?;

        if raw.v != PROTOCOL_VERSION {
            return Err(ProtocolError::new(
                ErrorCode::UnsupportedVersion,
                "supported version is 0.1",
            ));
        }
        check_message_id(&raw.id)?;
        let ts = Timestamp::parse(&raw.ts)?;
        let from = PartyId::parse(&raw.from)?;
        let to = PartyId::parse(&raw.to)?;
        if from == to {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "from and to must be different parties",
            ));
        }
        if let Some(corr) = &raw.corr {
            check_message_id(corr)?;
        }
        check_ext(&raw.ext)?;

        if !known_type(&raw.kind) {
            return Err(ProtocolError::new(
                ErrorCode::UnknownType,
                "unknown message type",
            ));
        }
        if !raw.body.is_object() {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "body must be a json object",
            ));
        }

        let wants_corr = type_requires_corr(&raw.kind);
        match (wants_corr, raw.corr.is_some()) {
            (true, false) => {
                return Err(ProtocolError::new(
                    ErrorCode::Malformed,
                    "this message type requires corr",
                ));
            }
            (false, true) => {
                return Err(ProtocolError::new(
                    ErrorCode::Malformed,
                    "this message type rejects corr",
                ));
            }
            _ => {}
        }

        let msg = parse_body(&raw.kind, raw.body)?;
        msg.check(ts, &from, &to)?;

        Ok(Self {
            v: raw.v,
            id: raw.id,
            from,
            to,
            ts,
            corr: raw.corr,
            ext: raw.ext,
            msg,
        })
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.v != PROTOCOL_VERSION {
            return Err(ProtocolError::new(
                ErrorCode::UnsupportedVersion,
                "supported version is 0.1",
            ));
        }
        check_message_id(&self.id)?;
        if let Some(corr) = &self.corr {
            check_message_id(corr)?;
        }
        if self.from == self.to {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "from and to must be different parties",
            ));
        }
        let wants_corr = self.msg.requires_corr();
        if wants_corr != self.corr.is_some() {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "corr does not match the message type",
            ));
        }
        check_ext(&self.ext)?;
        self.msg.check(self.ts, &self.from, &self.to)?;

        let mut map = Map::new();
        map.insert("v".to_string(), Value::String(self.v.clone()));
        map.insert("id".to_string(), Value::String(self.id.clone()));
        map.insert(
            "type".to_string(),
            Value::String(self.msg.type_name().to_string()),
        );
        map.insert("ts".to_string(), Value::String(self.ts.to_canonical()));
        map.insert("from".to_string(), Value::String(self.from.to_string()));
        map.insert("to".to_string(), Value::String(self.to.to_string()));
        if let Some(corr) = &self.corr {
            map.insert("corr".to_string(), Value::String(corr.clone()));
        }
        map.insert("body".to_string(), body_to_value(&self.msg)?);
        if !self.ext.is_empty() {
            map.insert(
                "ext".to_string(),
                serde_json::to_value(&self.ext).map_err(|err| {
                    ProtocolError::new(ErrorCode::Malformed, trim_serde(err.to_string()))
                })?,
            );
        }

        let bytes = serde_json::to_vec(&Value::Object(map))
            .map_err(|err| ProtocolError::new(ErrorCode::Malformed, trim_serde(err.to_string())))?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "message exceeds 256 KiB",
            ));
        }
        Ok(bytes)
    }

    pub fn summary(&self) -> String {
        format!(
            "{} {} {} -> {}",
            self.msg.type_name(),
            self.id,
            self.from,
            self.to
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvelope {
    v: String,
    id: String,
    #[serde(rename = "type")]
    kind: String,
    ts: String,
    from: String,
    to: String,
    #[serde(default)]
    corr: Option<String>,
    body: Value,
    #[serde(default)]
    ext: BTreeMap<String, Value>,
}

fn check_ext(ext: &BTreeMap<String, Value>) -> Result<(), ProtocolError> {
    if ext.len() > MAX_EXT_KEYS {
        return Err(ProtocolError::new(ErrorCode::Limit, "more than 8 ext keys"));
    }
    for key in ext.keys() {
        if key.len() > MAX_EXT_KEY_LEN {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "ext key longer than 64 characters",
            ));
        }
        let Some((vendor, token)) = key.split_once('.') else {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "ext key must be vendor.token",
            ));
        };
        if token.contains('.') || check_label(vendor).is_err() || check_label(token).is_err() {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "ext key must be vendor.token",
            ));
        }
    }
    Ok(())
}

fn parse_body(kind: &str, body: Value) -> Result<Message, ProtocolError> {
    let message = match kind {
        "signal.price" => Message::SignalPrice(from_value::<SignalPrice>(body)?),
        "signal.constraint" => Message::SignalConstraint(from_value::<SignalConstraint>(body)?),
        "status.pool" => Message::StatusPool(from_value::<StatusPool>(body)?),
        "offer.flexibility" => Message::OfferFlexibility(from_value::<OfferFlexibility>(body)?),
        "offer.accept" => Message::OfferAccept(from_value::<OfferDecision>(body)?),
        "offer.reject" => Message::OfferReject(from_value::<OfferDecision>(body)?),
        "offer.counter" => Message::OfferCounter(from_value::<OfferCounter>(body)?),
        "contract.activate" => Message::ContractActivate(from_value::<ContractActivate>(body)?),
        "contract.cancel" => Message::ContractCancel(from_value::<ContractCancel>(body)?),
        "contract.complete" => Message::ContractComplete(from_value::<ContractRef>(body)?),
        "dispatch.setpoint" => Message::DispatchSetpoint(from_value::<DispatchSetpoint>(body)?),
        "dispatch.ack" => Message::DispatchAck(from_value::<DispatchAck>(body)?),
        "meter.interval" => Message::MeterInterval(from_value::<MeterInterval>(body)?),
        "settlement.claim" => Message::SettlementClaim(from_value::<SettlementClaim>(body)?),
        "settlement.dispute" => Message::SettlementDispute(from_value::<SettlementDispute>(body)?),
        "error" => Message::Error(from_value::<ErrorBody>(body)?),
        _ => {
            return Err(ProtocolError::new(
                ErrorCode::UnknownType,
                "unknown message type",
            ))
        }
    };
    Ok(message)
}

fn body_to_value(message: &Message) -> Result<Value, ProtocolError> {
    let result = match message {
        Message::SignalPrice(body) => serde_json::to_value(body),
        Message::SignalConstraint(body) => serde_json::to_value(body),
        Message::StatusPool(body) => serde_json::to_value(body),
        Message::OfferFlexibility(body) => serde_json::to_value(body),
        Message::OfferAccept(body) | Message::OfferReject(body) => serde_json::to_value(body),
        Message::OfferCounter(body) => serde_json::to_value(body),
        Message::ContractActivate(body) => serde_json::to_value(body),
        Message::ContractCancel(body) => serde_json::to_value(body),
        Message::ContractComplete(body) => serde_json::to_value(body),
        Message::DispatchSetpoint(body) => serde_json::to_value(body),
        Message::DispatchAck(body) => serde_json::to_value(body),
        Message::MeterInterval(body) => serde_json::to_value(body),
        Message::SettlementClaim(body) => serde_json::to_value(body),
        Message::SettlementDispute(body) => serde_json::to_value(body),
        Message::Error(body) => serde_json::to_value(body),
    };
    result.map_err(|err| ProtocolError::new(ErrorCode::Malformed, trim_serde(err.to_string())))
}

fn from_value<T: for<'de> Deserialize<'de>>(body: Value) -> Result<T, ProtocolError> {
    serde_json::from_value(body)
        .map_err(|err| ProtocolError::new(ErrorCode::Malformed, trim_serde(err.to_string())))
}

fn trim_serde(mut detail: String) -> String {
    const MAX: usize = 180;
    if detail.len() > MAX {
        detail.truncate(MAX);
    }
    detail
}
