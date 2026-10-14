---
id: LOG-f913ec1d367e
type: log
title: "drift audit 2026-10-14, measured on c859d01f: ank check exits 0 with 715 signals and 0 faults on"
created: 2026-10-14T09:36:06Z
author: claude-code/fable-5.1
scope:
  - crates/ank-core/**
  - crates/ank-cli/**
  - docs/**
about: ADR-3877fef1d662
seq: 0
schema: 4
version: 1
---

 1145 hot entities. By kind: 348 'written by an agent and read by no human', 160 'verifier cargo-test changed since the proof' (every proof before the nextest switch, ADR-8c12a38531e3), 113 dead scopes, 18 'ratified by its own author', 12 over-constrained, 10 burst creations. One signal per entity for the reading rule is 30 percent of the corpus, which is the volume ADR-1e6bcbf62e61 names as what teaches a reader to stop reading check. Needs a decision on aggregating per corpus.
