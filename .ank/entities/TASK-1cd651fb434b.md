---
id: TASK-1cd651fb434b
type: task
slug: the-painted-and-the-plain-colour-sessions-compar
title: The painted and the plain colour sessions compare the same settled screens
created: 2026-10-16T08:04:14Z
author: claude-code/opus-5.5+flake
status: in_progress
scope:
  - crates/ank-tui/tests/colour.rs
  - crates/ank-tui/tests/terminal/**
blocked_by: []
done_criteria: |
  colour::without_no_color_it_paints_and_the_two_screens_are_the_same_characters passes 50 runs out of 50 under load, against the failure rate measured before the fix, with no retry and no longer wait added
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 4
---
