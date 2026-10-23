+++
id = "M10-028"
title = "Computer actions in niuse evidence"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-019"]
+++

## Problem

`niuse evidence` counted tool calls and denials but ignored the kernel's
structured `computer_action` events, so a desktop task had no product-side
evidence trail.

## Scope

- Parse `Event::ComputerAction` into `Evidence.computer`.
- Render count, failures, action, window, typed length, sanitized purpose, and
  error; never typed text.
- Include computer actions in `is_empty` and JSON.
- Keep the kernel as the event owner; no new schema in niuse.
- Offline test covers a successful click and a denied type.

## Acceptance

- [x] Computer actions are counted and listed.
- [x] Failed actions are counted.
- [x] Typed text remains absent; only length is shown.
- [x] JSON includes the computer list.
- [x] Empty evidence ignores computer only when none exist.
- [x] Tests/checks pass.

## Notes

The evidence report is descriptive, not a claim that the desktop state is
correct.
