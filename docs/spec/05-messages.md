# 5. Messages

## 5.1 Envelope

Every message is a JSON object.

| Field | Required | Meaning |
| --- | --- | --- |
| `v` | yes | The string `"0.1"`. Not SemVer. `"0.1.0"` is a different value and is rejected. |
| `id` | yes | Sender-chosen id, unique per sender for at least a day. See [03-security.md](03-security.md). |
| `type` | yes | One of the names in the table below. |
| `ts` | yes | Sender timestamp, canonical form. |
| `from` | yes | Party id of the sender. Must be the TLS peer. |
| `to` | yes | Party id of the receiver. |
| `corr` | conditional | Id of the message this one answers. Required on the types marked below. Absent otherwise. |
| `body` | yes | Object. Shape depends on `type`. |
| `ext` | no | Object of vendor keys. Keys match `vendor.token` (lowercase, one dot). Ignored if not understood. |

`from` and `to` are different parties. A party does not send itself an offer.

Field names are `snake_case`. OCPP uses `camelCase`. The difference is recorded in [decision 0001](../decisions/0001-json-and-a-small-clock.md) so that a later "alignment" patch does not silently fork the wire format.

Unknown message types are rejected with `unknown_type`. Unknown fields inside a known body are rejected with `malformed`. Extensions go in `ext`.

### Types

| `type` | Profile | `corr` | Body |
| --- | --- | --- | --- |
| `signal.price` | basic | no | Price curve |
| `signal.constraint` | basic | no | Power limit on a pool |
| `status.pool` | basic | no | Aggregate pool telemetry |
| `offer.flexibility` | contract | no | New offer, `revision` 1 |
| `offer.accept` | contract | yes | Accept the current revision |
| `offer.reject` | contract | yes | Reject the current revision |
| `offer.counter` | contract | yes | Replace terms, next revision |
| `contract.activate` | contract | no | Executor commits, assigns `contract_id` |
| `contract.cancel` | contract | no | Either party, accepted deals only |
| `contract.complete` | contract | no | Either party, active deals only |
| `dispatch.setpoint` | dispatch | no | Requested power inside an active contract |
| `dispatch.ack` | dispatch | yes | What the executor applied |
| `meter.interval` | contract | no | In/out energy for one interval |
| `settlement.claim` | contract | no | Sum of fiscal intervals |
| `settlement.dispute` | contract | no | One party contests an active or completed deal |
| `error` | basic | yes | Machine code plus a short detail |

## 5.2 Common shapes

An interval:

```json
{ "start": "2026-09-26T18:00:00Z", "end": "2026-09-26T20:00:00Z" }
```

A price:

```json
{ "currency": "EUR", "per": "kWh", "scale": -4, "amount": 1500 }
```

That is 0.1500 EUR/kWh.

Power bounds and energy bounds:

```json
{ "min": -50000, "max": 150000 }
```

```json
{ "min": 0, "max": 400000 }
```

## 5.3 Signals and status

### `signal.price`

Sent by whoever is quoting a price, usually the aggregator.

```json
{
  "signal_id": "px-20260926-16",
  "valid": { "start": "2026-09-26T16:00:00Z", "end": "2026-09-27T16:00:00Z" },
  "currency": "EUR",
  "per": "kWh",
  "scale": -4,
  "points": [
    {
      "start": "2026-09-26T16:00:00Z",
      "end": "2026-09-26T20:00:00Z",
      "import_amount": 3200,
      "export_amount": 900
    }
  ]
}
```

`import_amount` is the price of energy into the vehicle. `export_amount` is the price of energy out of it. They are different numbers in every tariff worth implementing. Points lie inside `valid`, are sorted by `start`, and do not overlap. Gaps are allowed. A gap means "no quote", not "price is zero".

### `signal.constraint`

Sent by a grid party, or by a CSMS enforcing a connection agreement it already has.

```json
{
  "constraint_id": "lim-feeder-9",
  "pool_id": "depot-7",
  "valid": { "start": "2026-09-26T18:00:00Z", "end": "2026-09-26T20:00:00Z" },
  "power_w": { "min": -10000, "max": 80000 },
  "reason": "feeder_thermal",
  "mandatory": true
}
```

`mandatory: true` means commercial dispatch is clamped into `power_w`. `mandatory: false` is advice. The receiver may log it and follow the contract. Known reason strings include `feeder_thermal`, `voltage`, and `connection_agreement`. Other lowercase reasons are valid; they are labels.

### `status.pool`

```json
{
  "pool_id": "depot-7",
  "observed_at": "2026-09-26T15:59:00Z",
  "vehicles_connected": 17,
  "power_w": 86000,
  "energy_capacity_wh": 2100000,
  "energy_stored_wh": 980000,
  "import_available_w": 150000,
  "export_available_w": 40000
}
```

`observed_at` is when the CSMS assembled the sums. It is not after the envelope `ts`, and it is not more than 900 seconds earlier. `power_w` uses the sign convention from the data model. Available power is a non-negative headroom in that direction, not a second signed number.

`energy_stored_wh` is less than or equal to `energy_capacity_wh`.

## 5.4 Offers and the deal

The state machine:

```text
Offered ---- accept ----> Accepted ---- activate ----> Active ---- complete ----> Completed
   |                         |                           |                            |
   +-- reject --> Cancelled  +-- cancel --> Cancelled    +-- dispute --> Disputed <---+
   |
   +-- counter --> Countered -- accept / reject / counter (same rules)
```

Roles:

- The sender of `offer.flexibility` is the **offerer** (seller of those terms). The receiver is the **counterparty**.
- Only the counterparty may accept, reject, or counter the current revision.
- A counter replaces the terms, increments the revision by exactly 1, and **swaps** offerer and counterparty. The party who just proposed the new terms is now the seller. The executor, pool, and product stay.
- Only the executor may activate. Either party may cancel an accepted deal, complete an active deal, or dispute an active or completed deal.
- There is no cancel from `Active`. Ending early is `contract.complete` plus the meter intervals you actually have. A one-sided "mutual cancel" flag would be a story about a phone call. The draft does not have a second signature to back it up.

Expiry: `expires_at` is the last instant at which the current revision may be accepted. The receiver compares it to the receiver's clock. After acceptance, expiry does not unwind the deal. Activation is allowed until `window.end`.

Reservation: entering `Accepted` or `Active` fails with `overlap` if another accepted or active deal on the same `pool_id` has an overlapping window. Offered and countered deals do not reserve the pool. First acceptance wins.

### `offer.flexibility`

`revision` is 1. Full example: [../../examples/offer.flexibility.json](../../examples/offer.flexibility.json).

Power bounds always satisfy `min <= max`, and they are not the useless point `0, 0`. A fixed power (`min == max`) is allowed. On top of that:

- `export_cap` includes discharge (`min < 0`);
- `absorb_cap` includes charge (`max > 0`).

Energy bounds are non-negative and `min <= max`. Products other than `reserve` have `max > 0`. A reserve may be power-only, with both energy bounds at 0.

### `offer.accept`, `offer.reject`

```json
{ "offer_id": "off-1001", "revision": 1 }
```

`corr` is the id of the offer or counter being answered. `revision` matches the deal's current revision.

### `offer.counter`

Same body shape as the original offer, with `revision` equal to the current revision plus one, and `corr` set. Terms are checked as a fresh offer (expiry in the future relative to this message's `ts`, interval rules, product rules).

### `contract.activate`

```json
{ "contract_id": "ctr-1001", "offer_id": "off-1001", "revision": 1 }
```

The executor sends this. `contract_id` is chosen by the executor and then fixed.

### `contract.cancel`

```json
{ "offer_id": "off-1001", "revision": 1 }
```

Valid only in `Accepted`, and only for the current revision.

### `contract.complete`

```json
{ "contract_id": "ctr-1001", "offer_id": "off-1001" }
```

Valid only in `Active`.

### `settlement.dispute`

```json
{
  "contract_id": "ctr-1001",
  "offer_id": "off-1001",
  "detail": "export interval 18:00-18:15 disagrees with site meter by 1.2 kWh"
}
```

Valid in `Active` or `Completed`. `detail` is required. This does not start a structured negotiation. It marks the deal as disputed so both sides stop treating it as settled. Anything more sophisticated is an open question.

## 5.5 Dispatch

### `dispatch.setpoint`

The aggregator (or whoever is allowed to request delivery; 0.1 allows either party of the deal) asks for a power level.

```json
{
  "contract_id": "ctr-1001",
  "offer_id": "off-1001",
  "interval": { "start": "2026-09-26T18:00:00Z", "end": "2026-09-26T18:15:00Z" },
  "power_w": -20000,
  "priority": "commercial"
}
```

The interval lies inside the contract window. Its start is not more than 60 seconds before the envelope `ts`. `power_w` lies inside the contract's power bounds. A setpoint outside the contract is `conflict`, not a clamp. Clamping is for grid constraints, not for widening a deal.

`priority` is `commercial` or `constraint`. A `constraint` setpoint is how a mandatory limit is expressed as something the executor must acknowledge. The executor still clamps to the intersection of the contract bounds and every applicable mandatory constraint. If that intersection is empty, the honest ack is `unavailable`.

### `dispatch.ack`

```json
{
  "contract_id": "ctr-1001",
  "requested_w": -20000,
  "applied_w": -10000,
  "kind": "clamped",
  "limited_by": "lim-feeder-9"
}
```

| `kind` | Rule |
| --- | --- |
| `applied` | `applied_w == requested_w`, and `limited_by` is absent. |
| `clamped` | `applied_w != requested_w`, and `limited_by` names the constraint. |
| `unavailable` | `applied_w` is 0. `limited_by` is optional. |

Silent clamping is a conformance failure. The aggregator cannot learn physics it was not told.

The reference function `effective_bounds` intersects the contract with an optional mandatory constraint. `clamp_power` then pulls the request inside that intersection. Empty intersection yields no applied power.

## 5.6 Meters and claims

### `meter.interval`

```json
{
  "contract_id": "ctr-1001",
  "offer_id": "off-1001",
  "pool_id": "depot-7",
  "interval": { "start": "2026-09-26T18:00:00Z", "end": "2026-09-26T18:15:00Z" },
  "energy_in_wh": 0,
  "energy_out_wh": 5000,
  "quality": "operational",
  "source": "csms_aggregate"
}
```

The interval ends at or before the envelope `ts`. Future meters are rejected. `source` is a label (`csms_aggregate` is the one the examples use). `quality` is closed: `operational` or `fiscal`.

### `settlement.claim`

```json
{
  "contract_id": "ctr-1001",
  "offer_id": "off-1001",
  "energy_in_wh": 12000,
  "energy_out_wh": 8000
}
```

There is no money field. See [04-data-model.md](04-data-model.md).

A claim is internally consistent only when the caller presents one or more `meter.interval` messages that all:

- name the same contract,
- have `quality: fiscal`,
- sum to the claimed counters.

If any interval is `operational`, the result is `not_fiscal`. If the sums differ, the result is `inconsistent`. The check does not prove the meters are true. The other party has its own copy and can dispute. Omitting an interval is a lie the JSON cannot see.

## 5.7 `error`

```json
{ "code": "profile_unsupported", "detail": "peer has basic only" }
```

`corr` is required. `code` is a lowercase label. Unknown codes are valid on **inbound** error messages, so a newer peer can explain a rejection the older peer does not have an enum slot for. `detail` is at most 256 characters.

Codes the reference implementation produces:

| Code | When |
| --- | --- |
| `malformed` | JSON or a field does not match the grammar |
| `unknown_type` | `type` is not in the table |
| `unsupported_version` | `v` is not `0.1` |
| `limit` | A numeric cap or the 7-day window |
| `profile_unsupported` | The peer's declared profiles do not include this message |
| `conflict` | Replay at the application layer, setpoint outside the contract, contradictory fields |
| `expired` | Accept arrived after `expires_at` |
| `wrong_state` | Event does not apply to the current deal state |
| `wrong_party` | This party cannot take that action |
| `bad_revision` | Revision is not the one the deal is on |
| `overlap` | Pool already reserved for that window |
| `inconsistent` | Ack or claim contradicts its own inputs |
| `not_fiscal` | Claim leaned on an operational meter |

## 5.8 A full trace

[../../examples/deal-happy.ndjson](../../examples/deal-happy.ndjson) is an offer that is accepted, activated, dispatched, acknowledged, metered, and completed. The meter is `operational`. There is no settlement claim in the file. That is the honest end of the sample.
