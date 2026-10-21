+++
id = "M2-055"
title = "Bootstrap progress only on a TTY"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["architecture"]
+++

## Scope

- The `\r[1/2] 抽取中…` progress line is written only when stderr is a terminal.
- Non-TTY runs no longer put carriage returns into captured logs; the summary still prints.
- Per-session failures still skip that session and continue.
- No behavior change for interactive use.

## Acceptance

- [x] Piped bootstrap output contains no carriage returns.
- [x] Interactive runs still show in-place progress.
- [x] Skipped sessions keep their warning.
- [x] The final `eprintln!` is gated with the same flag.

## Notes

Found while running bootstrap from a captured shell: the progress line is fine on a terminal and noise in a log. The summary is the durable output.
