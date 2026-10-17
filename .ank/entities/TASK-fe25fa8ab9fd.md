---
id: TASK-fe25fa8ab9fd
type: task
slug: no-tracked-file-outside-ank-cites-adr-2a63c0c876
title: No tracked file outside .ank/ cites ADR-2a63c0c87691 any more, so its supersession can land
created: 2026-10-17T14:35:02Z
author: haksolot@omarchy
status: done
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
proof:
  - type: test
    ref: local/e3b0c44298fc@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/44ca270bf8c3@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-core@11c0192e3520
    via: verifier
  - type: test
    ref: local/023c770a8dfc@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
  - type: test
    ref: local/de5e5c48a8b9@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
  - type: test
    ref: local/d3638ae6a106@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/03b7ace55901@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/f772f8db7da1@75f0570
    tree: scope/12582d132709
    criteria: bc75def15e31
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

accept refuses a supersession while any tracked file outside .ank/ still
cites the document it retires (ADR-3b6ba766a42e), names every site with its
line, and takes no bypass. A citation of the proposed successor is not a
defect. This is the task that makes the ratification of the successor
possible: do it before, not after.
