---
id: TASK-f2b8aedcd9a4
type: task
slug: output-with-stdin-feeds-git-from-its-own-thread
title: output_with_stdin feeds git from its own thread, so a path list larger than the pipe never deadlocks done
created: 2026-10-17T17:59:02Z
author: haksolot@omarchy
status: open
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  crate::git::output_with_stdin writes the child's stdin from a thread of its own (or otherwise interleaves writing with reading), so git hash-object --stdin-paths over any number of paths completes; a test through the binary runs ank done on a repository whose declared inputs match more than 3000 tracked files of 40-byte paths and sees it finish within the verifier timeout, red against the tree before the fix (hung, killed by the test's own deadline) and green after. The count of paths at which the current code hangs on Linux is measured and logged, together with the 64 KiB pipe arithmetic that predicts it. cargo nextest run --workspace is green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Seen on 2026-10-17 in two herdr worktrees at once, with the npm ank 0.9.2 and the same code on main: ank done hung for 53 and 42 minutes. Each ank (pid 1207158, 1398406) sat in anon_pipe_write and each git hash-object --stdin-paths child in anon_pipe with wchar exactly 64944 bytes written on stdout, which is 1584 hashes of 41 bytes: the pipe to the parent was full. output_with_stdin (crates/ank-cli/src/git.rs) calls write_all on the whole input before wait_with_output starts reading, so once the child's output outgrows the pipe before its input is consumed, both wait forever. The union verify.rs hashes is every file test-ank-cli's inputs ["**"] match: 3486 tracked paths on this repository today, about 157 KB of paths and 143 KB of hashes, past the 64 KiB a pipe holds. A done on the same inputs passed earlier the same day, before six pull requests added entity files: the corpus crossed the threshold today, which is why every worker hit it at once. Workaround in use until this lands: none; workers are held. Read but not reproduced in a test.
