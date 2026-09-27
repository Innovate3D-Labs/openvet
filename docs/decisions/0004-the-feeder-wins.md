# 0004. The feeder wins

Date: 2026-09-26. Status: accepted for 0.1.

## Context

A commercial deal and a connection agreement will disagree. The usual ways of hiding that are a priority integer nobody configures the same way, or a silent clamp inside the CSMS. The aggregator then thinks -20 kW happened. The feeder saw -10 kW. Settlement is a mess and, on a bad day, the feeder was right to be worried.

Frequency response was floated as a reason to let dispatch override local limits. That argument mixes a protection setting with a market position. This draft does not do frequency response at all.

## Decision

A mandatory `signal.constraint` intersects the contract's power bounds. The commercial request is clamped into the intersection. The ack is `clamped` and carries the constraint id. If the intersection is empty, the ack is `unavailable` and the applied power is 0.

A setpoint outside the **contract** is rejected, not clamped. The contract is the deal. The constraint is physics and the connection agreement. They are not the same kind of limit.

## Consequences

Aggregators must read `dispatch.ack`. An implementation that only checks that the message was acknowledged, and not the `applied_w`, is wrong.

Grid parties do not get a way, in this draft, to force an export the vehicle contract did not sell. Empty intersection means nobody invents a compromise on the wire.
