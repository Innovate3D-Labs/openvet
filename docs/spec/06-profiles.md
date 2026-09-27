# 6. Profiles

A peer implements a prefix of this list.

| Profile | Messages |
| --- | --- |
| Basic | `signal.price`, `signal.constraint`, `status.pool`, `error` |
| Contract | Basic, plus `offer.*`, `contract.*`, `meter.interval`, `settlement.*` |
| Dispatch | Contract, plus `dispatch.setpoint` and `dispatch.ack` |

Dispatch without contract is not a configuration. There is nothing to dispatch against.

Capability exchange on connect is not specified. Two peers that care have to know each other's profiles before they send, by configuration. A `hello` message is easy to add and useless until it has a story for mismatch, version skew, and a peer that lies. Tracked in [open-questions.md](open-questions.md).

The reference crate exposes the check as a function of a local capability set and a parsed message (`PeerCaps::check`). Parsing still succeeds for a message the local peer would refuse to act on. You want the log to contain what arrived.

## 6.1 What dispatch is allowed to mean

A dispatch interval in the examples is 15 minutes. One-minute intervals fit in the size limits if the horizon is short. Nothing in the draft provides a cycle-by-cycle loop, a deadline below the 60 second clock grace, or a guarantee that the ack returns before the interval starts.

The executor applies a setpoint by its own means. For a CSMS that means OCPP charging profiles or whatever the site controller already speaks. OpenVET does not embed OCPP.

When a mandatory `signal.constraint` overlaps the setpoint interval, the applied power is the request clamped to the intersection of:

1. the contract power bounds, and
2. the constraint power bounds.

The ack says `clamped` and names the constraint in `limited_by`. A later commercial setpoint does not loosen that. The feeder wins. Decision record: [0004](../decisions/0004-the-feeder-wins.md).

If the intersection is empty, the executor acks `unavailable` and applies 0. It does not pick a side and hope.

## 6.2 What this profile will not grow into quietly

Frequency and voltage control were in early notes as an "advanced profile". They are not a profile in this draft.

Droop and protection settings are local behaviour of inverters and chargers. They have their own safety cases, and they sit on a timescale this JSON transport does not have. A future document can specify a way to *configure* that local behaviour. It would need a different threat model than a commercial offer. It does not get added as two extra fields on `dispatch.setpoint`.
