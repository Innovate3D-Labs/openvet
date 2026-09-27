# Roadmap

Dates without an owner are a slide. This file names the next pieces of work. It does not promise a quarter.

## Now

A library crate in `src/` that a CSMS or an aggregator can link. It parses envelopes, runs the deal state machine, and carries them on one WebSocket. `examples/basic_server.rs` and `examples/basic_client.rs` speak a single offer and accept on localhost, without TLS.

The sample deal in `examples/deal-happy.ndjson` stops at an operational meter reading. It does not settle. That is deliberate. Most CSMS meter streams are not fiscal meters, and a demo that prints an invoice from an OCPP MeterValues feed is a demo of a lie.

## Next, still inside this repository

- A written mapping of which OCPP 2.0.1 / 2.1 messages a CSMS would translate into `status.pool`, `meter.interval`, and `dispatch.setpoint`. The mapping is an implementation note. OCPP remains the charger protocol.
- A pilot log from one operator and one aggregator, even a small depot, appended as an anonymised trace. Ten real offers will change the spec more than another month of editing.

## Parked until a pilot forces them

- MQTT topic layout and HTTP. A sketch would be treated as a binding.
- CBOR. If a message does not fit in 256 KiB of JSON, it is carrying too much.
- Payload signatures and a PKI profile. mTLS authenticates the peer. It does not give you a signature you can show a court three years later. Pretending otherwise is worse than leaving the field out.
- Droop, frequency, and voltage control. Local, safety-related, and already in inverter and charger firmware.
- Python bindings, C++ port, dashboards. Bindings of a moving draft freeze the mistakes.

## Explicitly somebody else's roadmap

ISO 15118, OCPP, OpenADR, and IEEE 2030.5 have their own working groups. OpenVET should track them closely enough to avoid a second way to say the same setpoint, and no closer.
