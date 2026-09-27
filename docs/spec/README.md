# Specification

OpenVET 0.1, discussion draft, 26 September 2026.

The files below are normative. Diagrams are not. The JSON Schema is not. The Rust crate is not. They exist so a disagreement has somewhere mechanical to show up.

| Document | Subject |
| --- | --- |
| [01-scope.md](01-scope.md) | Parties, neighbouring standards, out of scope |
| [02-architecture.md](02-architecture.md) | Hops, hubs, and who executes a setpoint |
| [03-security.md](03-security.md) | mTLS, party ids, privacy, what we refuse to fake |
| [04-data-model.md](04-data-model.md) | Time, power, energy, money, limits |
| [05-messages.md](05-messages.md) | Envelope and message types |
| [06-profiles.md](06-profiles.md) | Basic, contract, dispatch |
| [07-conformance.md](07-conformance.md) | Tests an implementation can fail in public |
| [open-questions.md](open-questions.md) | Decisions left open on purpose |

Reading order for a first sitting: scope, data model, messages, then the happy-path sequence in [../diagrams/sequences.md](../diagrams/sequences.md). The open questions are part of the draft, not an appendix to skip. Several of them are the reason a field you expect is missing.
