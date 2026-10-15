---
id: LOG-20f2b3c942ce
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:33Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-core/src/freeze.rs
  - crates/ank-cli/**
about: ADR-6b3f19e08a24
seq: 2
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: The three named anchors are each held; the universal "no frozen field relies on the CLI refusing a write" is not, since no test enumerates the frozen fields.
