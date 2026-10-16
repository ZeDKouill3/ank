---
id: LOG-845eea163d1f
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, at b3b568bc: no enforced_by declared, and the constraint"
created: 2026-10-15T19:43:54Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
about: ADR-f8a6cf65160e
seq: 0
schema: 4
version: 1
---

 stays served whole. It is a code-structure rule: every entity path handed to git derives from the store through entity_rel_paths and entity_rel_paths_to_stage, never from a literal, and the memo is keyed by every input. No test exercises a git lookup or a staging under the previous layout, and the CI grep the ADR mentions does not exist, so an edit to human.rs could break it with nothing turning red.
