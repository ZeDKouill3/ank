---
id: TASK-8ad3edcbd0fb
type: task
slug: no-environment-makes-a-pipe-colored-proves-the-c
title: no_environment_makes_a_pipe_colored proves the colour rule in at most 20 ank spawns
created: 2026-10-15T16:59:10Z
author: claude-code/opus-5.5+ba8e
status: done
scope:
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  The test cli::no_environment_makes_a_pipe_colored starts ank at most 20 times, counted by the number of spawns of CARGO_BIN_EXE_ank (92 at 409bb93c: 13 styled verbs times 7 environments), and still fails when NO_COLOR is wired backwards or when the Windows allowlist (WT_SESSION, ANSICON) is consulted in place of the terminal test, each shown by a mutation run recorded in the task log.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/14c87dd8ab12@b4f85ec
    tree: scope/62dd581ca47d
    criteria: eafe953ba116
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@b4f85ec
    tree: scope/62dd581ca47d
    criteria: eafe953ba116
    verifier: fmt-check@695ca51e4ab9
    via: verifier
schema: 4
version: 3
---

Measured by TASK-ba8ec1f11085 (see its log): the largest ank spawn count in cli.rs and status.rs, 92 spawns and 592 git under them, which on a Windows runner at about 25ms a process is the most expensive test of either file. The colour decision is one function; the product of every styled verb by every environment re-proves it thirteen times.
