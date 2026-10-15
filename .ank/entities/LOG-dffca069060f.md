---
id: LOG-dffca069060f
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:32Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-core/**
  - crates/ank-cli/**
  - docs/**
about: ADR-67a4ac10c534
seq: 1
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "The log is never proof, nothing authoritative is anchored in it, no hash chains over it" is held by no test: deleting_every_machinery_entry_changes_no_answer keeps the work entry, so an anchor on a work entry would turn nothing red.
