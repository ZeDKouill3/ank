---
id: TASK-6ccedaf956d8
type: task
slug: no-crate-cites-adr-559eebf5c6f5-so-adr-ac6be1ebe
title: No crate cites ADR-559eebf5c6f5, so ADR-ac6be1ebe9aa can be ratified
created: 2026-10-16T06:15:21Z
author: claude-code/opus-5.5+sweep
status: open
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/src/verbs.rs
blocked_by: []
done_criteria: |
  git grep -E '559eebf5c|ADR-559e' -- crates prints nothing; every remaining citation of ADR-ac6be1ebe9aa points at a rule ADR-ac6be1ebe9aa states; cargo test --workspace and cargo fmt --check are green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---
