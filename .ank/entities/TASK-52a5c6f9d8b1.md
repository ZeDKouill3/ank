---
id: TASK-52a5c6f9d8b1
type: task
slug: the-file-format-carries-anchor-and-the-goldens-a
title: The file format carries anchor, and the goldens and the format page agree on it
created: 2026-10-17T14:35:03Z
author: haksolot@omarchy
status: open
scope:
  - docs/format.md
  - docs/entity-fields.md
  - crates/ank-core/src/**
  - crates/ank-core/tests/golden/**
blocked_by: [TASK-5f7b49eaca61]
done_criteria: |
  ank-core parses and writes an optional anchor field on an adr and on a spec, holding a git object name, and leaves an entity without it untouched byte for byte; a golden carrying anchor and one without both round-trip. docs/format.md's 'What lives outside the files' and 'The two hashes' describe the tag object, the ref, the plumbing recipe that reproduces a ratification without ank (hash-object, tag, update-ref, tag -d) and a worked vector taken from a real accept once ADR-5e47007a75be's accept exists, or from the recipe until then. docs/entity-fields.md lists anchor. cargo nextest run -p ank-core is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 1
---

The field is optional and no schema bump is needed: an older reader ignores
it, a legacy anchor has none. `ratified:` keeps the hash and its meaning.
