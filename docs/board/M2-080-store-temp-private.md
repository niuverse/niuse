+++
id = "M2-080"
title = "Atomic store writes create private temp files"
status = "done"
milestone = "M2"
module = "stores"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Problem

Every store write used `path.with_extension("tmp")`: a predictable name, created with default `0644`, then chmodded to `0600` and renamed. The file was briefly world-readable, and a local attacker could pre-create that path as a symlink to a victim file.

## Fix

- The temp name is `.<file>.tmp-<pid>-<nanos>`, created with `create_new` and mode `0600` on Unix.
- The final rename preserves those permissions; no chmod window.
- A failed write or rename removes the temp file.
- A Unix test pre-creates the old predictable path as a symlink and verifies the victim is untouched.

## Acceptance

- [x] Temp mode is `0600` at creation.
- [x] The predictable `identity.tmp` symlink is not followed.
- [x] The destination still receives the new content.
- [x] No leftover temp file on the success path.

## Notes

Atomicity and privacy are different properties; the old code had the first and only eventually the second.
