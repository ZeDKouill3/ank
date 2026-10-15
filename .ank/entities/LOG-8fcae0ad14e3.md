---
id: LOG-8fcae0ad14e3
type: log
title: Measured through the binary (adr_enforced_by::the_over_constrained_charge_stops_counting_an_enforced
created: 2026-10-15T18:52:37Z
author: claude-code/opus-5.5+e3bb
scope:
  - crates/ank-core/**
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/**
  - docs/format.md
  - docs/entity-fields.md
about: TASK-e3bb16dd4ba1
seq: 1
schema: 4
version: 1
---

_rule, context_budget 100, limit 50): before, check charged the held ADR 71 characters and the loose one 74, total 145; after ank amend <held> --enforced-by src/tests.rs#logs_every_module on the accepted ADR, the held one is absent from the breakdown and the total is 74. The claim record still hashes both texts: only the charge and what context serves change. JSON: constraint stays a string (ADR-6fd69efb629c forbids retyping it), empty for an enforced rule, beside a new enforced_by array. The relief note on an accepted heaviest ADR now names ank amend <id> --enforced-by, since the field is the one amend outside the anchor; cli.rs's assertion that no 'ank amend ADR-' is named was narrowed to that command. Symbol matched as a whole word: src/tests.rs#logs_every is reported against a file holding logs_every_module.
