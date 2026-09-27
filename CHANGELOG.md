# Changelog

## 0.1.0-draft — 2026-09-26

First public discussion draft.

- Scope is the CSMS–aggregator hop: signals, offers, dispatch requests, meter evidence.
- Wire format is JSON, protocol version string `0.1`, UTC timestamps with a `Z` and whole seconds.
- Money and energy are integers. Metering uses separate import and export counters.
- The deal state machine and the pool-overlap rule live in the library.
- A WebSocket session checks party direction, profile, and repeated message ids for the life of the connection.
- Settlement claims require fiscal meter intervals. The sample trace does not have them, and does not claim.
