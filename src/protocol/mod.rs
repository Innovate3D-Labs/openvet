//! Deal state, and with the `transport` feature one WebSocket session.

mod deal;
mod dispatch;

#[cfg(feature = "transport")]
mod session;

pub use deal::{Deal, DealEvent, DealState, PoolBook};
pub use dispatch::{clamp_power, effective_bounds};

#[cfg(feature = "transport")]
pub use session::{accept, connect, Session, SessionError};
