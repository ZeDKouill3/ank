---
id: LOG-6d2b66daf2d8
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:16Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - docs/**
about: ADR-c23bef1cc93e
seq: 4
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "Every verb that reads an identifier accepts the same form" is tested for new, amend, show, find, log, claim and context only, not surface-wide; "claims stay per repository" is not asserted (the snapshots cover the peer's .ank/, not its refs).
