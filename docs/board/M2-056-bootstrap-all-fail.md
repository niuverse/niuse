+++
id = "M2-056"
title = "Bootstrap fails when every session fails"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["architecture"]
+++

## Scope

- Per-session extraction errors are still skipped, so one bad transcript does not kill the run.
- If every selected session failed and none was extracted, bootstrap exits non-zero with an explicit message.
- The existing inbox is left untouched on that path; no empty rewrite pretends success.
- A partial failure still writes whatever succeeded.

## Acceptance

- [x] An all-failed run exits 1 with `all N session(s) failed`.
- [x] The inbox file is byte-identical after the failure.
- [x] The per-session `skip …` warning remains.
- [x] The success path and `--profile-only` are unchanged.

## Notes

Found when the provider was unreachable: the old behavior printed `0 new candidate(s)` and exited 0, which reads as "nothing to remember" rather than "the extraction never ran".
