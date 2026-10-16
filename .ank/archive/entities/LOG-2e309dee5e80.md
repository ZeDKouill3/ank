---
id: LOG-2e309dee5e80
type: log
title: Measured after the fix, same loop, same frozen ank binary (627291a9), only the test executable
created: 2026-10-16T08:55:28Z
author: claude-code/opus-5.5+flake
scope:
  - crates/ank-tui/tests/colour.rs
  - crates/ank-tui/tests/terminal/**
about: TASK-1cd651fb434b
seq: 2
schema: 4
version: 1
---

 changed. Heavier load, 8 busy-loop shells on 4 cores: before 22 failures / 100, after 0 failures / 100 (final code, settled() guarding (reading), (not read), (not asked) plus the focused answered title). Earlier, 4 shells: before 4 / 60, after 0 / 60. Idle before: 0 / 60. Signatures of the 22 failures before: 19 had the focus mark on a title other than the one the key reached (key '4' still on 2 ENTITIES; key Enter still on the listing with the splash in 3 BODY), 2 (runs 74, 93) were the opening frame drawn before the claims/status answer, header 'branch (not asked)' and claims '(reading)', 1 (run 42) Enter's frame was the opening splash. Whole colour module, 8 shells: 0 failures / 20 (5 tests each). cargo nextest run --workspace: 1529 passed. No retry and no wait lengthened: Live::until's own deadline is untouched, only its predicates changed.
