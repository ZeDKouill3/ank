---
id: LOG-e7eca8dbfce2
type: log
title: "Floor at five decided by the human (Sean) over the literal threshold: fold only when a kind fires"
created: 2026-10-15T20:16:49Z
author: claude-code/opus-5.5+9e7a
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/**
about: TASK-9e7a652aa4f9
seq: 4
schema: 4
version: 1
---

 on more than max(min(20, floor(5% hot_files)), 5) entities, and only on signals whose subject is an entity id (corpus, allowed_signers, coordination, ref names never fold). Measured on ank-cli: literal threshold 80 failing tests of 1077, with the floor 10, each a contract change (kind on findings, --kind flag, goldens, docs/integrating.md help and json blocks) or a real fold (40 rewritten entries, 12 decisions in review's count 14 -> 3). Mutation: removing .max(FOLD_NAMES) turns 2 of the 3 new unit tests red. status_answers_a_thousand_entities_in_under_a_quarter_second failed once under the full ank-cli run and passes alone (5.3s). ank-contract, ank-mcp, ank-tui: 351/351.
