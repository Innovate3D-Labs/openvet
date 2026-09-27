# OpenVET

Open Vehicle Energy Trading. A draft protocol for a charge-point operator and an aggregator to exchange flexibility offers, price signals, dispatch requests, and meter evidence.

Version 0.1, September 2026. Discussion draft. Messages will change. There is no compatibility promise, no certification, and no one whose job it is to answer the phone when a pilot breaks.

The prose in `docs/spec/` is the specification. The JSON Schema and the Rust crate are checks against that prose. If they disagree, the prose wins and the other two are bugs.

## What problem this is for

A CSMS already knows its chargers. An aggregator already knows the market it wants to bid into. Between those two systems, flexibility is usually a private REST API that dies when one of the two vendors changes a field name.

OpenVET is a proposal for that one hop:

```text
vehicle  --ISO 15118--  charger  --OCPP--  CSMS  --OpenVET--  aggregator  --market rules--  DSO / exchange
```

The car stays on ISO 15118. The charger stays on OCPP. The wholesale market stays on whatever that market already requires. OpenVET starts where a CSMS has a pool of connected vehicles and an aggregator wants a deal with a beginning, an end, and a meter reading attached.

V2G, V2H, and V2B are power-flow directions and site types. They are the same messages. Positive power charges the vehicle. Negative power discharges it.

## What you can read first

| If you want | Go to |
| --- | --- |
| The boundary, in one sitting | [docs/spec/01-scope.md](docs/spec/01-scope.md) |
| A deal from offer to meter | [docs/diagrams/sequences.md](docs/diagrams/sequences.md) |
| The actual JSON | [docs/spec/05-messages.md](docs/spec/05-messages.md) and [examples/](examples/) |
| Things we have not decided | [docs/spec/open-questions.md](docs/spec/open-questions.md) |
| Why a few choices look stubborn | [docs/decisions/](docs/decisions/) |

## Repository map

```text
docs/spec/        normative text
docs/diagrams/    architecture and sequences
docs/decisions/   short decision records
schemas/          JSON Schema for the 0.1 envelope
examples/         JSON traces, plus small programs that link the library
src/              the library
```

Depend on it from another crate:

```toml
[dependencies]
openvet = { path = "../openvet" }
```

```rust
let envelope = openvet::Envelope::from_slice(bytes)?;
let mut book = openvet::PoolBook::new();
```

A socket looks like this. The host is the CSMS. The remote party is the aggregator. TLS stays in the host; `connect` is the plain-websocket helper the examples use.

```rust
let session_config = openvet::SessionConfig {
    local: openvet::PartyId::parse("csms:example-municipal")?,
    remote: openvet::PartyId::parse("agg:example-flex")?,
    caps: openvet::PeerCaps::contract(),
};
let mut session = openvet::connect("ws://127.0.0.1:9741", session_config).await?;
session.send(&envelope).await?;
let reply = session.recv().await?;
```

`examples/basic_server.rs` listens on port 9741 and accepts one offer. `examples/basic_client.rs` sends `examples/offer.flexibility.json`.

## Profiles in this draft

1. **Basic.** Price signals, grid constraints, pool status.
2. **Contract.** Offers, the deal state machine, meter intervals, settlement claims.
3. **Dispatch.** Setpoints and acknowledgements on a human timescale (minutes, not cycles).

Primary frequency response stays in the inverter. This draft does not set droop curves, and it should not grow a field for them casually. That is a safety-related control problem wearing a JSON coat.

## Build

```shell
cargo test --all-targets
cargo run --example basic_server
cargo run --example basic_client
```

The tests are named with the conformance ids from [docs/spec/07-conformance.md](docs/spec/07-conformance.md). A test fails if the spec lists an id the code never mentions, or the other way around.

## Status of the institutions

This is not an LF Energy project, not an Open Charge Alliance project, and not a standard. Those organisations are reasonable places to take a draft that has survived a real pilot. Citing them on a slide does not make them involved.

Apache-2.0. See [LICENSE](LICENSE). Contribution rules are in [CONTRIBUTING.md](CONTRIBUTING.md). The roadmap is intentionally short: [ROADMAP.md](ROADMAP.md).
