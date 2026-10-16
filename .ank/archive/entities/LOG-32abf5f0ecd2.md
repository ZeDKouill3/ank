---
id: LOG-32abf5f0ecd2
type: log
title: "Stopped before ank done: cargo-nextest is not installed on this box, and the cargo-test verifier"
created: 2026-10-15T12:43:55Z
author: claude-code/opus-5.5+0766
scope:
  - crates/ank-cli/src/update.rs
about: TASK-0766190c03bf
seq: 3
schema: 4
version: 1
---

 requires it (config.yml exits 127 without it). Not installed, per the orchestrator's instruction. The plain cargo test --workspace run was green except the help.json golden, which was blessed for the new update refusal; the update suite is 18/18 after the bless.
