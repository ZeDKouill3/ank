---
id: LOG-575b9e48d4dd
type: log
title: "TDD: 7 tests through the binary in crates/ank-cli/tests/signers.rs (outside the declared scope;"
created: 2026-10-15T22:40:38Z
author: claude-code/opus-5.5+6831
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - docs/signing.md
about: TASK-68313ec97cd6
seq: 1
schema: 4
version: 1
---

 tests/ is where migrate's live), all 7 watched red first (unknown subcommand 'add'/'retire'/'remove', help without them), then green. Readings taken: <key> for retire/remove is '<keytype> <key>' or the key alone; an undeclared key is refused at 1 (no entity, so not 2); remove of a principal's last key removes the principal, of the last principal removes signers: (an empty mapping would not parse). Surprise: remove with no other key declared faults nothing, because a corpus declaring no key is advisory (§8) -- the test keeps a second principal declared so the faults are the key's. Out-of-scope edits forced by the suite: cli.rs listing caps the usage column at 30 (signers' 47-char usage narrowed every summary and broke the_listing/help tests), tests/cli.rs classifies --key as outside-the-repository and --principal/--note/--date as not a path, golden help.json blessed.
