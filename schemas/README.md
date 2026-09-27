# JSON Schema

`openvet-0.1.schema.json` describes the envelope and the message bodies for draft 0.1.

It checks shapes: types, required fields, name grammar, the timestamp pattern, integer money. It does not check the deal state machine, pool overlap, setpoint clamping, or whether a claim's counters match the meters. Those rules are prose plus the Rust crate, because expressing them as JSON Schema turns into a second program nobody wants to debug.

If the schema and [../docs/spec/05-messages.md](../docs/spec/05-messages.md) disagree, the spec is right. Fix the schema.

The schema does not use a `https://` id. There is no canonical host for this draft.
