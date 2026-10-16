---
id: LOG-15ec41ba36ec
type: log
title: "Measured 2026-10-16 in a pty (160x45, ank-tui probe through the binary): the form's Control-U is"
created: 2026-10-16T09:45:50Z
author: claude-code/opus-5.5+ac6b-gaps
scope:
  - crates/ank-tui/**
about: ADR-ac6be1ebe9aa
seq: 1
schema: 4
version: 1
---

 not a command that requires a chord. With --title typed 'abc' in the edit form, three Backspaces (0x7f, no modifier) left the field empty, the same state Control-U reaches; Escape closes the form as Control-C does. Control-U is a second road to a state a bare key reaches, which is exactly the shape keys::narrowing already has for the search, and form.rs's own press() says so. A form field is not outside the rule -- it is inside it and satisfies it. No task: 'no command requires a modifier chord' holds for the form as built.
