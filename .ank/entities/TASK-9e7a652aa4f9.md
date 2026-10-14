---
id: TASK-9e7a652aa4f9
type: task
slug: a-finding-carries-a-kind-and-check-folds-a-kind
title: A finding carries a kind, and check folds a kind past the threshold into one corpus signal
created: 2026-10-14T10:17:24Z
author: claude-code/fable-5.1+plan
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Through the binary, per ADR-df1035934387: on a scratch corpus of 30 tasks written by an agent and read by no human, ank check prints one signal with subject corpus carrying the kind, the count 30, five identifiers and the command that lists the rest, and no per-entity line of that kind; the same corpus with 10 such tasks prints 10 per-entity lines; the fold applies on the same terms to the verifier-changed and dead-scope kinds, each measured on its own fixture; 30 faults of one kind are printed one by one; ank check --json carries kind on every finding and the golden help and check fixtures carry the field; review and status count a folded signal as one.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Measured on 2026-10-14 (c859d01f): 715 signals, of which 348 read-by-no-human, 160 verifier-changed and 113 dead-scope lines, each one per entity. The three hand-written corpus folds in human.rs (prose identifiers, pre-convention authors, cold entities) are the shape to generalise; Finding gains kind, and one fold applies to every kind.
