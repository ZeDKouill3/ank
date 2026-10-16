---
id: LOG-54aec1c5c468
type: log
title: "Measured: ank check on a scratch corpus of 190 open tasks scoped src/** and 30 ACCEPTED ADRs whose"
created: 2026-10-15T19:43:52Z
author: claude-code/opus-5.5+9e7a
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/**
about: TASK-9e7a652aa4f9
seq: 2
schema: 4
version: 1
---

 scopes match only gitignored files takes 14s user CPU (release ank on PATH, 19-20s wall); the same 30 ADRs as proposed take 0.23s, and the 30 accepted ADRs alone 0.77s, with 21 git processes either way (GIT_TRACE). The cost is CPU and grows with tasks x accepted dead ADRs; it predates this task. The dead-scope fold fixture uses proposed ADRs, which signal the same dead scope.
