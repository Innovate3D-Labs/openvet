# Contributing

This is a discussion draft with no appointed editors. Until that changes, a change lands if it keeps the spec, the schema, and the crate telling the same story.

## What we will argue about

A new message type needs a paragraph in [docs/spec/open-questions.md](docs/spec/open-questions.md) first, unless the type is replacing one that already failed a concrete case. The failure is the interesting part. "We might need this later" is how a twenty-message protocol becomes eighty.

If you change a limit (size, power cap, interval length, status age), change the number in `src/limits.rs` and the table in [docs/spec/04-data-model.md](docs/spec/04-data-model.md) in the same change. The conformance list in [docs/spec/07-conformance.md](docs/spec/07-conformance.md) and the tests have to mention the same `OV-` ids. The crate checks that.

## What a good change looks like

- Prose first. The schema and the Rust types follow the sentence you actually want implementers to obey.
- One example under `examples/` for any new message, using the same party ids as the others (`csms:example-municipal`, `agg:example-flex`, `grid:example-dso`). Inventing a new fictional utility in every file makes review harder.
- No vehicle identifiers, locations finer than a pool, payment instruments, or customer names in examples. The examples get copied into logs.
- Comments in code say why a rule exists when the code alone looks arbitrary. The 60 second dispatch grace is a clock-skew choice. A reader should not have to reverse-engineer that.

## Tone

Write like you are leaving a note for the next person who has to implement this on a Thursday. Short sentences. Concrete numbers. If something is unfinished, say what is missing and what a pilot would have to invent locally.

English is the language of the spec. A translation can come later. Two normative languages is how ambiguities get a lawyer.

## Review

Disagree with the design in the open, against the text. Personal remarks about the author are out. There is no foundation code of conduct yet because there is no foundation. If this ever has more than a handful of regular contributors, adopt the Contributor Covenant rather than writing a homemade one.

Do not commit secrets. The draft has no production endpoints and it should stay that way.

## Legal

Contributions are under Apache-2.0, the same as the repository. Nothing in the spec is legal, tax, or grid-code advice. Settlement law (in Germany, Eichrecht and metering roles; elsewhere, whatever the regulator actually wrote) is local. The protocol carries evidence. It does not decide who is allowed to invoice.
