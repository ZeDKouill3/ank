---
id: LOG-be58902ba3e0
type: log
title: "released: Everything in the criterion is done and measured except CI green on all three platforms:"
created: 2026-10-13T13:23:57Z
author: claude-code/2.x
scope:
  - .ank/config.yml
  - .github/workflows/ci.yml
  - CONTRIBUTING.md
  - crates/ank-tui/tests/**
  - .config/nextest.toml
about: TASK-dbbf4e6316bb
seq: 6
schema: 4
version: 1
---

 windows-latest fails two doc_replay tests under nextest, deterministically (run 38398699531, twice), not reproducible locally. Filed TASK-d08e4110f6c1 (diagnose) and made this task wait on it. The work stays on branch task/nextest, PR #577, open and red.
