---
id: TASK-ba8ec1f11085
type: task
slug: count-the-processes-each-test-of-cli-rs-and-stat
title: Count the processes each test of cli.rs and status.rs starts
created: 2026-10-13T10:11:07Z
author: claude-code/2.x
status: in_progress
scope:
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/status.rs
blocked_by: []
done_criteria: |
  An ank log records, for the 20 tests of cli.rs and status.rs that start the most processes, the count of ank and git processes each starts, measured with GIT_TRACE at an absolute path and a count of ank spawns, never with a clock; one task is filed per reduction it motivates, each with a numeric bound on the count.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 2
---

On Windows, cli.rs takes 407.6s against 36s on Linux, and status.rs 89.3s
against 4.9s: process creation costs about 25ms on a Windows runner against 2
to 3ms on Linux (CLAUDE.md). cli.rs holds 399 spawn sites. This task measures
before anything is cut, so each reduction is bounded by a count.
