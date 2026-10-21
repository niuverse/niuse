+++
id = "M2-033"
title = "Slash-command tab completion"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-031"]
docs = ["architecture"]
+++

## Scope

- Tab at a `/` prefix completes against the command list, including subcommands such as `/mem recall` and `/journal search`.
- The inserted text ends with a space, so the next keystroke starts the argument.
- Completion is active only in the TTY editor; piped input is untouched.
- A pure `slash_candidates` function is unit tested.

## Acceptance

- [x] `/mem r` resolves to `/mem recall`.
- [x] Non-slash input and prefixes ending in a space return nothing.
- [x] Multiple matches are listed by rustyline in length order.
- [x] The completer adds no dependency beyond rustyline.

## Notes

Twenty-odd commands are too many to memorize. Completion stays deliberately shallow: commands, not arguments or memory ids.
