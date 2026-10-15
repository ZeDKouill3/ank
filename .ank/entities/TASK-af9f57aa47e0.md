---
id: TASK-af9f57aa47e0
type: task
slug: the-documentation-says-how-to-split-a-suite-into
title: The documentation says how to split a suite into verifiers that reuse and select
created: 2026-10-13T12:02:36Z
author: claude-code/2.x
status: done
scope:
  - docs/proof.md
  - docs/SUMMARY.md
blocked_by: [TASK-15b59a393a38]
done_criteria: |
  The documentation site has a page, linked from SUMMARY.md and from proof.md, that shows a repository with two packages declaring one verifier each with disjoint inputs, the output of ank verify after a change to one package, and the done that follows reusing the other; it states that done reuses by content and verify selects by change and proves nothing; every output it shows is measured by the suite (ADR-2b62b9a1fe67).
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/94a7c74309a9@18a5f92
    tree: scope/74aa24e3d7d2
    criteria: 1495acb07dd9
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@18a5f92
    tree: scope/74aa24e3d7d2
    criteria: 1495acb07dd9
    verifier: fmt-check@695ca51e4ab9
    via: verifier
schema: 4
version: 3
---

The user-facing half of the plan: a reproducible recipe for any language, so a
team adopting ank can cut the cost of done in its own repository without
reading the ADRs.
