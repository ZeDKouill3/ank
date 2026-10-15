---
id: LOG-997c7ae3b2d7
type: log
title: ank verify is implemented and its targeted suites are green (verify_verb 6/6, writes, golden,
created: 2026-10-15T17:02:26Z
author: claude-code/opus-5.5+15b5
scope:
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/src/done.rs
about: TASK-15b59a393a38
seq: 1
schema: 4
version: 1
---

 schema, cli unit 369/369; a mutation selecting every verifier reddens 3 of the 6 verify_verb tests). Two skill.rs tests are red by design: every_dispatched_verb_is_listed_in_section_4 and help_prints_section_4s_order_inside_every_group read the ratified SPEC-14cd Commands block, which does not list verify, and SPEC-14cd says the declaration lands and is signed before the dispatch. The close needs a superseding SPEC-14cd carrying 'ank verify' before 'ank done', ratified, plus the citation re-point of SPEC-14cd in verbs.rs, view.rs, skill.rs and docs/specification.md.
