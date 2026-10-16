---
id: LOG-f31b0cdf80b5
type: log
title: "Measured: ank accept A B --pr in a scratch repo with a bare file:// origin and a gh stub, GIT_TRACE"
created: 2026-10-11T20:19:51Z
author: claude-code/opus-5.5/task-68db
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - docs/ratifying.md
about: TASK-68db19c9ed93
seq: 3
schema: 4
version: 1
---

 at an absolute path: 33 git processes with 5 tracked files and 33 with 305 (the tracked-change check is cat-file --batch-check + hash-object --stdin-paths, two processes whatever the tree), 2 gh invocations (auth status, pr create), git status --porcelain empty afterwards, main back at origin/main. Red observed first for every criterion test (unknown flag --pr, extra argument, the page lacking --pr); the --json several-ids test was written after and green on first run, behaviour already in place. Windows: the gh stub is a Rust program compiled once per run with rustc, so tests/accept_pr.rs is not cfg(unix); on Windows PATH is the inherited one minus dirs holding gh.exe, on Unix only a bin with git linked in. Only run here on Linux: Windows and macOS are verified by CI, not by this session. human::tests::accept_takes_no_bypass_flag narrowed from 'no flag' to 'exactly --pr' (ADR-2a63 adds it; it bypasses nothing). help.json golden re-blessed: accept's diff only.
