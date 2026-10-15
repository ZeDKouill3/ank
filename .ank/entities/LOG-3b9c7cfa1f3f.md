---
id: LOG-3b9c7cfa1f3f
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:34Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
  - install.sh
  - install.ps1
  - .github/workflows/**
  - skill/**
  - docs/**
about: ADR-e1d750884b82
seq: 1
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "Nothing under skill/ is committed a second time" has no test or CI step; "no network is attempted" is not asserted; that npx's output is the reason of a failure is not asserted, only its exit code; a build with no skill/ is exercised only through the empty-list function.
