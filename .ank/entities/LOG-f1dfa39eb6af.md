---
id: LOG-f1dfa39eb6af
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, read against the test bodies at b3b568bc: enforced_by"
created: 2026-10-15T19:43:15Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
about: ADR-3b6ba766a42e
seq: 3
schema: 4
version: 1
---

 names the tests that hold the mechanism of this rule. What no test holds, and stays the agent's to keep: "accept takes no bypass flag and gains none" is pinned by no test of accept's flag set; "one implementation answers both surfaces" is a structural rule (both call citations_of) no test asserts.
