---
id: TASK-a77dca75f8e3
type: task
slug: an-identifier-is-painted-by-its-kind-done-recede
title: An identifier is painted by its kind, done recedes to grey and closed reads red
created: 2026-10-14T09:50:36Z
author: clausse@macbook-air-de-clausse
status: done
scope:
  - crates/ank-contract/src/meaning.rs
  - crates/ank-cli/src/style.rs
  - crates/ank-tui/src/paint.rs
blocked_by: []
done_criteria: |
  A proposed spec superseding SPEC-89070ce7f3b8 carries the new palette rows, so the code and the specification agree once both are ratified
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/554376868ce0@80aad14
    tree: scope/9fa666e71784
    criteria: 71a2a423f612
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@80aad14
    tree: scope/9fa666e71784
    criteria: 71a2a423f612
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Implements ADR-f0665c68d03d. The palette was chosen on a preview build: ADR in a soft orange (173) because every base colour is already spent by a status, and accepted kept green while done recedes.
