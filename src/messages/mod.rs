//! Message bodies. Parsing lives in [`crate::envelope`].
//!
//! Unknown labels (a constraint reason, an inbound error code) are accepted.
//! Unknown obligations (a message type, a product, a meter quality) are not.

use crate::error::{ErrorCode, ProtocolError};
use crate::id::{check_detail, check_label, Name, PartyId};
use crate::limits::{
    DISPATCH_GRACE_SECONDS, MAX_PRICE_POINTS, MAX_STATUS_AGE_SECONDS, MAX_VEHICLES,
};
use crate::time::{Interval, Timestamp};
use crate::units::{
    check_currency, check_energy_value, check_power_value, check_scale, EnergyBounds, Price,
    PricePer, WattBounds,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Basic,
    Contract,
    Dispatch,
}

/// What a peer has implemented. Dispatch includes contract. Basic is always on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCaps {
    contract: bool,
    dispatch: bool,
}

impl PeerCaps {
    pub fn basic() -> Self {
        Self {
            contract: false,
            dispatch: false,
        }
    }

    pub fn contract() -> Self {
        Self {
            contract: true,
            dispatch: false,
        }
    }

    pub fn full() -> Self {
        Self {
            contract: true,
            dispatch: true,
        }
    }

    pub fn check(self, message: &Message) -> Result<(), ProtocolError> {
        let allowed = match message.profile() {
            Profile::Basic => true,
            Profile::Contract => self.contract,
            Profile::Dispatch => self.dispatch,
        };
        if allowed {
            Ok(())
        } else {
            Err(ProtocolError::new(
                ErrorCode::ProfileUnsupported,
                "peer profile does not include this message",
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Product {
    ChargeShift,
    ExportCap,
    AbsorbCap,
    Reserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineKind {
    UnmanagedCharge,
    Zero,
    ExternalRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub kind: BaselineKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<Name>,
}

impl Baseline {
    pub fn check(&self) -> Result<(), ProtocolError> {
        match self.kind {
            BaselineKind::ExternalRef => {
                if self.ref_id.is_none() {
                    return Err(ProtocolError::new(
                        ErrorCode::Inconsistent,
                        "external_ref requires ref_id",
                    ));
                }
            }
            BaselineKind::UnmanagedCharge | BaselineKind::Zero => {
                if self.ref_id.is_some() {
                    return Err(ProtocolError::new(
                        ErrorCode::Inconsistent,
                        "ref_id is only valid for external_ref",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferTerms {
    pub product: Product,
    pub pool_id: Name,
    pub window: Interval,
    pub power_w: WattBounds,
    pub energy_wh: EnergyBounds,
    pub price: Price,
    pub baseline: Baseline,
    pub executor: PartyId,
    pub expires_at: Timestamp,
}

impl OfferTerms {
    pub fn check(&self, sent_at: Timestamp) -> Result<(), ProtocolError> {
        self.window.check()?;
        self.power_w.check_offer()?;
        self.energy_wh.check()?;
        self.price.check()?;
        self.baseline.check()?;
        check_product_bounds(self.product, self.power_w, self.energy_wh)?;
        if self.expires_at <= sent_at {
            return Err(ProtocolError::new(
                ErrorCode::Expired,
                "offer expires at or before the message timestamp",
            ));
        }
        if self.expires_at > self.window.end {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "expiry is after the window end",
            ));
        }
        Ok(())
    }
}

fn check_product_bounds(
    product: Product,
    power: WattBounds,
    energy: EnergyBounds,
) -> Result<(), ProtocolError> {
    match product {
        Product::ExportCap if power.min >= 0 => {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "export_cap requires a negative power minimum",
            ));
        }
        Product::AbsorbCap if power.max <= 0 => {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "absorb_cap requires a positive power maximum",
            ));
        }
        _ => {}
    }
    if product != Product::Reserve && energy.max == 0 {
        return Err(ProtocolError::new(
            ErrorCode::Malformed,
            "energy maximum must be positive for this product",
        ));
    }
    Ok(())
}

fn executor_is_party(
    executor: &PartyId,
    from: &PartyId,
    to: &PartyId,
) -> Result<(), ProtocolError> {
    if executor == from || executor == to {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ErrorCode::WrongParty,
            "executor must be one of the two parties on the envelope",
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PricePoint {
    pub start: Timestamp,
    pub end: Timestamp,
    pub import_amount: i64,
    pub export_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalPrice {
    pub signal_id: Name,
    pub valid: Interval,
    pub currency: String,
    pub per: PricePer,
    pub scale: i8,
    pub points: Vec<PricePoint>,
}

impl SignalPrice {
    pub fn check(&self) -> Result<(), ProtocolError> {
        self.valid.check()?;
        check_currency(&self.currency)?;
        check_scale(self.scale)?;
        if self.points.is_empty() {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "price signal has no points",
            ));
        }
        if self.points.len() > MAX_PRICE_POINTS {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "more than 2048 price points",
            ));
        }
        let mut previous_end: Option<Timestamp> = None;
        for point in &self.points {
            let interval = Interval {
                start: point.start,
                end: point.end,
            };
            interval.check()?;
            if point.start < self.valid.start || point.end > self.valid.end {
                return Err(ProtocolError::new(
                    ErrorCode::Malformed,
                    "price point lies outside valid",
                ));
            }
            if let Some(end) = previous_end {
                if point.start < end {
                    return Err(ProtocolError::new(
                        ErrorCode::Malformed,
                        "price points overlap or are not sorted",
                    ));
                }
            }
            previous_end = Some(point.end);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalConstraint {
    pub constraint_id: Name,
    pub pool_id: Name,
    pub valid: Interval,
    pub power_w: WattBounds,
    pub reason: String,
    pub mandatory: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SignalConstraint {
    pub fn check(&self) -> Result<(), ProtocolError> {
        self.valid.check()?;
        self.power_w.check()?;
        check_label(&self.reason)?;
        if let Some(detail) = &self.detail {
            check_detail(detail)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusPool {
    pub pool_id: Name,
    pub observed_at: Timestamp,
    pub vehicles_connected: u32,
    pub power_w: i64,
    pub energy_capacity_wh: u64,
    pub energy_stored_wh: u64,
    pub import_available_w: u64,
    pub export_available_w: u64,
}

impl StatusPool {
    pub fn check(&self, sent_at: Timestamp) -> Result<(), ProtocolError> {
        if self.observed_at > sent_at {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "status observed_at is after ts",
            ));
        }
        if sent_at.unix() - self.observed_at.unix() > MAX_STATUS_AGE_SECONDS {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "status is older than 900 seconds",
            ));
        }
        if self.vehicles_connected > MAX_VEHICLES {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "vehicle count exceeds 1000000",
            ));
        }
        check_power_value(self.power_w)?;
        check_energy_value(self.energy_capacity_wh)?;
        check_energy_value(self.energy_stored_wh)?;
        if self.energy_stored_wh > self.energy_capacity_wh {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "stored energy exceeds capacity",
            ));
        }
        if self.import_available_w > MAX_ABS_POWER || self.export_available_w > MAX_ABS_POWER {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "available power exceeds 10 GW",
            ));
        }
        Ok(())
    }
}

const MAX_ABS_POWER: u64 = crate::limits::MAX_ABS_POWER_W as u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferFlexibility {
    pub offer_id: Name,
    pub revision: u32,
    pub terms: OfferTerms,
}

impl OfferFlexibility {
    pub fn check(
        &self,
        sent_at: Timestamp,
        from: &PartyId,
        to: &PartyId,
    ) -> Result<(), ProtocolError> {
        if self.revision != 1 {
            return Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "a new offer starts at revision 1",
            ));
        }
        self.terms.check(sent_at)?;
        executor_is_party(&self.terms.executor, from, to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferDecision {
    pub offer_id: Name,
    pub revision: u32,
}

impl OfferDecision {
    pub fn check(&self) -> Result<(), ProtocolError> {
        if self.revision == 0 {
            Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "revision starts at 1",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferCounter {
    pub offer_id: Name,
    pub revision: u32,
    pub terms: OfferTerms,
}

impl OfferCounter {
    pub fn check(
        &self,
        sent_at: Timestamp,
        from: &PartyId,
        to: &PartyId,
    ) -> Result<(), ProtocolError> {
        if self.revision < 2 {
            return Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "a counter-offer starts at revision 2",
            ));
        }
        self.terms.check(sent_at)?;
        executor_is_party(&self.terms.executor, from, to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractActivate {
    pub contract_id: Name,
    pub offer_id: Name,
    pub revision: u32,
}

impl ContractActivate {
    pub fn check(&self) -> Result<(), ProtocolError> {
        if self.revision == 0 {
            Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "revision starts at 1",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractCancel {
    pub offer_id: Name,
    pub revision: u32,
}

impl ContractCancel {
    pub fn check(&self) -> Result<(), ProtocolError> {
        if self.revision == 0 {
            Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "revision starts at 1",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRef {
    pub contract_id: Name,
    pub offer_id: Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Commercial,
    Constraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchSetpoint {
    pub contract_id: Name,
    pub offer_id: Name,
    pub interval: Interval,
    pub power_w: i64,
    pub priority: Priority,
}

impl DispatchSetpoint {
    pub fn check(&self, sent_at: Timestamp) -> Result<(), ProtocolError> {
        self.interval.check()?;
        check_power_value(self.power_w)?;
        if self.interval.start.unix() + DISPATCH_GRACE_SECONDS < sent_at.unix() {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "setpoint starts more than 60 seconds before ts",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckKind {
    Applied,
    Clamped,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchAck {
    pub contract_id: Name,
    pub requested_w: i64,
    pub applied_w: i64,
    pub kind: AckKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limited_by: Option<Name>,
}

impl DispatchAck {
    pub fn check(&self) -> Result<(), ProtocolError> {
        check_power_value(self.requested_w)?;
        check_power_value(self.applied_w)?;
        match self.kind {
            AckKind::Applied => {
                if self.applied_w != self.requested_w || self.limited_by.is_some() {
                    return Err(inconsistent_ack());
                }
            }
            AckKind::Clamped => {
                if self.applied_w == self.requested_w || self.limited_by.is_none() {
                    return Err(inconsistent_ack());
                }
            }
            AckKind::Unavailable => {
                if self.applied_w != 0 {
                    return Err(inconsistent_ack());
                }
            }
        }
        Ok(())
    }
}

fn inconsistent_ack() -> ProtocolError {
    ProtocolError::new(
        ErrorCode::Inconsistent,
        "ack kind does not match requested_w, applied_w, and limited_by",
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterQuality {
    Operational,
    Fiscal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeterInterval {
    pub contract_id: Name,
    pub offer_id: Name,
    pub pool_id: Name,
    pub interval: Interval,
    pub energy_in_wh: u64,
    pub energy_out_wh: u64,
    pub quality: MeterQuality,
    pub source: String,
}

impl MeterInterval {
    pub fn check(&self, sent_at: Timestamp) -> Result<(), ProtocolError> {
        self.interval.check()?;
        if self.interval.end > sent_at {
            return Err(ProtocolError::new(
                ErrorCode::Malformed,
                "meter interval ends after ts",
            ));
        }
        check_energy_value(self.energy_in_wh)?;
        check_energy_value(self.energy_out_wh)?;
        check_label(&self.source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementClaim {
    pub contract_id: Name,
    pub offer_id: Name,
    pub energy_in_wh: u64,
    pub energy_out_wh: u64,
}

impl SettlementClaim {
    pub fn check(&self) -> Result<(), ProtocolError> {
        check_energy_value(self.energy_in_wh)?;
        check_energy_value(self.energy_out_wh)
    }
}

/// Internal consistency of a claim and the intervals the caller attaches.
///
/// This does not prove the meters are true. The other party has its own copy.
/// Leaving an interval out is a lie the function cannot see.
pub fn check_claim(
    claim: &SettlementClaim,
    intervals: &[MeterInterval],
) -> Result<(), ProtocolError> {
    claim.check()?;
    if intervals.is_empty() {
        return Err(ProtocolError::new(
            ErrorCode::Inconsistent,
            "claim has no meter intervals",
        ));
    }
    let mut sum_in = 0u64;
    let mut sum_out = 0u64;
    for interval in intervals {
        if interval.contract_id != claim.contract_id || interval.offer_id != claim.offer_id {
            return Err(ProtocolError::new(
                ErrorCode::Inconsistent,
                "meter interval names a different contract",
            ));
        }
        if interval.quality != MeterQuality::Fiscal {
            return Err(ProtocolError::new(
                ErrorCode::NotFiscal,
                "claim includes an interval that is not fiscal",
            ));
        }
        sum_in = sum_in
            .checked_add(interval.energy_in_wh)
            .ok_or_else(|| ProtocolError::new(ErrorCode::Limit, "import energy sum overflowed"))?;
        sum_out = sum_out
            .checked_add(interval.energy_out_wh)
            .ok_or_else(|| ProtocolError::new(ErrorCode::Limit, "export energy sum overflowed"))?;
    }
    if sum_in != claim.energy_in_wh || sum_out != claim.energy_out_wh {
        return Err(ProtocolError::new(
            ErrorCode::Inconsistent,
            "claim counters are not the sum of the intervals",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementDispute {
    pub contract_id: Name,
    pub offer_id: Name,
    pub detail: String,
}

impl SettlementDispute {
    pub fn check(&self) -> Result<(), ProtocolError> {
        check_detail(&self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorBody {
    pub code: String,
    pub detail: String,
}

impl ErrorBody {
    pub fn check(&self) -> Result<(), ProtocolError> {
        // Unknown codes stay valid. A newer peer must be able to explain itself.
        check_label(&self.code)?;
        check_detail(&self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    SignalPrice(SignalPrice),
    SignalConstraint(SignalConstraint),
    StatusPool(StatusPool),
    OfferFlexibility(OfferFlexibility),
    OfferAccept(OfferDecision),
    OfferReject(OfferDecision),
    OfferCounter(OfferCounter),
    ContractActivate(ContractActivate),
    ContractCancel(ContractCancel),
    ContractComplete(ContractRef),
    DispatchSetpoint(DispatchSetpoint),
    DispatchAck(DispatchAck),
    MeterInterval(MeterInterval),
    SettlementClaim(SettlementClaim),
    SettlementDispute(SettlementDispute),
    Error(ErrorBody),
}

impl Message {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::SignalPrice(_) => "signal.price",
            Self::SignalConstraint(_) => "signal.constraint",
            Self::StatusPool(_) => "status.pool",
            Self::OfferFlexibility(_) => "offer.flexibility",
            Self::OfferAccept(_) => "offer.accept",
            Self::OfferReject(_) => "offer.reject",
            Self::OfferCounter(_) => "offer.counter",
            Self::ContractActivate(_) => "contract.activate",
            Self::ContractCancel(_) => "contract.cancel",
            Self::ContractComplete(_) => "contract.complete",
            Self::DispatchSetpoint(_) => "dispatch.setpoint",
            Self::DispatchAck(_) => "dispatch.ack",
            Self::MeterInterval(_) => "meter.interval",
            Self::SettlementClaim(_) => "settlement.claim",
            Self::SettlementDispute(_) => "settlement.dispute",
            Self::Error(_) => "error",
        }
    }

    pub fn profile(&self) -> Profile {
        match self {
            Self::SignalPrice(_)
            | Self::SignalConstraint(_)
            | Self::StatusPool(_)
            | Self::Error(_) => Profile::Basic,
            Self::DispatchSetpoint(_) | Self::DispatchAck(_) => Profile::Dispatch,
            _ => Profile::Contract,
        }
    }

    pub fn requires_corr(&self) -> bool {
        matches!(
            self,
            Self::OfferAccept(_)
                | Self::OfferReject(_)
                | Self::OfferCounter(_)
                | Self::DispatchAck(_)
                | Self::Error(_)
        )
    }

    pub fn check(
        &self,
        sent_at: Timestamp,
        from: &PartyId,
        to: &PartyId,
    ) -> Result<(), ProtocolError> {
        match self {
            Self::SignalPrice(body) => body.check(),
            Self::SignalConstraint(body) => body.check(),
            Self::StatusPool(body) => body.check(sent_at),
            Self::OfferFlexibility(body) => body.check(sent_at, from, to),
            Self::OfferAccept(body) | Self::OfferReject(body) => body.check(),
            Self::OfferCounter(body) => body.check(sent_at, from, to),
            Self::ContractActivate(body) => body.check(),
            Self::ContractCancel(body) => body.check(),
            Self::ContractComplete(_) => Ok(()),
            Self::DispatchSetpoint(body) => body.check(sent_at),
            Self::DispatchAck(body) => body.check(),
            Self::MeterInterval(body) => body.check(sent_at),
            Self::SettlementClaim(body) => body.check(),
            Self::SettlementDispute(body) => body.check(),
            Self::Error(body) => body.check(),
        }
    }
}

pub(crate) fn known_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "signal.price"
            | "signal.constraint"
            | "status.pool"
            | "offer.flexibility"
            | "offer.accept"
            | "offer.reject"
            | "offer.counter"
            | "contract.activate"
            | "contract.cancel"
            | "contract.complete"
            | "dispatch.setpoint"
            | "dispatch.ack"
            | "meter.interval"
            | "settlement.claim"
            | "settlement.dispute"
            | "error"
    )
}

pub(crate) fn type_requires_corr(type_name: &str) -> bool {
    matches!(
        type_name,
        "offer.accept" | "offer.reject" | "offer.counter" | "dispatch.ack" | "error"
    )
}
