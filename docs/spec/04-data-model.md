# 4. Data model

## 4.1 Identifiers

A **name** is one to 63 characters: a lowercase letter, then lowercase letters, digits, `_`, or `-`.

A **party id** is `role:name`. The roles in 0.1 are `csms`, `agg`, `grid`, and `hub`.

Message ids use the same character rules as names, up to 64 characters, and may also contain digits in the first position. `offmsg-1001` is valid. UUIDs with hyphens are valid if lowercased; the usual UUID text form contains uppercase hex and **is not** valid until lowercased. That is annoying and intentional. One canonical form, no case folding in the comparator.

Party ids and pool ids are case-sensitive because they are already required to be lowercase. Equality is byte equality.

## 4.2 Time

A timestamp is exactly 20 characters: `YYYY-MM-DDTHH:MM:SSZ`.

Year from 2020 through 2100 inclusive. No fractional seconds. No numeric offset. `2026-09-26T18:00:00+02:00` is a different string and it is rejected, even though a person can see what it means. Two encodings of one instant is how intervals stop overlapping in one implementation and overlap in another.

Leap seconds are rejected (`ss` from 00 to 59). Interval arithmetic uses 86400-second days. For a 15-minute energy interval the difference does not matter. If it ever matters for a settlement, the settlement is using the wrong clock source anyway.

Intervals are half-open: `[start, end)`. An instant `t` is inside when `start <= t < end`. Two intervals overlap when each starts before the other ends.

The maximum interval length in 0.1 is 7 days (604800 seconds). A longer commitment is a sequence of offers. The cap keeps a single message reviewable and keeps the "one active deal per pool" rule from pinning a site for a year by accident.

## 4.3 Power and energy

Power is an integer number of watts.

- Positive power charges the vehicle (import from the grid's point of view, into the battery).
- Negative power discharges the vehicle (export).

Energy on the wire is never a signed net. Meters report two counters:

- `energy_in_wh`, watt-hours into the vehicle;
- `energy_out_wh`, watt-hours out of the vehicle.

Both are non-negative integers. Subtracting them is a presentation choice and it destroys information the moment a window contains both directions. Offers use the same unit for their energy bounds, also non-negative: the bound is a magnitude available, and the product plus the power sign say which way it flows.

## 4.4 Money

A price is an integer, a scale, a currency, and a per-unit.

```text
major currency units per kWh = amount × 10^scale
```

`currency` is three uppercase letters. The draft does not check that the code is on the ISO 4217 list. `scale` is an integer from -9 to 0. `per` is `kWh` only.

`amount` 1500 and `scale` -4 in `EUR` is 0.1500 EUR/kWh. Negative amounts are valid. Negative prices happen, and a protocol that rejects them will be patched on the first windy Sunday.

Floats are forbidden. JSON numbers with a fractional part fail parsing in the reference crate. The schema says `type: integer`.

The price on an offer is what the counterparty pays the offerer per kWh of the product's energy term, if they accept. After a counter-offer the roles swap, and the new offerer is the seller of the revised terms. The protocol does not compute the line item. A settlement claim carries energy totals, not a money amount, so the claim and the price cannot contradict each other inside one message. Whoever turns that into an invoice does it in their own billing system, where tax already lives.

## 4.5 Baselines

| `kind` | Meaning |
| --- | --- |
| `unmanaged_charge` | The reference is the charging the pool would have done with no deal. The curve is agreed out of band. |
| `zero` | The reference is no flow. |
| `external_ref` | `ref_id` names a document both parties already have. `ref_id` is required for this kind and forbidden for the others. |

## 4.6 Limits

These values are normative. The constants live in `src/limits.rs`. Change both together.

| Limit | Value | Reason |
| --- | --- | --- |
| Envelope size | 256 KiB | A week of quarter-hour prices fits. A memory accident does not. |
| Interval length | 604800 s | Reviewable deals. Longer contracts are a chain. |
| Absolute power | 10_000_000_000 W | 10 GW. Catches a unit error before it becomes a setpoint. |
| Energy in one field | 1_000_000_000_000 Wh | 1 TWh. Same idea. Split the pool if you are genuinely larger. |
| Price scale | -9 … 0 | Finer than nano-units of currency is noise. |
| Price points in one signal | 2048 | A week of 5-minute slots fits. A week of 1-minute slots does not; send a shorter horizon. |
| Status age | 900 s | Older than a metering period means "send a new status". |
| Dispatch grace | 60 s | A setpoint may start up to a minute before the sender's timestamp. Clock skew, not a control cycle. |
| `ext` keys | 8, each key ≤ 64 chars | An escape hatch, not a second schema. |
| Detail strings | 256 chars | Error and dispute text. No control characters other than space. |
| Vehicles in one status | 1_000_000 | A sanity cap, not a market forecast. |

## 4.7 Meter quality

`operational` means the CSMS summed its own telemetry. Useful for dispatch and for debugging a deal. Not a basis for `settlement.claim`.

`fiscal` means the sender is prepared to stand behind the number under the metering regime that applies to them. The protocol cannot check that. It can refuse to attach a claim to a stack of operational intervals, and it does.

Most European public chargers will only have `operational` data on this hop for the foreseeable future, because the fiscal meter sits in a different system with a different legal role. A pilot that needs settlement needs a story for that gap before it needs another message type.
