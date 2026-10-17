---
id: TASK-adea229723df
type: task
slug: the-pseudo-terminal-harnesses-read-the-slave-s-n
title: The pseudo-terminal harnesses read the slave's name with ptsname_r or under a lock, so two concurrent sessions never share one terminal
created: 2026-10-17T16:18:48Z
author: claude-code/fable-5.1+4952
status: open
scope:
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
blocked_by: [TASK-4952001e9596]
done_criteria: |
  pty::open in crates/ank-tui/tests/terminal/mod.rs and in crates/ank-cli/tests/tui.rs no longer reads the slave's path out of ptsname(3)'s static buffer: either ptsname_r into a buffer the caller owns, or TIOCGPTN on Linux, or a lock held across posix_openpt, grantpt, unlockpt and ptsname. A test in crates/ank-tui/tests/terminal/mod.rs opens sessions from several threads at once, at least 1000 per thread with the masters held, and asserts every path answered equals the one TIOCGPTN (Linux) or ptsname_r re-asked single-threaded gives for the same descriptor; it is run red against the ptsname(3) read before the change and the count is recorded with ank log. The ONE_AT_A_TIME comment and the pty-serial group in .config/nextest.toml are revisited: kept only if a reason other than ptsname remains, and the reason is written where the lock is. cargo test --workspace is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Measured by TASK-4952001e9596 on 2026-10-17 (LOG-60f070d91ca6, LOG-6205cf467706): 4 threads each running posix_openpt/grantpt/unlockpt/ptsname with 8 masters held, 160000 opens, 8 answers naming another thread's terminal; 40000 single-threaded opens, 0. Two readers deliberately placed on one slave: one q written, one of the two exits and the other waits forever, 3 rounds of 3. That is the hang seen once on 2026-10-14 in dashboard.rs (the_desk_still_draws_one_panel_at_a_time never got its q; a_wide_frame_draws_every_panel_and_marks_one blank, because its own master had no slave attached). dashboard.rs takes no ONE_AT_A_TIME lock, and the lock is a mitigation named in the harness as such: the fix belongs in pty::open, in both copies.

The inherited /dev/ptmx descriptors TASK-4952001e9596 was written about were measured and were not the cause; that task closed the leak and left this one.
