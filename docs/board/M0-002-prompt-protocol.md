+++
id = "M0-002"
title = "Prompt assembly and in-band protocol"
status = "done"
milestone = "M0"
module = "prompt"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture", "plan"]
+++

## Scope

- Stable prefix: persona → identity → user → memory → protocol.
- Volatile sections last: today → loops → journal.
- Section budgets: 600 each for identity and user, 2000 for memory, 300 for loops, 600 for journal.
- In-band protocol: `<<niuse:memory>>`, `<<niuse:loop>>`, `<<niuse:journal>>`.
- Streaming strip needs a tail buffer; unterminated blocks stay visible.

## Acceptance

- [x] Section order covered by a test; expired memory excluded; entries older than 30 days marked unverified.
- [x] Oversized identity is truncated and labelled.
- [x] Protocol tests cover blocks split across chunks, partial prefixes, false markers, and unterminated blocks.
- [x] In real calls the candidate block is stripped and the prose is intact.

## Notes

The original plan had memory and loop blocks only. To avoid a second model call, a journal block was added and is written automatically.
