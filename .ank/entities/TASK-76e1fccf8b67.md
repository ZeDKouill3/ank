---
id: TASK-76e1fccf8b67
type: task
slug: live-until-hands-back-a-screen-only-once-its-pre
title: Live::until hands back a screen only once its predicate holds on two equal samples in a row
created: 2026-10-19T07:25:31Z
author: claude-fixture@vmi
status: open
scope:
  - crates/ank-tui/tests/terminal/mod.rs
blocked_by: []
done_criteria: |
  In crates/ank-tui/tests/terminal/mod.rs, Live::until returns a screen only when its predicate holds on it and that screen equals the sample before it, the pulse masked (same_but_the_pulse), so no caller can assert on a frame the terminal had only half handed over. Before the change, the number of frames the old until returned that changed on the very next sample is counted over at least three runs of the ank-tui suite and logged on this task; after it, the ank-tui suite runs green at least five times in a row on Linux, the count logged, and one run of all three platforms is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Measured on 2026-10-19 under `ank done`, which runs test-ank-tui beside test-ank-cli:
`graph::a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet` panicked at
graph.rs:1105 with 8 rows against 11. Its `until` accepted the first frame where the
title, the count and the cursor were drawn, before the rows under them had arrived.

It is one shape seen three times. TASK-8be089554bb1 (#646) hit it on macos-latest in
`the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing` (nine rows, the last
two the listing's own) and fixed that test alone with an `at_rest` helper. TASK-aeea26cac0f9
names it in `on_the_graph_f_is_named_by_the_fold_it_walks` (the key line not rewritten yet).
Fixing the tests one at a time leaves the other 240-odd `until` callers with the same flaw.

A torn frame lasts between two reads of the drain and never across two samples (the argument
of #646). So the fix goes in the one function every caller goes through: a frame satisfying
the predicate is returned only once the next sample is the same frame. The cost is one 25ms
sample per wait.

This does not close TASK-aeea26cac0f9, whose criterion asks for its own reproduction and
300 runs; it removes the mechanism that task names.
