//! Who is allowed to speak on this socket.
//!
//! OpenVET 0.1 authenticates with mutual TLS and then requires `from` to be
//! that peer. It does not say which certificate extension stores the party id,
//! which CA is trusted, or how fast a stolen key is revoked. Those choices
//! belong to the host. This module checks the consequence: an envelope on a
//! session may only travel between the two parties the host configured.

use crate::envelope::Envelope;
use crate::error::{ErrorCode, ProtocolError};
use crate::id::PartyId;
use crate::messages::PeerCaps;

/// The two parties on one connection, as the host understands them after TLS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    pub local: PartyId,
    pub remote: PartyId,
    pub caps: PeerCaps,
}

/// `inbound` means the envelope just arrived. Otherwise the host is about to send it.
pub fn enforce_direction(
    envelope: &Envelope,
    local: &PartyId,
    remote: &PartyId,
    inbound: bool,
) -> Result<(), ProtocolError> {
    let (expect_from, expect_to) = if inbound {
        (remote, local)
    } else {
        (local, remote)
    };
    if &envelope.from != expect_from || &envelope.to != expect_to {
        return Err(ProtocolError::new(
            ErrorCode::WrongParty,
            "envelope parties do not match this session",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{Message, StatusPool};
    use crate::time::Timestamp;
    use crate::Envelope;
    use std::collections::BTreeMap;

    fn party(text: &str) -> PartyId {
        PartyId::parse(text).unwrap()
    }

    fn sample(from: &str, to: &str) -> Envelope {
        Envelope {
            v: "0.1".to_string(),
            id: "m-1".to_string(),
            from: party(from),
            to: party(to),
            ts: Timestamp::parse("2026-09-26T16:00:00Z").unwrap(),
            corr: None,
            ext: BTreeMap::new(),
            msg: Message::StatusPool(StatusPool {
                pool_id: crate::id::Name::parse("depot-7").unwrap(),
                observed_at: Timestamp::parse("2026-09-26T15:59:00Z").unwrap(),
                vehicles_connected: 1,
                power_w: 0,
                energy_capacity_wh: 1000,
                energy_stored_wh: 0,
                import_available_w: 1000,
                export_available_w: 0,
            }),
        }
    }

    #[test]
    fn inbound_must_come_from_the_remote_party() {
        let local = party("csms:example-municipal");
        let remote = party("agg:example-flex");
        let incoming = sample("agg:example-flex", "csms:example-municipal");
        enforce_direction(&incoming, &local, &remote, true).unwrap();
        let spoofed = sample("grid:example-dso", "csms:example-municipal");
        assert_eq!(
            enforce_direction(&spoofed, &local, &remote, true)
                .unwrap_err()
                .code,
            ErrorCode::WrongParty
        );
    }
}
