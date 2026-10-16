---
id: TASK-ccce7b24350c
type: task
slug: a-reader-that-wrote-nothing-does-not-checkpoint
title: A reader that wrote nothing does not checkpoint the index on close
created: 2026-10-16T12:36:27Z
author: claude-code/opus-5.5+intel
status: done
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/warm_readers.rs
blocked_by: []
done_criteria: |
  A connection to the index that committed no change closes without checkpointing the WAL, so no reader takes the exclusive lock that refuses other readers. Through the binary: over an index whose WAL holds more than 8 MB of frames no checkpoint applied and no -shm, a lone 'ank find' exits 0 and leaves the -wal file present with the same byte count and the database file with the same byte count it found. A verb that committed a write still checkpoints on close as before. Measured cause, staged locally (one reader, then four started 30-100 ms later, ANK_INDEX_BUSY_MS=0): 72 of 96 late readers refused with extended code 5 (SQLITE_BUSY, not 261) at ensure_schema's first read while the first reader's close checkpointed the WAL; 0 of 96 with checkpoint-on-close off.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/c892d98f4e8c@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-core@11c0192e3520
    via: verifier
  - type: test
    ref: local/4f957d528852@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
  - type: test
    ref: local/1fbce85ba02a@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
  - type: test
    ref: local/7caa8253a3ea@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/6a278f762e7f@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/bae2c83b4b60@6d47ff3
    tree: scope/55fdbb8f0950
    criteria: 9a98247bbace
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 4
---

Release rehearsal v0.9.2 (run 38897015410, macos-15-intel, two attempts): warm_readers::readers_meeting_a_wal_recovery_are_not_refused refused readers #8-#11 with 'database is locked'. Since #580 the test shares one binary with ~720 others, so the last readers spawn after the first ones exit; the last reader out checkpoints the 18 MB WAL under an exclusive lock and the late readers meet plain SQLITE_BUSY, which TASK-f0dd3c98a6bf's 261 wait does not cover.
