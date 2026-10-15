---
id: LOG-29e222dbb3da
type: log
title: "ank done refused at exit 5: cli::every_argument_on_the_surface_is_classified_as_carrying_a_path_or_n"
created: 2026-10-15T18:59:01Z
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
seq: 3
schema: 4
version: 1
---

ot was red, since --enforced-by carries a path and was classified nowhere; the targeted runs had not included it. Fixed rather than classified away: the path half of an entry now goes through context::normalised on new --enforced-by, amend --enforced-by and --drop-enforced-by (./src/tests.rs#f and src\main.rs#main stored as src/...), the three flags are in PATH_FLAGS, and adr_enforced_by::the_path_of_an_entry_is_normalised_as_a_scope_is was red before the fix.
