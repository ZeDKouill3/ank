---
id: LOG-bf75029ac13d
type: log
title: "cargo nextest list --workspace on Windows, before (origin/main c859d01f): 1501 tests in all;"
created: 2026-10-14T08:19:32Z
author: claude-code/2.1.299
scope:
  - crates/ank-cli/tests/**
  - crates/ank-tui/tests/**
  - .config/nextest.toml
  - crates/ank-cli/Cargo.toml
  - crates/ank-tui/Cargo.toml
about: TASK-22af5b0918a4
seq: 4
schema: 4
version: 1
---

 ank-cli integration 790 across 40 binaries, ank-tui integration 12 (its pty suites are cfg(unix) and do not list on Windows). After: 1357 in all; ank-cli::integration 646, ank-tui::integration 12; bin/ank 364 and ank-tui lib 209 unchanged. Distinct tests, qualified by crate and suite module: 658 before, 658 after, and the two sets diff empty. The listed count drops by 144 = 4 x 36: scratch/mod.rs carries 4 #[test] of its own, compiled into each of the 37 binaries that declared mod scratch, so they were listed 148 times and are now listed 4 times. No test was lost or added; the criterion's identical count holds for distinct tests, not for the duplicated listing.
