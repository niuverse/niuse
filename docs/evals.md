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

The script runs the Rust cases with `cargo test`, the notes-pack MCP
integration, and (when the sibling checkout exists) niucode's MCP client and
agent-dispatch tests. No API key, network, or model call is involved.

## Cases

| Case | What it proves |
|---|---|
| `memory_on_off_changes_the_answer...` | A remembered judgement changes the answer; the same provider without it does not cite it |
| `goal_injection_survives_only_while_open` | An open goal is injected; a closed goal disappears from the prompt |
| `brief_surfaces_overdue_promises_and_stale_memory` | The startup brief reports overdue promises, stale memory, and the review queue |
| `plan_steps_close_only_after_step_verification` | A step is refused until `verify --step` passes, then closes and shows in evidence |
| `verify_records_and_evidence_reports_it` | A passing and a failing `niuse verify` append events and appear in evidence |
| `evidence_json_reports_files_and_checks` | A thread's write path, check, and failure are reported as JSON |
| `a_turn_can_call_memory_search` | The model calls `memory_search` and receives both fresh and expired memory with ids |
| `notes_pack_searches_reads_and_refuses_escape` | A real MCP pack searches, reads, and rejects a path escape |
| `a_hung_tool_times_out_and_kills_the_server` | A hung MCP server cannot stall the session |
| `a_writable_server_is_refused_before_spawn` | A non-read-only manifest never starts |

Each case is named for the capability, so a red test says what regressed
instead of only which file changed.

## What this is not

- It does not measure model quality; the provider is scripted.
- It does not cover token cost or latency.
- It does not replace dogfood. Real use still decides whether the companion is
  useful; these cases keep the plumbing honest between dogfood days.

## Next cases

- A dogfood trend check that blocks a release when acceptance or token trends
  regress. This needs real days, not a scripted provider.
