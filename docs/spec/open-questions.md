# Open questions

These are part of the draft. An implementation that fills one of them in locally should write down the local answer, because the peer may have written down a different one.

## Baselines

`unmanaged_charge` is the baseline everyone wants and almost nobody can audit. The draft stores a label. It does not store the counterfactual load curve, and it does not define a calculation. A pilot that settles money on `charge_shift` needs that calculation in the contract they sign outside OpenVET. If a later version carries a curve, it needs a size budget (the 2048-point cap is the current clue) and a rule for who publishes the curve before the window starts, not after.

## Who may send a setpoint

0.1 allows either party of the deal to send `dispatch.setpoint`. That is loose. In practice the aggregator dispatches and the CSMS executes. It is loose because a DSO constraint expressed as a setpoint may be sent by the grid party via the CSMS, and the draft does not yet have a clean "on behalf of" story. Tightening this without that story just moves the exception into `ext`.

## One deal per pool

The overlap rule blocks two accepted products on the same pool and window. Real portfolios stack charge-shifting and a reserve on the same vehicles, with a priority. Getting that priority wrong double-counts megawatts, which is worse than refusing the second deal. The blunt rule stays until someone brings a trace of a stack that worked, including the hour it did not.

## Fiscal meters

`quality: fiscal` is a claim about the sender's legal situation. The draft cannot verify it. European public charging often splits the fiscal meter (Eichrecht, MID, national meter roles) from the CSMS telemetry used for operations. Until a pilot names the system that is allowed to emit `fiscal`, settlement claims are a closed door. That is preferable to an open one.

## Certificate binding and revocation

The prose says the TLS peer must be `from`. It does not say which certificate field carries the party id, which CA is trusted, or how fast a stolen CSMS key dies. OCSP and CRL behaviour at connection time is an operational choice with a latency cost. It needs a short profile document, written with someone who already runs a PKI, not another JSON example.

## Non-repudiation

mTLS proves who you are talking to today. It does not produce an artefact a third party can check in three years. Settlement disputes eventually want that. A signature scheme needs canonical JSON (or a switch to a format that has one), a named algorithm, and a decision about which fields are signed. Adding `"sig": "base64..."` before those decisions would be a bug with a confident face.

## Capability exchange

Peers are configured with each other's profiles. A handshake message is the obvious next step and it has been left out on purpose, until there is a rule for a peer that advertises dispatch and then never acks.

## Replay cache and clock source

Receivers should keep ids for 24 hours. The number is a suggestion with teeth only once two implementations share a test. The same is true of "clocks within a minute". Both want the socket-level test harness from the roadmap, not another field.

## Duplicate JSON keys

`serde_json` keeps the last duplicate key. The draft wants rejection. The reference parser does not currently detect duplicates. Treat duplicate keys as `malformed` in new parsers. A test will be added when the crate grows a parser that can see them. Until then, do not rely on a duplicate-key message meaning anything.

## Hubs

Direct connections are the assumed deployment. Hub forwarding without a defined way to preserve the original sender's identity is underspecified on purpose. Do not build a marketplace on top of the hole.
