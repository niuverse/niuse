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
| `reports_files_checks_failures_and_denials` | Includes a completed delegation's task and summary |
| `an_unfinished_delegation_is_not_reported_as_success` | A spawn without a result is `?`, not ✓ |
| `pty_tui_closes_a_step_only_after_tagged_verification` | The full TUI plan/verify flow works in a terminal |
| `read_only_commands_read_the_store` | TUI `/steps` and tagged `/verify` share the CLI gate |
| `a_failed_verification_keeps_the_step_open_until_a_pass` | Failure stays visible, blocks done, then a pass closes it |
| `steps_execute_one_at_a_time_and_vanish_after_verification` | A closed step leaves the next prompt; the remaining step stays |
| `two_packs_plan_step_and_verification_close_the_loop` | Notes + calendar + plan + tagged verification compose in one scenario |
| `pty_tui_mounts_mcp_and_dispatches_a_tool` | The TUI mounts a manifest and dispatches a namespaced tool |
| `mcp_init_merges_new_servers_and_keeps_existing_text` | A new pack is appended; comments survive; invalid TOML is untouched |
| `mcp_init_writes_a_mountable_read_only_config` | Generated config mounts and both packs answer |
| `calendar_pack_lists_and_searches_without_network` | A second read-only pack lists, searches, and excludes out-of-window events |
| `stats_json_reports_cache_hit_and_tokens_per_turn` | Cost signals are derived from token totals, not guessed |
| `a_fat_store_stays_under_the_prompt_budget` | A 200-memory store stays under 4.2k tokens |
| `plan_injection_caps_at_six_steps` | A 50-step plan injects six, not all |
| `memory_is_truncated_before_it_can_dominate_the_prompt` | 500 memories are dropped, not appended |
| `plan_prompt_injects_open_steps_and_skips_done_ones` | Open steps reach the prompt; done steps do not |
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
