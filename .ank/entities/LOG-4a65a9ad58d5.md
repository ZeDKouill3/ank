---
id: LOG-4a65a9ad58d5
type: log
title: "drift audit 2026-10-14, measured on c859d01f: check reports 12 over-constrained scopes, the largest"
created: 2026-10-14T09:36:05Z
author: claude-code/fable-5.1
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
about: ADR-6e09f8a67540
seq: 1
schema: 4
version: 1
---

 at 49638 characters of constraint against a limit of 4000 (18745 on 2026-08-23 when this was decided). ank context crates/ank-cli/src/human.rs --json serves 26 constraints, 28617 characters. The pricing at creation did not stop the growth: 2.6x in seven weeks. Needs a decision on the perimeter practice, not on the limit.
