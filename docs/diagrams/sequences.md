# Sequences

Normative rules are in [../spec/05-messages.md](../spec/05-messages.md). These diagrams are the same story, drawn so a review can point at a step.

Party ids used throughout: `csms:example-municipal`, `agg:example-flex`, `grid:example-dso`. The JSON trace for the first diagram is [../../examples/deal-happy.ndjson](../../examples/deal-happy.ndjson).

## A deal that completes

The meter in this sequence is operational. Nobody sends `settlement.claim`. That is the point of the example: telemetry is not an invoice.

```mermaid
sequenceDiagram
  participant CSMS as csms:example-municipal
  participant AGG as agg:example-flex

  CSMS->>AGG: status.pool
  AGG->>CSMS: signal.price
  CSMS->>AGG: offer.flexibility revision 1
  AGG->>CSMS: offer.accept
  CSMS->>AGG: contract.activate
  AGG->>CSMS: dispatch.setpoint commercial
  CSMS->>AGG: dispatch.ack applied
  CSMS->>AGG: meter.interval operational
  CSMS->>AGG: contract.complete
```

## A counter-offer

The aggregator does not want the price. They send `offer.counter` at revision 2. They are now the offerer. The municipal CSMS is the counterparty and is the only one who can accept. The executor stays the CSMS.

```mermaid
sequenceDiagram
  participant CSMS as csms:example-municipal
  participant AGG as agg:example-flex

  CSMS->>AGG: offer.flexibility revision 1
  AGG->>CSMS: offer.counter revision 2
  Note over CSMS,AGG: Roles swap. Executor stays the CSMS.
  CSMS->>AGG: offer.accept revision 2
  CSMS->>AGG: contract.activate
```

## The feeder wins

A commercial discharge request that sits outside a mandatory constraint is clamped. The ack names the constraint. The CSMS does not "mostly follow" the aggregator.

```mermaid
sequenceDiagram
  participant Grid as grid:example-dso
  participant CSMS as csms:example-municipal
  participant AGG as agg:example-flex

  Note over CSMS,AGG: Contract is already active. Export down to -50 kW is allowed by the deal.
  Grid->>CSMS: signal.constraint mandatory, export floor -10 kW
  AGG->>CSMS: dispatch.setpoint -20 kW
  CSMS->>AGG: dispatch.ack clamped to -10 kW, limited_by the constraint
```

## What does not happen

There is no sequence in which OpenVET talks to the vehicle, and none in which an operational meter becomes a settlement claim without a fiscal interval. Those omissions are specified.
