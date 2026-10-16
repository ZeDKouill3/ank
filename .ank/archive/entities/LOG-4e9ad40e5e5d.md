---
id: LOG-4e9ad40e5e5d
type: log
title: "Measured through the binary, scratch repo, verify [first: 'sleep 0.3; exit 3', second: 'sleep 37 &"
created: 2026-10-11T20:17:06Z
author: claude-code/opus-5.5/task-26ea
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-contract/src/verbs.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-26eab418555a
seq: 7
schema: 4
version: 1
---

 wait', third: 'sleep 38'], verifier_jobs 2: done exits 5 with 'verifier 'first' failed (exit 3)', one running line; ps -C sleep counts 0 'sleep 3[78]' processes 0s and 1s after it returns; TMPDIR left empty (0 files), cancelled verifiers remove their capture file. Mutation: with the group kill turned into kill -0, the heartbeat test goes red ('a verifier process is still running after done returned'), so the test catches an orphaned grandchild; restored. Red before the change: 7 of 8 tests (6 on 'unknown field verifier_jobs', and verifier_jobs unset timed out left at 10.0s, i.e. sequential). Green after: 8/8, cargo test --workspace green, fmt clean. Trade-off noted: on Unix a verifier now leads its own process group, so a Ctrl-C at the terminal reaches ank and not the verifier directly.
