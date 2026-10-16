---
id: LOG-7dbebc4295f2
type: log
title: help --json is rendered in crates/ank-cli/src/cli.rs (verb_json), outside the declared scope;
created: 2026-10-15T12:30:13Z
author: claude-code/opus-5.5+e72b
scope:
  - crates/ank-contract/src/**
  - crates/ank-cli/tests/**
  - SECURITY.md
  - docs/conventions.md
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/paint.rs
  - crates/ank-cli/src/cli.rs
about: TASK-e72b519aeda9
seq: 2
schema: 4
version: 1
---

 amended the scope to include it. Reading taken for writes: it declares what the verb itself leaves under .ank/ and refs/ank/, and a lease renewal is not counted, because renews already declares it per verb; the measurement runs Nothing verbs with no claim held by the caller (another agent holding one plus an orphan claim ref, so a prune would show), and the Renews::Never ones again under a held claim.
