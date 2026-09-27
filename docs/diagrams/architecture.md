# Architecture

The picture is an explanation of [../spec/02-architecture.md](../spec/02-architecture.md). The spec wins if they disagree.

```mermaid
flowchart LR
  vehicle[Vehicle]
  charger[Charger]
  csms[CSMS]
  agg[Aggregator]
  grid[DSO or other grid party]
  market[Market or program]

  vehicle -->|ISO 15118| charger
  charger -->|OCPP| csms
  csms -->|OpenVET offers, meters, acks| agg
  agg -->|OpenVET setpoints, prices| csms
  grid -->|OpenVET mandatory constraint| csms
  agg -->|OpenADR, IEEE 2030.5, or a market API| market
  grid --- market
```

OpenVET is the edge between CSMS and aggregator, plus an optional constraint from the grid party to the CSMS. Everything else is labelled so a reader can see who already owns it.

A hub, if someone insists, sits on the OpenVET edges and can read the envelopes. It is not drawn because 0.1 does not define how a forwarded message keeps its sender. See the open questions.
