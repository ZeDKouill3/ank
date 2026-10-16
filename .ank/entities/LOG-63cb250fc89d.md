---
id: LOG-63cb250fc89d
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:36Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
  - docs/**
about: ADR-96174f1ac2b7
seq: 1
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "Nothing walks a filesystem looking for a corpus" is tested on the no-fallback case only; "writing never crosses a corpus boundary" is held for peers, not for a declared corpus and its refs; "the map is held outside the repository" is never asserted.
