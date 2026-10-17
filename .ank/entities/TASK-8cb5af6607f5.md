---
id: TASK-8cb5af6607f5
type: task
slug: a-test-that-hangs-turns-the-ci-leg-red-in-minute
title: A test that hangs turns the ci leg red in minutes, not after six hours
created: 2026-10-17T20:51:43Z
author: haksolot@omarchy
status: open
scope:
  - .config/nextest.toml
  - docs/ci-jobs.md
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  The ci profile of .config/nextest.toml declares a slow-timeout with terminate-after, so a test still running past a bound measured against the slowest honest test of the suite (the 1000-process scratch-root test on Windows among them, timed from a recent run and logged) is killed and reported as a failure, and the leg ends within that bound plus the suite; the bound and the slowest test's time are both written beside the setting, and docs/ci-jobs.md says what a TIMEOUT line in a run means. Shown by one ci run on all three platforms with the setting in place, green, with its id logged.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

On 2026-10-17 the macOS leg of PR #640 sat 46, 30 and 26 minutes in the nextest step across three heads (runs 39060806985, 39064235334, 39066576127), a hung test holding the job for what would have been GitHub's six-hour limit, while Linux and Windows finished in their usual time; each run had to be cancelled by hand. The ci profile has fail-fast = false and nothing else: no slow-timeout, so nextest never terminates a test. A bound would have turned each hang into a red leg naming the test within minutes.
