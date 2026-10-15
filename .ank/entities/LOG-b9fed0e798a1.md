---
id: LOG-b9fed0e798a1
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:35Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/src/**
  - install.sh
  - install.ps1
  - npm/**
about: ADR-64f32c74a0f9
seq: 2
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "Never downloads or unpacks an archive itself" is pinned by no dependency or no-HTTP test; "no other verb checks for or announces a newer release" is held for status and context only.
