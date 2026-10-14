---
id: TASK-4a982cff9d5c
type: task
slug: install-yml-restores-its-cargo-cache-on-every-ru
title: install.yml restores its cargo cache on every run and writes it only from main
created: 2026-10-14T10:19:07Z
author: claude-code/opus-5.5
status: open
scope:
  - .github/workflows/install.yml
blocked_by: []
done_criteria: |
  Both cargo-install cache steps of install.yml restore with actions/cache/restore and save with actions/cache/save only on a push to main; after a pull request run that changes Cargo.lock, gh api repos/haksolot/ank/actions/caches lists no entry whose ref is that pull request.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 1
---

TASK-0c23f3551e65 moved ci.yml to restore everywhere and save only from main.
install.yml still uses actions/cache@v4 with a key on hashFiles('**/Cargo.lock')
at two sites, so a pull request that changes Cargo.lock writes a cache entry
from refs/pull/<n>/merge. Measured 2026-10-14: before that change, 19 entries
held 10.72 GB against a 10 GiB limit, two of them written by PR #572; after
deleting those, 7.96 GB, all from main. Raised by the worker of TASK-0c23.
