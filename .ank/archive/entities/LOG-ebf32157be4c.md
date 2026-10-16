---
id: LOG-ebf32157be4c
type: log
title: Measured on a lab clone of this repository at 4633129a, old binary (PATH ank, 280e2f9b, reads
created: 2026-10-15T21:40:18Z
author: claude-code/opus-5.5+3a7a
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - .ank/config.yml
  - .ank/allowed_signers
  - docs/signing.md
  - crates/ank-cli/tests/skill.rs
about: TASK-3a7a73ccf2b7
seq: 3
schema: 4
version: 1
---

 .ank/allowed_signers) against the new one (reads signers: after ank signers migrate). Real keys: old 0 faults, 0 signature faults, 49 ratification signatures unchecked (this box has no gpg, so every gpg-signed one is E); new after migrate and commit: 0 faults, 0 signature faults, the same 49 unchecked. Decoy probe (only a key nobody holds declared, so every ratification judged becomes a fault): old 46 signature faults, new 46, and the 46 subjects are the same set (diff empty). 46 judged + 49 unchecked = 95 accepted ratified entities, all reached by both. Before the commit of the deletion, two dead-scope faults appear (SPEC-77d99d8d1ef2, TASK-313165e55d81 scope .ank/allowed_signers); once the deletion is committed git records it and they vanish, leaving one scope-not-yet signal on this task. migrate on the real file: 4 keys, 1 principal, the 31 comment lines kept as the gpg key's note, 0 bytes above the block changed (cmp of the prefix).
