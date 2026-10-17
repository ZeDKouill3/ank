---
id: TASK-fe25fa8ab9fd
type: task
slug: no-tracked-file-outside-ank-cites-adr-2a63c0c876
title: No tracked file outside .ank/ cites ADR-2a63c0c87691 any more, so its supersession can land
created: 2026-10-17T14:35:02Z
author: haksolot@omarchy
status: open
scope:
  - docs/**
  - CLAUDE.md
  - CONTRIBUTING.md
  - skill/**
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  git grep -l ADR-2a63c0c87691 -- ':!.ank' prints nothing: every former site either cites ADR-5e47007a75be, the proposed successor, or drops the citation and leaves the history to ank show, and no sentence left behind describes --pr as anything but the recipe the tree still runs today. ank check exits 0.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

accept refuses a supersession while any tracked file outside .ank/ still
cites the document it retires (ADR-3b6ba766a42e), names every site with its
line, and takes no bypass. A citation of the proposed successor is not a
defect. This is the task that makes the ratification of the successor
possible: do it before, not after.
