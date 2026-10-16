---
id: LOG-aa93d1334d0b
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:14Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - crates/ank-contract/**
  - skill/**
  - docs/**
about: ADR-a8f9c603a0e7
seq: 2
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: No test runs check on a task whose designated method never fired, so "check never faults a task for a method that did not fire" is held by no test; "no verifier inspects the route" and "sent nowhere" are negative design rules no test can hold.
