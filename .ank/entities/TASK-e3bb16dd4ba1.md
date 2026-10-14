---
id: TASK-e3bb16dd4ba1
type: task
slug: an-adr-carries-enforced-by-amend-writes-it-on-an
title: An ADR carries enforced_by, amend writes it on an accepted ADR, check confronts it with the tree, and context serves the rule by its title
created: 2026-10-14T10:17:25Z
author: claude-code/fable-5.1+plan
status: open
scope:
  - crates/ank-core/**
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/**
  - docs/format.md
  - docs/entity-fields.md
blocked_by: []
done_criteria: |
  Through the binary, per ADR-25584c56c1a7: ank new adr --enforced-by crates/x/tests/a.rs#name writes enforced_by and ank show prints it; ank amend --enforced-by and --drop-enforced-by write it on a proposed ADR and on an accepted one, and check reports no altered-since-ratification fault afterwards; check signals an entry whose path matches no tracked file and one whose symbol the file does not contain, naming the entry; ank context <path> in orientation mode and in execution mode prints an ADR carrying enforced_by as one line and its constraint text nowhere, and the over-constrained charge of a task covering it no longer counts it, measured by the charge before and after; a golden fixture under crates/ank-core/tests/golden/valid/ carries the field, docs/format.md describes it, docs/entity-fields.md is regenerated, and SCHEMA_VERSION stays 4.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

The pattern is the one amends (eb15140e) and method (a81d4bd2) followed: an opt() row in registry.rs at its canonical position, the model and parse fields, an emit arm that omits the field when empty, a golden fixture, docs/format.md, docs/entity-fields.md regenerated. The anchor covers constraint and scope only (ratification_anchor in human.rs), which is what lets amend write the field on an accepted ADR.
