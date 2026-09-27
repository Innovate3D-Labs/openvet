//! CSMS side of a local session.
//!
//! Reads `examples/offer.flexibility.json`, sends it to `ws://127.0.0.1:9741`
//! (or the URL in the first argument), and prints the reply.
//! Start `basic_server` first.

use openvet::{connect, Envelope, PartyId, PeerCaps, SessionConfig};
use std::env;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:9741".to_string());
    let offer = Envelope::from_slice(include_bytes!("offer.flexibility.json"))?;
    let config = SessionConfig {
        local: PartyId::parse("csms:example-municipal")?,
        remote: PartyId::parse("agg:example-flex")?,
        caps: PeerCaps::contract(),
    };
    let mut session = connect(&url, config).await?;
    session.send(&offer).await?;
    println!("sent {}", offer.summary());
    let reply = session.recv().await?;
    println!("received {}", reply.summary());
    Ok(())
}
