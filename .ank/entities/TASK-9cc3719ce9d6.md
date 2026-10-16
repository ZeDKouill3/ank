---
id: TASK-9cc3719ce9d6
type: task
slug: a-cold-status-over-n-claim-refs-reads-the-defaul
title: A cold status over N claim refs reads the default branch for all of them in one batch
created: 2026-10-15T16:59:10Z
author: claude-code/opus-5.5+ba8e
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/status.rs
blocked_by: []
done_criteria: |
  Through the binary, counted with GIT_TRACE at an absolute path and never timed: a cold ank status --json (no index, no memoised verdict) over 500 claim refs whose tasks the default branch does not carry starts no more git processes than the same cold run over one such ref, which is 23 at 409bb93c against 1007 over 500, and none of those processes is a cat-file -p <rev>:<path> naming one entity file. A test in status.rs asserts both counts.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/092beb0961a1@d1f56b0
    tree: scope/770b6f27a365
    criteria: ee499cbd950d
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@d1f56b0
    tree: scope/770b6f27a365
    criteria: ee499cbd950d
    verifier: fmt-check@695ca51e4ab9
    via: verifier
schema: 4
version: 3
---

Measured by TASK-ba8ec1f11085 (see its log): 996 of the 1007 processes are cat-file -p main:.ank/{tasks,entities}/TASK-*.md, two per claim ref, from file_at_branch trying both layouts outside the preload, on the pruning predicate's cold path. The warm path is already invariant and asserted (status_asks_git_no_question_twice_and_reads_the_plane_in_one_batch), which is why the cold one went unseen: ADR-cc65f1388a71 holds for a verb's whole run, not only its second.
