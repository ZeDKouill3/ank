---
id: LOG-0325f6becfe3
type: log
title: "Measured through the pty (tests/keyline.rs): 7 tests red before the change, all timing out on 'the"
created: 2026-10-16T09:02:57Z
author: claude-code/opus-5.5+keyline
scope:
  - crates/ank-tui/src/**
  - crates/ank-tui/tests/**
  - crates/ank-cli/tests/tui.rs
about: TASK-933fdb42c7d9
seq: 1
schema: 4
version: 1
---

 session to read its corpus' because no frame carried a key line; green after. Lines measured: 160x45 entities = '[? keys]  [Enter open]  [f kind]  [/ find]  [c claim]  [l log]  [r release]  [d done]  [m amend]  [e edit]  [n new]  [q quit]' (125 cols); 80x24 drops r, l, d, c, m from the middle -> '[? keys]  [Enter open]  [f kind]  [/ find]  [e edit]  [n new]  [q quit]'; body at 110 = '[? keys]  [Space page]  [s rules]  [b back]  [c claim]  [d done]  [m amend]  [e edit]  [n new]  [q quit]'. Modal screens name their own keys and not ? and q, which they read as dismiss, needle or text: '[y run]  [Esc dismiss]', '[Enter keep]  [Esc clear]', '[Enter compose]  [Esc close]' (the form's two rows are new in the table, Offered::Filling). The overlays (?, x, the form) stop above the key line. Collateral: at 110x30 the body page loses a row, the opened task reads 'rows 1-23 of 25', EDITS(1)/LOG-... falls off the page and colour.rs's two LOG-dim tests go red; the fix is WINDOW 110x30 -> 110x32 in colour.rs, another worker's file, left untouched. ank-tui + ank-cli tui under nextest: 335 run, 333 passed, those 2 failed.
