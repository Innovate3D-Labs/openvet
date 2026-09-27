//! Aggregator side of a local session.
//!
//! Listens on 127.0.0.1:9741, accepts one offer, and sends `offer.accept`.
//! TLS is not in this example. A host that has a certificate terminates it
//! and passes the resulting stream to `Session::accept`.

use openvet::{
    accept, DealEvent, Envelope, Message, OfferDecision, PartyId, PeerCaps, PoolBook,
    SessionConfig, Timestamp,
};
use std::collections::BTreeMap;
use std::process::ExitCode;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9741").await?;
    eprintln!("listening {}", listener.local_addr()?);
    let (tcp, _) = listener.accept().await?;
    let config = SessionConfig {
        local: PartyId::parse("agg:example-flex")?,
        remote: PartyId::parse("csms:example-municipal")?,
        caps: PeerCaps::contract(),
    };
    let mut session = accept(tcp, config.clone()).await?;
    let incoming = session.recv().await?;
    println!("received {}", incoming.summary());

    let Message::OfferFlexibility(offer) = &incoming.msg else {
        return Err(format!("expected an offer, got {}", incoming.summary()).into());
    };
    let mut book = PoolBook::new();
    book.open(&incoming.from, &incoming.to, offer, incoming.ts)?;
    let now = Timestamp::parse("2026-09-26T16:05:00Z")?;
    book.apply(
        &offer.offer_id,
        DealEvent::Accept {
            actor: config.local.clone(),
            revision: offer.revision,
            now,
        },
    )?;

    let reply = Envelope {
        v: "0.1".to_string(),
        id: "m-accept".to_string(),
        from: config.local.clone(),
        to: config.remote.clone(),
        ts: now,
        corr: Some(incoming.id.clone()),
        ext: BTreeMap::new(),
        msg: Message::OfferAccept(OfferDecision {
            offer_id: offer.offer_id.clone(),
            revision: offer.revision,
        }),
    };
    session.send(&reply).await?;
    println!("sent {}", reply.summary());
    Ok(())
}
