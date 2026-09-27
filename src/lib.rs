//! Library for the OpenVET 0.1 draft.
//!
//! Link this crate into a CSMS or an aggregator. It parses envelopes, runs the
//! deal state machine, and (with the default `transport` feature) carries one
//! JSON text frame per envelope over a WebSocket.
//!
//! The normative text is `docs/spec/`. Wire version `"0.1"` is a discussion
//! draft. This crate's Cargo version is not a compatibility promise.
//!
//! # What the host still does
//!
//! Terminate TLS in the process that embeds the crate, then hand the stream to
//! [`accept`] or build the WebSocket yourself and call [`Session::from_ws`].
//! The certificate field that carries a party id is not defined. [`SessionConfig`]
//! is the host saying which party it believes is on that socket.
//!
//! [`accept`]: protocol::accept
//! [`Session::from_ws`]: protocol::Session::from_ws
//! [`SessionConfig`]: security::SessionConfig

#![forbid(unsafe_code)]

pub mod envelope;
pub mod error;
pub mod id;
pub mod limits;
pub mod messages;
pub mod protocol;
pub mod security;
pub mod time;
pub mod units;

pub use envelope::Envelope;
pub use error::{ErrorCode, ProtocolError};
pub use id::{Name, PartyId};
pub use messages::{
    check_claim, AckKind, Baseline, BaselineKind, ContractActivate, ContractCancel, ContractRef,
    DispatchAck, DispatchSetpoint, ErrorBody, Message, MeterInterval, MeterQuality, OfferCounter,
    OfferDecision, OfferFlexibility, OfferTerms, PeerCaps, PricePoint, Priority, Product, Profile,
    SettlementClaim, SettlementDispute, SignalConstraint, SignalPrice, StatusPool,
};
pub use protocol::{clamp_power, effective_bounds, Deal, DealEvent, DealState, PoolBook};
pub use security::{enforce_direction, SessionConfig};
pub use time::{Interval, Timestamp};
pub use units::{EnergyBounds, Price, PricePer, WattBounds};

#[cfg(feature = "transport")]
pub use protocol::{accept, connect, Session, SessionError};
