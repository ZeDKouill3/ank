---
id: LOG-64fe08bfea31
type: log
title: "Spawn count measured, not read: a temporary, uncommitted line at the top of main() appended one"
created: 2026-10-15T18:45:56Z
author: claude-code/opus-5.5+8ad3
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-8ad3edcbd0fb
seq: 1
schema: 4
version: 1
---

 line per process to $ANK_SPAWN_LOG, run as cargo build --workspace then cargo test -p ank-cli --test integration -- --exact cli::no_environment_makes_a_pipe_colored. Before (test at ec16c52f): 92 lines. After: 13 (1 claim in color_fixture + 2 verbs [context under the claim, help] x (1 reference + 5 environments)). Two verbs suffice because style::detect() runs once in main.rs before dispatch; no_verb_writes_an_escape_sequence_to_a_pipe still walks the whole styled surface.
