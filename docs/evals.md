+++
id = "evals"
title = "Offline personal-agent evals"
kind = "doc"
status = "living"
order = 40
summary = "Named offline cases for memory injection, goal lifecycle, the startup brief, and the notes MCP pack. A prompt/turn regression suite, not a model benchmark."
+++

## Run

```sh
./scripts/eval.sh
```

The script runs the Rust cases with `cargo test` and then the notes-pack MCP
integration. No API key, network, or model call is involved.

## Cases

| Case | What it proves |
|---|---|
| `memory_on_off_changes_the_answer...` | A remembered judgement changes the answer; the same provider without it does not cite it |
| `goal_injection_survives_only_while_open` | An open goal is injected; a closed goal disappears from the prompt |
| `brief_surfaces_overdue_promises_and_stale_memory` | The startup brief reports overdue promises, stale memory, and the review queue |
| `notes_pack_searches_reads_and_refuses_escape` | A real MCP pack searches, reads, and rejects a path escape |

Each case is named for the capability, so a red test says what regressed
instead of only which file changed.

## What this is not

- It does not measure model quality; the provider is scripted.
- It does not cover token cost or latency.
- It does not replace dogfood. Real use still decides whether the companion is
  useful; these cases keep the plumbing honest between dogfood days.

## Next cases

- Recall across all six stores through the model's `memory_search` tool.
- MCP timeout and read-only refusal through the same eval runner.
- A dogfood trend check that blocks a release when acceptance or token trends
  regress.
