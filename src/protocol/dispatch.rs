//! Where a commercial request meets a mandatory limit.
//!
//! A setpoint outside the contract is rejected by [`crate::Deal::check_setpoint`].
//! Clamping is only for the intersection with a grid constraint. If that
//! intersection is empty, the executor has nothing honest to apply.

use crate::units::WattBounds;

/// Intersection of the contract and one mandatory constraint.
///
/// `None` means the two ranges do not meet. The ack for that case is
/// `unavailable`, not a compromise picked on the wire.
pub fn effective_bounds(
    contract: WattBounds,
    constraint: Option<WattBounds>,
) -> Option<WattBounds> {
    let Some(constraint) = constraint else {
        return Some(contract);
    };
    let min = contract.min.max(constraint.min);
    let max = contract.max.min(constraint.max);
    if min > max {
        None
    } else {
        Some(WattBounds { min, max })
    }
}

pub fn clamp_power(requested: i64, bounds: WattBounds) -> i64 {
    requested.clamp(bounds.min, bounds.max)
}
