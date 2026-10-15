---
id: LOG-15bbabd646d0
type: log
title: Mutation runs, each applied alone to crates/ank-cli/src/style.rs detect(), cargo build --workspace,
created: 2026-10-15T18:45:59Z
author: claude-code/opus-5.5+8ad3
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-8ad3edcbd0fb
seq: 2
schema: 4
version: 1
---

 then the single test. M1 NO_COLOR wired backwards (the terminal check becomes '!is_terminal() && NO_COLOR is none', so setting NO_COLOR turns colour on in a pipe): FAILED, '["context"] under [("NO_COLOR", Some("1"))] coloured a pipe'. M2 Windows allowlist consulted in place of the terminal test (the terminal check becomes '!WINDOWS_TERMINALS.any(set)'): FAILED, '["context"] under [("TERM", Some("xterm-256color"))] coloured a pipe'. Restored: ok. A first draft caught M2 only because TERM_PROGRAM/TERM leaked in from the shell running the suite; every run now lays its environment over one that removes NO_COLOR, TERM, TERM_PROGRAM, ConEmuANSI, WT_SESSION and ANSICON, so both catches hold on any developer's shell.
