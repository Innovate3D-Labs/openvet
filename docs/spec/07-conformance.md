# 7. Conformance

A conforming 0.1 implementation follows the prose in this directory. The tests below are the slice of that prose which the reference crate can fail in public. They are necessary checks, not a certificate.

The id in the left column appears in this file and in `tests/conformance.rs`. The crate test `conformance_ids_match_the_spec` fails if the two lists diverge. If you add a rule, add the id in both places in the same change.

Transport behaviour, TLS, and replay caches are specified in prose and are **not** in this table. They need a running peer. The crate does not open a socket.

| Id | Rule |
| --- | --- |
| OV-ENV-01 | An envelope round-trips, and a payload that is not a JSON object fails with `malformed`. |
| OV-ENV-02 | Timestamps are the 20-character `Z` form. Offsets, fractional seconds, and impossible dates fail. |
| OV-ENV-03 | `v` other than `0.1` fails with `unsupported_version`. |
| OV-ENV-04 | `offer.accept`, `offer.reject`, `offer.counter`, `dispatch.ack`, and `error` require `corr`. Other types reject a stray `corr`. |
| OV-ENV-05 | Payloads larger than 256 KiB fail with `limit`. |
| OV-ENV-06 | `ext` keys follow `vendor.token`, and more than 8 keys fails. |
| OV-ID-01 | Party ids are `role:name` in lowercase. Unknown roles and uppercase UUID text fail. |
| OV-UNIT-01 | A price formats as decimal currency units without binary floating point. Fractional JSON numbers fail. |
| OV-UNIT-02 | A negative import price is valid. |
| OV-UNIT-03 | Absolute power above 10 GW fails with `limit`. |
| OV-PRICE-01 | Price points outside `valid`, unsorted, or overlapping fail. |
| OV-STAT-01 | Pool status older than 900 seconds, or observed after `ts`, fails. |
| OV-OFFER-01 | `examples/offer.flexibility.json` decodes, and revision on a new offer is 1. |
| OV-BASE-01 | `external_ref` requires `ref_id`. The other baseline kinds reject it. |
| OV-DEAL-01 | The sample trace runs from offer through complete. |
| OV-DEAL-02 | The offerer cannot accept their own offer. |
| OV-DEAL-03 | A counter-offer swaps roles, increments revision by one, and keeps the executor. |
| OV-DEAL-04 | Only the executor can activate. |
| OV-DEAL-05 | Cancel is valid in `Accepted` and invalid in `Active`. |
| OV-DEAL-06 | A second acceptance on the same pool and an overlapping window fails with `overlap`. |
| OV-DEAL-07 | An accept after `expires_at` fails with `expired`. |
| OV-DISP-01 | A commercial request is clamped to the intersection of contract and mandatory constraint. |
| OV-DISP-02 | An ack whose `kind` disagrees with the numbers fails with `inconsistent`. |
| OV-DISP-03 | An empty intersection is unavailable, not a guessed compromise. |
| OV-MTR-01 | Meter counters are separate non-negative integers, and a future interval fails. |
| OV-SET-01 | A claim that includes an operational interval fails with `not_fiscal`. |
| OV-SET-02 | A claim whose counters are not the sum of the presented intervals fails with `inconsistent`. |
| OV-PROF-01 | A basic-only peer rejects `offer.flexibility` with `profile_unsupported` and still accepts `status.pool`. |
