# 2. Architecture

## 2.1 One picture

```text
+--------+   ISO 15118   +--------+   OCPP    +------+   OpenVET   +------------+
| vehicle| ------------> | charger| --------> | CSMS | -----------> | aggregator |
+--------+               +--------+           +------+              +------------+
                                                 ^                        |
                                                 |                        |  OpenADR, market API,
                                                 |                        |  or a DSO contract
                                                 |                        v
                                                 |                  +------------+
                                                 +-- constraint ----| DSO / grid |
                                                    (OpenVET)       +------------+
```

The arrow from the grid party is optional. Many deployments will only ever run CSMS and aggregator. When a mandatory constraint arrives, it wins over a commercial setpoint. The rule is in [06-profiles.md](06-profiles.md) and decision [0004](../decisions/0004-the-feeder-wins.md).

A fuller drawing, including the hub, is in [../diagrams/architecture.md](../diagrams/architecture.md).

## 2.2 Who executes

The offer names an `executor`. In the usual case that is the CSMS: it is the only party that can turn a setpoint into OCPP charging profiles. The aggregator sends `dispatch.setpoint`. The CSMS answers with `dispatch.ack` and the power it actually applied.

The executor field is the identity of the deal. A counter-offer may change price, window, and bounds. It may not quietly replace the executor, the pool, or the product. If those are wrong, the counterparty rejects and the other side opens a new offer.

## 2.3 Pools

A `pool_id` is a name the two parties already agreed. The protocol does not allocate it and does not describe its geography.

Status messages report counts and sums: how many vehicles are connected, power now, energy stored, power available in each direction. They do not list vehicles. A pool of one vehicle is a vehicle record with the identifier removed. Parties that build single-vehicle pools have to answer that privacy question themselves; the message will not stop them.

In 0.1 a pool has at most one accepted or active deal overlapping a given window. The rule is blunt. It exists because double-selling the same kilowatts to two buyers is the failure mode that matters, and product-by-product stacking rules need an implementer who has done it for real. Split the pool (`depot-7-export`, `depot-7-charge`) if you need two products at once. The limitation is recorded in [open-questions.md](open-questions.md).

## 2.4 Transport

Normative for 0.1:

- The payload is one JSON value, UTF-8, no BOM.
- WebSocket is the carrier this draft expects. One text frame, one envelope. The subprotocol name `openvet.v0` is what the library asks for. Ping is answered with pong. Reconnect, idle timeout, and the certificate profile are not specified.
- The library checks each envelope and can carry it on a socket the host already accepted. `connect` in the examples opens a plain `ws://` socket. A deployment terminates TLS first.

MQTT, HTTP, and HTTP/2 are not bindings yet. A polling API in front of these messages creates two copies of the deal state. If a later binding adds HTTP, it should be for fetching a recorded trace, not for the live offer.

## 2.5 Time and order

There is no total order across parties. Each envelope carries the sender's timestamp. Receivers use their own clock for expiry and for the dispatch grace period. The crate takes the clock as an argument so tests can replay a trace without pretending to be NTP.

Clocks are assumed to be within a minute of each other. The draft does not specify how. Sixty seconds of grace on a dispatch is the allowance for that assumption, not a control loop.
