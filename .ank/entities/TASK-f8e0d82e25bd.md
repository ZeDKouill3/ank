---
id: TASK-f8e0d82e25bd
type: task
slug: the-key-line-on-graph-names-f-by-the-fold-it-wal
title: The key line on GRAPH names f by the fold it walks there, not by kind
created: 2026-10-17T09:28:34Z
author: haksolot@omarchy
status: open
scope:
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/tests/keyline.rs
blocked_by: []
done_criteria: |
  Through the binary in a pseudo-terminal, at 160x45 and at 80x24: with the focus on GRAPH, the key line's entry for f reads [f fold] and the frame contains no [f kind]; with the focus on the tasks listing, the same entry still reads [f kind]. The word fold is declared once in the key table and the key line reads it there. A test in crates/ank-tui/tests/keyline.rs asserts all of this and fails on the tree before the fix.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Seen in the review of PR #621: on GRAPH, f walks the fold (open, then all), and the header cell says [f open] or [f all]. But the key line still labels the key [f kind], the word of its binding in bindings.rs. ADR-ac6be1ebe9aa wants the key line computed from what the focused panel offers, and on GRAPH the key does not offer kind. The full key list already says it right (Binding::entry: f kind; open or all on GRAPH), so only the key line is wrong.
