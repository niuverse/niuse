+++
id = "M2-024"
title = "Profile-only bootstrap"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-022"]
docs = ["plan"]
+++

## Scope

- `niuse bootstrap --profile-only` implies `--profile` and stops after drafting `user.md`.
- It skips the per-session candidate extraction, saving one call per session.
- The existing inbox is left untouched.
- `--dry-run` still previews the sessions and the profile intent without a model call.

## Acceptance

- [x] `--profile-only` sets the profile path even without `--profile`.
- [x] The command returns before candidate extraction.
- [x] Dry run reports the profile intent and lists sessions.
- [x] README documents the flag.

## Notes

Sometimes only the "who you are" half is missing. Making the user pay for candidate extraction in that case is unnecessary.
