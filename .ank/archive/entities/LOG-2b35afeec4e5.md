---
id: LOG-2b35afeec4e5
type: log
title: "Measured on the wire, not read. ank tui in a pty (tests/colour.rs): TASK- after 38;5;3, ADR- after"
created: 2026-10-14T10:05:01Z
author: clausse@macbook-air-de-clausse
scope:
  - crates/ank-contract/src/meaning.rs
  - crates/ank-cli/src/style.rs
  - crates/ank-tui/src/paint.rs
about: TASK-a77dca75f8e3
seq: 1
schema: 4
version: 1
---

 38;5;173, [closed] after 38;5;1, open 38;5;4, proposed 38;5;5, LOG- after 2. ank find in a pty, three filters: TASK- 33, ADR- 38;5;173, [closed] 31, [open] 34, [proposed] 35. Mutation check: Decision set back to 35 in style.rs, ank-cli rebuilt, find_at_a_terminal_paints_each_kind_its_own_way fails naming [[35]]; restored, passes. Note: cargo test -p ank-tui does not rebuild the ank binary its pty tests spawn, so a mutation measured that way passes on the stale binary. NO_COLOR frame identity still holds (without_no_color_... green). cargo test --workspace: 1602 passed before the two pty tests were added.
