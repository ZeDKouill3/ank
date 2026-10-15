---
id: LOG-bc54e0164d7c
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:17Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
about: ADR-4004eb9be5e9
seq: 2
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "Compares against the merge base" is held by no test: none advances main past the branch point, so comparing against main's tip would pass; plumbing-only rests on the debug_assert over PLUMBING in git::output, not on a test.
