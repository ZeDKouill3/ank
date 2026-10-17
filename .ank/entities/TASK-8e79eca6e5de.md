---
id: TASK-8e79eca6e5de
type: task
slug: a-test-process-s-scratch-root-cannot-be-locked-o
title: A test process's scratch root cannot be locked or swept by a sibling between its creation and its lock
created: 2026-10-17T13:42:56Z
author: haksolot@omarchy
status: open
scope:
  - crates/ank-cli/tests/scratch/mod.rs
blocked_by: []
done_criteria: |
  In crates/ank-cli/tests/scratch/mod.rs, root() takes its lock without a window in which another process's sweep can hold that lock or judge the root abandoned. A test in that module runs root() in one process while others sweep the same base concurrently, at least 1000 times, and observes no WouldBlock panic and no removal of a live root. That test fails on the tree before the fix, and passes after on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Seen on macOS CI for PR #624 (run 39023341364, job 117369363108): amend_amends::a_task_and_a_spec_refuse_both_flags panicked at tests/scratch/mod.rs:71 with 'a fresh root cannot already be locked by somebody else: WouldBlock'. The PR changed only one .ank file.

Read but not yet reproduced: root() calls File::create(mine/OWNER) and then try_lock(). In between, a sibling process's sweep() -> abandoned() opens the same OWNER file and calls try_lock() on it. If the sibling holds the lock at that instant, root() gets WouldBlock. If the sibling's try_lock wins first, it judges the root abandoned and remove_dir_all deletes a live root. The GRACE window covers only a root with no lock file yet, not one whose lock file exists but is not yet locked.
