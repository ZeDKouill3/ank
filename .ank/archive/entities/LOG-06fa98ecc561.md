---
id: LOG-06fa98ecc561
type: log
title: "Measured through the binary: a copy placed in a node_modules/@haksolot/ank tree, npm stub exiting 0"
created: 2026-10-15T12:32:46Z
author: claude-code/opus-5.5+0766
scope:
  - crates/ank-cli/src/update.rs
about: TASK-0766190c03bf
seq: 2
schema: 4
version: 1
---

 after removing it, ank update exits 9 with 'error[9]: the route exited 0, and <path> is missing after the install' and hint 'npm install -g @haksolot/ank@9000.1.0 --prefer-online'. Before the fix the same run exited 0 (both new tests red on the code assertion). Also added the refusal and amended the note in ank-contract verbs.rs, outside the declared scope, since help --json carries update's refusals.
