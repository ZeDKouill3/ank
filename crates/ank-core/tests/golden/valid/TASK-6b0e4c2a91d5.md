---
id: TASK-6b0e4c2a91d5
type: task
slug: split-the-auth-suite
title: Split the auth suite from the web suite
created: 2026-10-15T09:12:00Z
author: claude-code/opus-5
status: done
scope:
  - crates/auth/**
blocked_by: []
done_criteria: |
  The auth suite passes on its own.
criteria_by: creator
verify: [auth-tests, web-tests]
proof:
  - type: test
    ref: local/e51b22c09a4f@a3f9c21
    tree: scope/4be2d10c
    criteria: 9d0c1e7a2b44
    verifier: auth-tests@1f2e3d4c5b6a
    via: verifier
  - type: test
    ref: local/7c09e1b2d3f4@a3f9c21
    tree: scope/4be2d10c
    criteria: 9d0c1e7a2b44
    verifier: web-tests@6a5b4c3d2e1f
    via: verifier
    reused: local
    by: claude-code/opus-5+sibling
schema: 4
version: 3
---

Two entries of one `done`, and only the second carries `reused` and `by`. The
first is an execution for this task. The second is a passing run `done` found in
the registry under the git common directory with the same key -- `web-tests` at
the same definition hash, over the same bytes of its `inputs` -- and took
instead of starting the verifier again. It keeps the reference and the route of
the run that produced it, and `by` names the identity that ran it, here a
sibling worktree of the same repository.
