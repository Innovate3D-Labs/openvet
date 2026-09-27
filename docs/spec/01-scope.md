# 1. Scope

## 1.1 The hop

OpenVET defines messages between organisations that already operate behind the charger:

- a **station side** (`csms`), the party that runs the charging backend and can ask chargers to follow a limit;
- an **aggregator** (`agg`), the party that pools flexibility and faces a market, a DSO contract, or a building;
- a **grid party** (`grid`), a DSO or another party entitled to send a mandatory constraint;
- a **hub** (`hub`), an optional forwarder. A hub is a party. It can read the payload. See [03-security.md](03-security.md).

A message has one sender and one receiver. The draft describes bilateral exchange. A hub may sit in the middle and still forwards the same envelopes. The hub does not become a market.

## 1.2 Neighbouring protocols

Each of these already has a job. OpenVET assumes they keep it.

| Hop | Protocol that already lives there | OpenVET's relation to it |
| --- | --- | --- |
| Vehicle to charger | ISO 15118-20 | No messages. No contract certificates. No schedule renegotiation. |
| Charger to CSMS | OCPP 2.0.1 / 2.1 | The CSMS translates. `SetChargingProfile` and transaction meter values stay in OCPP. |
| Aggregator to a demand-response program | OpenADR | A price signal may be *derived from* an OpenADR event. OpenVET does not wrap `oadrDistributeEvent`. |
| Aggregator to a DER interface | IEEE 2030.5 | Site-level DER controls stay there. A pool status is not a DERList. |

The practical test: if a requirement can be met by a field that OCPP, ISO 15118, OpenADR, or IEEE 2030.5 already carries on that hop, OpenVET does not grow a second copy of the field.

## 1.3 Products

An offer names one product.

| Product | Meaning in this draft |
| --- | --- |
| `charge_shift` | Move consumption within the window, relative to a named baseline. |
| `export_cap` | Offer discharge power (V2G). Power minimum is negative. |
| `absorb_cap` | Offer to absorb power (charge on request). Power maximum is positive. |
| `reserve` | Hold a power range available. Energy may be zero. Actually calling the reserve is a later setpoint, still inside the contract window. |

The baseline is a label (`unmanaged_charge`, `zero`, `external_ref`), not a calculated curve. How many kilowatt-hours were "shifted" compared with a world that did not happen is the oldest argument in flexibility settlement. This draft records which label the two parties said they were using. It does not compute the counterfactual. See [open-questions.md](open-questions.md).

## 1.4 Out of scope

The following are out of scope for 0.1. They are listed because each of them showed up in early sketches and would have made the draft look finished while leaving the hard part to the reader.

- Communication with a vehicle, a charger, or a connector.
- Driver identification, contracts with drivers, roaming (OCPI), and payment at the charger.
- Fiscal invoicing, tax, and who is allowed to be the invoice issuer. A `settlement.claim` is a statement of meter totals between the two parties. It becomes an invoice only under local metering law, outside this protocol.
- Market clearing, imbalance prices, and bid formats of a specific power exchange.
- Primary frequency response, voltage ride-through, and protection settings.
- Certificates, firmware, diagnostics, and the rest of the CSMS job.

Regulatory programs are context, not requirements. AFIR sets expectations for drivers about price transparency at public chargers. NEVI is a US infrastructure funding program. FERC Order 2222 is about wholesale-market access for distributed-energy aggregations in the United States. None of them specifies this JSON. A compliance claim that treats OpenVET as the implementation of those texts is incorrect.

## 1.5 A principle for extensions

Unknown **labels** are tolerable. Unknown **obligations** are not.

A constraint reason or an error code this implementation has never seen is still a valid message; the receiver logs it and continues. A message type, a product, a meter quality, or a priority this implementation has never seen is rejected. Those change what a party must do.

Vendor extras go in the envelope `ext` object. An `ext` key must not change the meaning of a normative field. Receivers ignore keys they do not understand.
