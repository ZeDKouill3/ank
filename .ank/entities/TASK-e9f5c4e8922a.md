---
id: TASK-e9f5c4e8922a
type: task
slug: the-suite-is-split-into-one-verifier-per-crate-e
title: The suite is split into one verifier per crate, each declaring its inputs
created: 2026-10-13T10:11:07Z
author: claude-code/2.x
status: done
scope:
  - .ank/config.yml
  - crates/ank-cli/tests/**
blocked_by: [TASK-dc04757dfecb, TASK-dbbf4e6316bb]
done_criteria: |
  config.yml declares one default test verifier per workspace crate, each running that crate's tests and declaring as inputs its crate, every workspace crate it depends on, Cargo.toml and Cargo.lock; fmt-check declares the Rust sources and rustfmt.toml; a test reads config.yml and cargo metadata and fails when a verifier's inputs omit a workspace dependency of its crate; ank new task with no flag lands with the per-crate verifiers.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/0e80c545fad9@60a5ad0
    tree: scope/cdd1e11e8012
    criteria: 2ceb74df4176
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@60a5ad0
    tree: scope/cdd1e11e8012
    criteria: 2ceb74df4176
    verifier: fmt-check@695ca51e4ab9
    via: verifier
schema: 4
version: 3
---

Dogfoods the mechanism of the product track: with reuse in place, a task that
touches only ank-tui re-runs the ank-tui verifier and reuses the rest. The
test against cargo metadata keeps an `inputs` from going stale when a crate
gains a dependency, which is the false-reuse risk ADR-a2faf59f6ce0 names.
