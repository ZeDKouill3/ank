---
id: TASK-933fdb42c7d9
type: task
slug: the-last-row-of-every-frame-is-a-line-of-keys-an
title: The last row of every frame is a line of keys, and every entry and every panel title is a touch target
created: 2026-10-16T08:19:02Z
author: claude-code/opus-5.5+keyline
status: open
scope:
  - crates/ank-tui/src/**
  - crates/ank-tui/tests/**
  - crates/ank-cli/tests/tui.rs
blocked_by: []
done_criteria: |
  Through the binary in a pseudo-terminal, at 80x24 and at 160x45 (ADR-ac6be1ebe9aa, "The keys are always on the screen"): the last row of every frame -- with each panel focused, with the key list open, with the list x opens, with a form open, with a search open, and with a command waiting -- is a line of keys computed from the key table and filtered by what the screen holds, so its entries change with the focused panel. Wherever ? and q do what they name (no command waiting, no search open, no form open) the line begins with the key that opens the key list and ends with the key that quits, and where the width cannot hold it whole it drops entries from the middle and keeps those two; at 160x45 the entities line is drawn whole. A tap on any entry of that line does what pressing its key does: on [q quit] the session ends, on [? keys] the key list opens, on an entry of the middle its key runs. A tap on a panel title does what pressing the digit that title names does, at both sizes. The frame spends at most five rows on chrome at 80x24 (tests/chrome.rs rewritten against this decision).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 1
---
