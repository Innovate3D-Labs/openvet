# Governance

September 2026. There is one repository and no governing board.

## Who decides

Nobody has been appointed. While that is true, the text in `docs/spec/` changes by pull request, and a change that breaks 0.1 is allowed. The version string on the wire is `"0.1"`. It means "this draft", not "stable minor release".

When a second implementation exists that is not a copy of this crate, breaking changes should be called out in [CHANGELOG.md](CHANGELOG.md) in a way an implementer can grep. Until then the changelog is a courtesy.

## What "normative" means here

`docs/spec/` is normative. Diagrams explain it. `schemas/openvet-0.1.schema.json` is a mechanical check for the shapes that fit in JSON Schema. The crate in `src/` is a mechanical check for the rules that need a state machine, plus a WebSocket session a host can embed.

If the schema accepts a message the prose forbids, the schema is wrong. If the crate accepts a message the prose forbids, the crate is wrong. If you prefer the code, change the prose first and say why in `docs/decisions/`.

## Affiliations that do not exist

OpenVET is not a project of the Linux Foundation, LF Energy, the Open Charge Alliance, or any regulator. A future home in one of those places is a reasonable conversation after a pilot, because those organisations already host the neighbouring standards. Putting their logos on a README before they have heard of the work is how drafts embarrass their authors.

## Releases

Tag `v0.1.0` only when the conformance list and the crate agree and someone has used the messages against a system that is not this repository. That has not happened. Do not tag it early to make the repo look finished.

## Patents

Apache-2.0 includes a patent grant from contributors. If you cannot offer that grant for a change, do not open the pull request. There is no separate patent policy because there is no entity to enforce one.
