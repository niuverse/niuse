+++
id = "M5-011"
title = "TUI brief/report/steps composition"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M5-010"]
docs = ["roadmap", "evals"]
+++

## Problem

`/brief`, `/report`, and `/steps` were tested individually or through their
CLI commands, but not run back-to-back in one TUI session. That is the path a
user actually takes when deciding what to do next.

## Scope

- Seed a plan step, goal, overdue loop, and a dogfood row in a temp home.
- Run `/brief`, `/report`, and `/steps` in one PTY session.
- Assert the brief header/loop, the report header/Dogfood line, and the plan step.

## Acceptance

- [x] `/brief` prints `niuse brief` and the overdue loop id.
- [x] `/report` prints `# niuse weekly report` and `Dogfood:`.
- [x] `/steps` prints `p-1 check brief`.
- [x] All three share one home and one command hook.
- [x] Runs under `cargo test -p niuse --features tui`.

## Notes

The test uses a replay provider with no turns, so only the product commands are
exercised. The seed data intentionally has an overdue loop and one open step.
