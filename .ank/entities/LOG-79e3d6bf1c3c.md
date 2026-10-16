---
id: LOG-79e3d6bf1c3c
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:19Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - docs/**
about: ADR-3094538d831e
seq: 1
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: No fixture sits under the similarity threshold, so "a move under it stays a fault" is held by no test; "never repairs, no scope edited automatically" is not asserted on the entity bytes; "runs on a dead scope and nothing else" has no zero-dead-scope control; "a path in a body is prose" has no test.
