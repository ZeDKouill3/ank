---
id: LOG-03b6f5dbd8e9
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:15Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
about: ADR-9e56318631f3
seq: 1
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: No two-root test holds "the corpus's drift resolves against the corpus" or "every entity path handed to git resolves against the corpus"; "never derived from a working directory" is held only implicitly by tests running from a temp dir.
