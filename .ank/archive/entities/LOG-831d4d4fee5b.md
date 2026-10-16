---
id: LOG-831d4d4fee5b
type: log
title: "Baseline measured with GIT_TRACE at an absolute path, counting 'trace: built-in:' lines, never"
created: 2026-10-16T06:20:32Z
author: claude-code/opus-5.5+5395
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5395fef418b7
seq: 1
schema: 4
version: 1
---

 timed: ank check --json on a drifting_repo fixture with HEAD (feature) and default branch (main) apart starts 18 git processes with the binary built from 409bb93c, 3 runs, 18 each; the current tree before the change also 18. Six of them are 'cat-file -p <rev>:.ank/{entities,tasks,adr}', three per revision (main and the merge base). Target with one cat-file --batch per revision: 14. ls-tree is not in the ADR-9307 plumbing list, so the listing is one cat-file --batch fed the three tree names, parsing git's raw tree format.
