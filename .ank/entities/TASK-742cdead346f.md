---
id: TASK-742cdead346f
type: task
slug: the-contract-summary-of-tui-names-the-graph-and
title: The contract summary of tui names the graph, and the golden fixtures move with it
created: 2026-10-14T09:16:49Z
author: ankwork
status: open
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
blocked_by: [TASK-643611645dea]
done_criteria: |
  The summary of the tui verb in crates/ank-contract/src/verbs.rs names the blocked_by graph among what the reader shows, in the same sentence shape it has today; `ank help tui` and `ank help --json` print the new summary through the binary, and the golden fixtures under crates/ank-cli/tests/golden-json that pin them are regenerated so that the golden suite is green with no shape change: no field is added, removed, renamed or retyped and the contract version does not move. `cargo test --workspace` is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Housekeeping the contract owes the screen. ADR-6fd69efb629c makes `ank help --json` the one description of every verb, generated from one table and pinned by a golden fixture, so the sentence that says what `ank tui` shows lives there and nowhere else -- and after the GRAPH screen lands it is short one thing. A summary narrower than the binary is read as the binary refusing, which the note beside the tui entry already says about its own history.

Small on purpose, and apart from the view: the only files it touches are the table and the fixtures that pin it, so it can run beside the fold task without the two meeting in a file. The docs under docs/ name no screen of the reader today (measured with grep on 2026-10-14), so there is no page to move.
