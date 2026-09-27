# 0002. The car is not a party

Date: 2026-09-26. Status: accepted for 0.1.

## Context

"Vehicle energy trading" is easy to read as a protocol spoken by the vehicle. ISO 15118-20 already covers scheduled and dynamic control between the EV and the EVSE, including bidirectional power. OCPP already carries the session up to the CSMS. A third stack on the car would have no deployment path.

The commercial relationship OpenVET is about is between companies: whoever operates the chargers, and whoever takes a position against a market or a DSO.

## Decision

Party roles are `csms`, `agg`, `grid`, and `hub`. There is no `ev` role. Messages carry pool aggregates. Vehicle identifiers are out of scope, including in `ext`.

V2G is a negative power number on a pool the CSMS already controls, not a new session layer.

## Consequences

A CSMS that wants per-vehicle optimisation does that internally and offers the pool result. Two aggregators cannot address the same car through this draft, because the car is not addressable here. That is a feature until the privacy and double-counting questions have a better answer than a VIN field.
