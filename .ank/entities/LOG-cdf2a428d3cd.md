---
id: LOG-cdf2a428d3cd
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:22Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
  - docs/**
about: ADR-8bd76e8d7c4e
seq: 3
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: The absences (no browser reader, no viewer/ directory, no HTML page, no task producing one) are asserted by no test; that the contract is documented in docs/integrating.md is held only by the replay of its examples; "reaches the corpus only via --json" is implied by the dependency graph, never asserted on argv.
