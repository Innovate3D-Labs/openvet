# 0003. Money and energy are integers

Date: 2026-09-26. Status: accepted for 0.1.

## Context

JSON numbers are often parsed into binary floats. 0.1 EUR is already not a value you want in a settlement. Energy protocols that allow a signed net (`import - export`) lose the fact that both directions happened inside the same interval, which is normal for a V2G window.

## Decision

Prices are an integer `amount`, a `scale` from -9 to 0, and an ISO-like currency code. The value in major units is `amount × 10^scale`. The reference code formats that in decimal. It does not use floating point.

Meter intervals carry `energy_in_wh` and `energy_out_wh`, both non-negative. Setpoints stay signed watts, because a setpoint is a request for one direction at a time, not a meter.

A `settlement.claim` repeats the energy counters. It does not include a computed sum of money. The price lives on the accepted offer. Two representations of the same invoice inside one protocol is how disputes get a second argument they did not need.

## Consequences

Callers that think in euros have to convert at the edge and pick a scale. We recommend 4 fractional digits for retail-like EUR prices and fewer for wholesale EUR/MWh, but the protocol does not enforce a scale per currency.

Negative amounts are legal. Rejecting them would make the first negative-price event a spec violation rather than a data point.
