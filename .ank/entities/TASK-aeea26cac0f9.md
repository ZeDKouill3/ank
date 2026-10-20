---
id: TASK-aeea26cac0f9
type: task
slug: the-keyline-test-on-graph-waits-for-the-key-line
title: The keyline test on GRAPH waits for the key line to be rewritten before it reads it
created: 2026-10-17T21:54:03Z
author: haksolot@omarchy
status: done
scope:
  - crates/ank-tui/tests/keyline.rs
  - crates/ank-tui/tests/terminal/mod.rs
blocked_by: []
done_criteria: |
  In crates/ank-tui/tests/keyline.rs, on_the_graph_f_is_named_by_the_fold_it_walks settles on a frame whose last row has been repainted for the GRAPH focus before it asserts: the settling predicate names the key line's expected content (or a repaint counter the harness exposes), never only the focused panel, so a frame with 3 GRAPH focused and the key line still reading [f kind] is not accepted. The cause is named in a log on this task from a reproduction (the test looped under load until red, with the frame it saw). That test, run at least 300 times on Linux after the change, fails zero times, and the count is logged; one run on all three platforms is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/14e5f4e3f780@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/ed930b31e5c7@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/db07bfa15a78@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/56d8935b2e0e@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/47638288d909@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
    reused: local
    by: claude-code/aeea
  - type: test
    ref: local/2ec522963efa@42391ba
    tree: scope/fd3ad1dc0f6c
    criteria: 600302e15fac
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

Seen on ubuntu-latest for PR #640 (run 39071756405, job 117510875009, a log-only commit over a head that had passed ubuntu minutes before): keyline::on_the_graph_f_is_named_by_the_fold_it_walks panicked at keyline.rs:350 with the 160x45 frame showing 3 GRAPH focused while the key line still read [f kind]. TASK-adea229723df's worker reproduced the same shape on Linux (2 red of 100 alone, red at run 130 of a third loop) and named the mechanism: until() matched a frame whose last row had not been rewritten yet. TASK-f8e0d82e25bd made f named by the fold; this is its test sampling one frame too early. Read but not reproduced here.
