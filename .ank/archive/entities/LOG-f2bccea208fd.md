---
id: LOG-f2bccea208fd
type: log
title: "Red before the change, through the binary (ank watch --once): in a clone carrying"
created: 2026-10-12T12:50:41Z
author: claude-code/opus-5.5-refmap
scope:
  - crates/ank-daemon/src/fetch.rs
  - crates/ank-cli/tests/watch.rs
about: TASK-625d3bf0c9a1
seq: 1
schema: 4
version: 1
---

 +refs/ank/*:refs/ank/*, the cycle rewrote the clone's own claim on TASK-4f1e... from 4092041d to origin's forged ef3adcc0, and brought refs/ank/claims/TASK-0000000000ff onto the local plane. Fix: --refmap= (empty) on the mirror fetch. After: the new test and the 29 other watch.rs tests green, ank-daemon crate green.
