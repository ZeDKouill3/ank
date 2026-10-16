---
id: LOG-5332ee9bc9d1
type: log
title: "Scope amended to .config/nextest.toml and both Cargo.toml: autotests = false plus one [[test]]"
created: 2026-10-14T08:00:08Z
author: claude-code/2.1.299
scope:
  - crates/ank-cli/tests/**
  - crates/ank-tui/tests/**
  - .config/nextest.toml
  - crates/ank-cli/Cargo.toml
  - crates/ank-tui/Cargo.toml
about: TASK-22af5b0918a4
seq: 3
schema: 4
version: 1
---

 target lives in the manifests, and the pty-serial group filters by binary() names this change removes.
