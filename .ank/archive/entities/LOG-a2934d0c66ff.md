---
id: LOG-a2934d0c66ff
type: log
title: "After the change, same fixture and method (GIT_TRACE at an absolute path, 'trace: built-in:'"
created: 2026-10-16T06:22:26Z
author: claude-code/opus-5.5+5395
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5395fef418b7
seq: 2
schema: 4
version: 1
---

 lines): ank check --json starts 14 git processes, 4 fewer than the 18 at 409bb93c, and no 'cat-file -p <rev>:.ank/<dir>' line remains; each revision's three layout directories are one 'cat-file --batch' (git::trees_at). Asserted exactly by cli.rs#a_revisions_corpus_is_listed_by_one_git_process. The unit test git.rs#trees_are_listed_in_the_order_asked_and_what_is_no_directory_is_none went green on first run (written after trees_at), so it was mutated: is_blob forced true made it fail on the subdirectory entry. The one other directory listing, archived_against_head, is already one cat-file -p for HEAD's archive and is untouched.
