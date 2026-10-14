---
id: LOG-a2eb7d03e29a
type: log
title: "Second ank done failed exit 5; full nextest run with MSYS_NO_PATHCONV unset: 1357 run, 1356 passed,"
created: 2026-10-14T09:13:48Z
author: claude-code/2.1.299
scope:
  - crates/ank-cli/tests/**
  - crates/ank-tui/tests/**
  - .config/nextest.toml
  - crates/ank-cli/Cargo.toml
  - crates/ank-tui/Cargo.toml
about: TASK-22af5b0918a4
seq: 7
schema: 4
version: 1
---

 1 failed: ank-cli::bin/ank human::tests::ratifying_in_place_still_refuses_a_target_that_was_never_accepted, a unit test in src/ that this change does not touch, panicking on 'ssh-keygen: ' with an empty message after a git 're-init: ignored --initial-branch' warning (a fixture directory met twice). Passed 3 of 3 runs alone. A flake under load outside this scope, worth its own task.
