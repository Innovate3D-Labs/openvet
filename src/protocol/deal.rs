//! Deal state, without a socket and without a clock.
//!
//! Callers pass `now`. Tests can replay a trace, and a peer can decide what
//! its clock is. [`Deal::apply`] does not know about other deals.
//! [`PoolBook::apply`] does, and that is the path that enforces one
//! accepted deal per pool and window.

use crate::error::{ErrorCode, ProtocolError};
use crate::id::{Name, PartyId};
use crate::messages::{DispatchSetpoint, OfferFlexibility, OfferTerms};
use crate::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealState {
    Offered,
    Countered,
    Accepted,
    Active,
    Completed,
    Cancelled,
    Disputed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    pub offer_id: Name,
    pub revision: u32,
    pub state: DealState,
    pub offerer: PartyId,
    pub counterparty: PartyId,
    pub executor: PartyId,
    pub pool_id: Name,
    pub product: crate::messages::Product,
    pub window: crate::time::Interval,
    pub expires_at: Timestamp,
    pub terms: OfferTerms,
    pub contract_id: Option<Name>,
}

/// An action by one party. The book checks that they are allowed to take it.
#[derive(Debug, Clone)]
pub enum DealEvent {
    Accept {
        actor: PartyId,
        revision: u32,
        now: Timestamp,
    },
    Reject {
        actor: PartyId,
        revision: u32,
    },
    Counter {
        actor: PartyId,
        revision: u32,
        terms: OfferTerms,
        now: Timestamp,
    },
    Activate {
        actor: PartyId,
        contract_id: Name,
        revision: u32,
        now: Timestamp,
    },
    Cancel {
        actor: PartyId,
        revision: u32,
    },
    Complete {
        actor: PartyId,
        contract_id: Name,
    },
    Dispute {
        actor: PartyId,
        contract_id: Name,
    },
}

impl Deal {
    pub fn open(
        from: &PartyId,
        to: &PartyId,
        offer: &OfferFlexibility,
        sent_at: Timestamp,
    ) -> Result<Self, ProtocolError> {
        offer.check(sent_at, from, to)?;
        Ok(Self {
            offer_id: offer.offer_id.clone(),
            revision: offer.revision,
            state: DealState::Offered,
            offerer: from.clone(),
            counterparty: to.clone(),
            executor: offer.terms.executor.clone(),
            pool_id: offer.terms.pool_id.clone(),
            product: offer.terms.product,
            window: offer.terms.window,
            expires_at: offer.terms.expires_at,
            terms: offer.terms.clone(),
            contract_id: None,
        })
    }

    /// Apply an event without looking at other pools.
    ///
    /// Overlap is enforced by [`PoolBook::apply`]. Calling this directly is
    /// for a process that already knows it holds a single deal.
    pub fn apply(&mut self, event: DealEvent) -> Result<(), ProtocolError> {
        self.apply_with(event, &[])
    }

    pub fn apply_with(&mut self, event: DealEvent, others: &[Deal]) -> Result<(), ProtocolError> {
        match event {
            DealEvent::Accept {
                actor,
                revision,
                now,
            } => {
                self.require_negotiating()?;
                self.require_counterparty(&actor)?;
                self.require_revision(revision)?;
                if now > self.expires_at {
                    return Err(ProtocolError::new(ErrorCode::Expired, "offer expired"));
                }
                self.ensure_no_overlap(others)?;
                self.state = DealState::Accepted;
            }
            DealEvent::Reject { actor, revision } => {
                self.require_negotiating()?;
                self.require_counterparty(&actor)?;
                self.require_revision(revision)?;
                self.state = DealState::Cancelled;
            }
            DealEvent::Counter {
                actor,
                revision,
                terms,
                now,
            } => {
                self.require_negotiating()?;
                self.require_counterparty(&actor)?;
                let Some(expected) = self.revision.checked_add(1) else {
                    return Err(ProtocolError::new(
                        ErrorCode::BadRevision,
                        "revision cannot increase past the end of u32",
                    ));
                };
                if revision != expected {
                    return Err(ProtocolError::new(
                        ErrorCode::BadRevision,
                        "counter revision must be the current revision plus one",
                    ));
                }
                terms.check(now)?;
                if terms.pool_id != self.pool_id
                    || terms.product != self.product
                    || terms.executor != self.executor
                {
                    return Err(ProtocolError::new(
                        ErrorCode::Conflict,
                        "counter-offer changed pool, product, or executor",
                    ));
                }
                self.window = terms.window;
                self.expires_at = terms.expires_at;
                self.terms = terms;
                std::mem::swap(&mut self.offerer, &mut self.counterparty);
                self.revision = revision;
                self.state = DealState::Countered;
            }
            DealEvent::Activate {
                actor,
                contract_id,
                revision,
                now,
            } => {
                if self.state != DealState::Accepted {
                    return Err(wrong_state());
                }
                if actor != self.executor {
                    return Err(wrong_party());
                }
                self.require_revision(revision)?;
                // The window is half-open, so the instant `end` is already over.
                if now >= self.window.end {
                    return Err(ProtocolError::new(
                        ErrorCode::Expired,
                        "activation is at or after window end",
                    ));
                }
                self.ensure_no_overlap(others)?;
                self.contract_id = Some(contract_id);
                self.state = DealState::Active;
            }
            DealEvent::Cancel { actor, revision } => {
                if self.state != DealState::Accepted {
                    return Err(wrong_state());
                }
                self.require_party(&actor)?;
                self.require_revision(revision)?;
                self.state = DealState::Cancelled;
            }
            DealEvent::Complete { actor, contract_id } => {
                if self.state != DealState::Active {
                    return Err(wrong_state());
                }
                self.require_party(&actor)?;
                self.require_contract(&contract_id)?;
                self.state = DealState::Completed;
            }
            DealEvent::Dispute { actor, contract_id } => {
                if self.state != DealState::Active && self.state != DealState::Completed {
                    return Err(wrong_state());
                }
                self.require_party(&actor)?;
                self.require_contract(&contract_id)?;
                self.state = DealState::Disputed;
            }
        }
        Ok(())
    }

    /// Either party of the deal may request delivery in 0.1.
    /// Who *should* send setpoints is still an open question.
    pub fn check_setpoint(
        &self,
        sender: &PartyId,
        setpoint: &DispatchSetpoint,
        sent_at: Timestamp,
    ) -> Result<(), ProtocolError> {
        setpoint.check(sent_at)?;
        if self.state != DealState::Active {
            return Err(wrong_state());
        }
        self.require_party(sender)?;
        if self.contract_id.as_ref() != Some(&setpoint.contract_id)
            || self.offer_id != setpoint.offer_id
        {
            return Err(ProtocolError::new(
                ErrorCode::Conflict,
                "setpoint names a different contract",
            ));
        }
        if !self.window.contains_interval(setpoint.interval) {
            return Err(ProtocolError::new(
                ErrorCode::Conflict,
                "setpoint interval is outside the contract window",
            ));
        }
        if !self.terms.power_w.contains(setpoint.power_w) {
            return Err(ProtocolError::new(
                ErrorCode::Conflict,
                "setpoint is outside the contract power bounds",
            ));
        }
        Ok(())
    }

    fn require_negotiating(&self) -> Result<(), ProtocolError> {
        if matches!(self.state, DealState::Offered | DealState::Countered) {
            Ok(())
        } else {
            Err(wrong_state())
        }
    }

    fn require_counterparty(&self, actor: &PartyId) -> Result<(), ProtocolError> {
        if actor == &self.counterparty {
            Ok(())
        } else {
            Err(wrong_party())
        }
    }

    fn require_party(&self, actor: &PartyId) -> Result<(), ProtocolError> {
        if actor == &self.offerer || actor == &self.counterparty {
            Ok(())
        } else {
            Err(wrong_party())
        }
    }

    fn require_revision(&self, revision: u32) -> Result<(), ProtocolError> {
        if revision == self.revision {
            Ok(())
        } else {
            Err(ProtocolError::new(
                ErrorCode::BadRevision,
                "revision does not match the deal",
            ))
        }
    }

    fn require_contract(&self, contract_id: &Name) -> Result<(), ProtocolError> {
        if self.contract_id.as_ref() == Some(contract_id) {
            Ok(())
        } else {
            Err(ProtocolError::new(
                ErrorCode::Conflict,
                "contract id does not match the deal",
            ))
        }
    }

    fn ensure_no_overlap(&self, others: &[Deal]) -> Result<(), ProtocolError> {
        for other in others {
            if other.offer_id == self.offer_id || other.pool_id != self.pool_id {
                continue;
            }
            let reserved = matches!(other.state, DealState::Accepted | DealState::Active);
            if reserved && self.window.overlaps(other.window) {
                return Err(ProtocolError::new(
                    ErrorCode::Overlap,
                    "pool already reserved for this window",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PoolBook {
    deals: Vec<Deal>,
}

impl PoolBook {
    pub fn new() -> Self {
        Self { deals: Vec::new() }
    }

    pub fn open(
        &mut self,
        from: &PartyId,
        to: &PartyId,
        offer: &OfferFlexibility,
        sent_at: Timestamp,
    ) -> Result<(), ProtocolError> {
        if self
            .deals
            .iter()
            .any(|deal| deal.offer_id == offer.offer_id)
        {
            return Err(ProtocolError::new(
                ErrorCode::Conflict,
                "offer id is already in the book",
            ));
        }
        self.deals.push(Deal::open(from, to, offer, sent_at)?);
        Ok(())
    }

    pub fn apply(&mut self, offer_id: &Name, event: DealEvent) -> Result<(), ProtocolError> {
        let index = self
            .deals
            .iter()
            .position(|deal| &deal.offer_id == offer_id)
            .ok_or_else(|| ProtocolError::new(ErrorCode::Conflict, "unknown offer"))?;
        let others: Vec<Deal> = self
            .deals
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, deal)| deal.clone())
            .collect();
        self.deals[index].apply_with(event, &others)
    }

    pub fn get(&self, offer_id: &Name) -> Option<&Deal> {
        self.deals.iter().find(|deal| &deal.offer_id == offer_id)
    }
}

fn wrong_state() -> ProtocolError {
    ProtocolError::new(
        ErrorCode::WrongState,
        "that action does not apply in the current state",
    )
}

fn wrong_party() -> ProtocolError {
    ProtocolError::new(ErrorCode::WrongParty, "this party cannot take that action")
}
