---
id: LOG-5c00bcc13bd7
type: log
title: "Outside the declared scope, unavoidably: an Adr literal gains enforced_by in the unit-test fixtures"
created: 2026-10-15T18:52:40Z
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
seq: 2
schema: 4
version: 1
---

 of claim.rs and index.rs, and edit.rs changed_fields notes the field (plus its fixture), since a new struct field does not compile otherwise; amends (eb15140e) touched the same three.
