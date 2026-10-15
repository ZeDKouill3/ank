---
id: LOG-4b3c5220cfbc
type: log
title: "Measured on a lab corpus with the built binary: claim leaves the claim ref AND rewrites the task"
created: 2026-10-15T12:37:21Z
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
seq: 3
schema: 4
version: 1
---

 file (version 1 -> 2), release removes the ref and writes a LOG entry plus a version bump, read adds verified: and a version, config edits config.yml, and check/status/context/show/tui --json leave nothing with no claim held by the caller. So claim declares both, not refs as its summary suggests.
