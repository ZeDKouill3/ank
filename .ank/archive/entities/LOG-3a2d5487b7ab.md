---
id: LOG-3a2d5487b7ab
type: log
title: "Full cargo nextest run --workspace --no-fail-fast: 1527/1528 pass. The one red is ank-tui"
created: 2026-10-16T06:33:00Z
author: claude-code/opus-5.5+5395
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5395fef418b7
seq: 3
schema: 4
version: 1
---

 colour::without_no_color_it_paints_and_the_two_screens_are_the_same_characters, a pty test outside this scope. Run alone through nextest it failed 2 of 3 times. Run directly against target/debug/ank swapped for each binary, it failed 1 of 10 runs with the pre-change binary and 0 of 10 with this one, so the flake predates this change and is not caused by it. An earlier full-suite red on git.rs#trees_are_listed... was the is_blob mutant still compiled: restoring the source with mv kept an mtime older than the mutant build, so cargo did not rebuild; touch fixed it.
