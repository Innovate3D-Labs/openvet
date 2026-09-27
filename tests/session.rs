//! The library speaking to itself on localhost.

#![cfg(feature = "transport")]

use openvet::{
    accept, connect, DealEvent, Envelope, ErrorCode, Message, OfferDecision, PartyId, PeerCaps,
    PoolBook, SessionConfig, SessionError, Timestamp,
};
use std::collections::BTreeMap;
use tokio::net::TcpListener;

fn party(text: &str) -> PartyId {
    PartyId::parse(text).unwrap()
}

fn csms_config() -> SessionConfig {
    SessionConfig {
        local: party("csms:example-municipal"),
        remote: party("agg:example-flex"),
        caps: PeerCaps::contract(),
    }
}

fn agg_config() -> SessionConfig {
    SessionConfig {
        local: party("agg:example-flex"),
        remote: party("csms:example-municipal"),
        caps: PeerCaps::contract(),
    }
}

#[tokio::test]
async fn offer_is_accepted_across_a_socket_and_a_repeat_is_rejected() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut session = accept(tcp, agg_config()).await.unwrap();
        let incoming = session.recv().await.unwrap();
        let Message::OfferFlexibility(offer) = &incoming.msg else {
            panic!("expected offer, got {}", incoming.summary());
        };
        let mut book = PoolBook::new();
        book.open(&incoming.from, &incoming.to, offer, incoming.ts)
            .unwrap();
        let now = Timestamp::parse("2026-09-26T16:05:00Z").unwrap();
        book.apply(
            &offer.offer_id,
            DealEvent::Accept {
                actor: party("agg:example-flex"),
                revision: offer.revision,
                now,
            },
        )
        .unwrap();
        let reply = Envelope {
            v: "0.1".to_string(),
            id: "m-accept".to_string(),
            from: party("agg:example-flex"),
            to: party("csms:example-municipal"),
            ts: now,
            corr: Some(incoming.id.clone()),
            ext: BTreeMap::new(),
            msg: Message::OfferAccept(OfferDecision {
                offer_id: offer.offer_id.clone(),
                revision: offer.revision,
            }),
        };
        session.send(&reply).await.unwrap();
        session.send(&reply).await.unwrap();
    });

    let offer = Envelope::from_slice(include_bytes!("../examples/offer.flexibility.json")).unwrap();
    let mut client = connect(&format!("ws://{address}"), csms_config())
        .await
        .unwrap();
    client.send(&offer).await.unwrap();
    let accepted = client.recv().await.unwrap();
    assert!(matches!(accepted.msg, Message::OfferAccept(_)));
    match client.recv().await {
        Err(SessionError::Protocol(err)) => assert_eq!(err.code, ErrorCode::Conflict),
        other => panic!("expected a replay conflict, got {other:?}"),
    }
    server.await.unwrap();
}
