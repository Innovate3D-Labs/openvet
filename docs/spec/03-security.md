# 3. Security

## 3.1 What 0.1 actually requires

Send envelopes over TLS 1.2 or later. TLS 1.3 is what you want in a new deployment. Authenticate both ends with certificates. The receiver accepts a message when the authenticated peer is the party in `from`.

That last sentence is the whole authorisation model. There is no bearer token, no username, and no JSON signature. Tokens end up in logs. A `sig` field without a canonicalisation rule and a named algorithm is security theatre, so there isn't one.

The certificate profile that binds a subject to `csms:example-municipal` is **not** defined. Neither is revocation. A pilot has to pick both and write them down as local policy. The spec will not pretend they are solved. See [open-questions.md](open-questions.md).

## 3.2 What not to reuse

ISO 15118 contract certificates and the V2G PKI identify a vehicle to a charger. They are the wrong objects for "this CSMS may talk to this aggregator". Use an organisational certificate issued for the backend. Mixing the two PKIs feels neat in a diagram and fails in the first audit, because the vehicle owner, the charge-point operator, and the aggregator are different legal persons.

## 3.3 Hubs

If a hub forwards the envelope, the hub sees prices, pool totals, and contract ids. Payload encryption independent of the hop is not in this draft. A party that cannot accept a hub reading the commercial terms should not introduce a hub.

A hub forwards. It does not originate `offer.accept` for someone else. The `from` field remains the party that signed the TLS session on the first hop. How a hub proves that on the second hop is part of the missing certificate profile. Until that exists, prefer a direct connection.

## 3.4 Replay

A receiver should remember message ids it has accepted for at least 24 hours and reject a repeat with `conflict`. A `Session` remembers ids until the socket closes, and only up to 4096 of them. That cap is a library limit so one connection cannot grow without bound. It is not a substitute for the 24-hour memory of a process that restarts. Ids are not written to disk.

## 3.5 Privacy

Normative messages carry organisation ids, pool names, counts, power, energy, and prices.

They do not carry VIN, EVCCID, contract certificate ids, RFID, e-mail addresses, customer account numbers, or geo coordinates. Do not put those in `ext` either. `ext` is for a vendor's trace id, not a side channel around this section.

Pool aggregates can still be sensitive. A depot with four vans and a status update every minute tells a neighbour a lot. That is a product decision for the operator. The protocol's contribution is to make the individual vehicle the unusual case, by not having a field for it.

## 3.6 Limits as a safety property

The size cap (256 KiB) and the power cap (10 GW absolute) are there so a broken sender fails a parse instead of a substation. They are not a rating of any real feeder. The CSMS still has to clamp to the connection agreement. OpenVET's clamp helper implements the protocol rule (mandatory constraint intersects the contract). It does not know the site.
