---
id: TASK-6ccedaf956d8
type: task
slug: no-crate-cites-adr-559eebf5c6f5-so-adr-ac6be1ebe
title: No crate cites ADR-559eebf5c6f5, so ADR-ac6be1ebe9aa can be ratified
created: 2026-10-16T06:15:21Z
author: claude-code/opus-5.5+sweep
status: done
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/src/verbs.rs
blocked_by: []
done_criteria: |
  git grep -E '559eebf5c|ADR-559e' -- crates prints nothing; every remaining citation of ADR-ac6be1ebe9aa points at a rule ADR-ac6be1ebe9aa states; cargo test --workspace and cargo fmt --check are green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/0bcf55fb921a@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-core@11c0192e3520
    via: verifier
  - type: test
    ref: local/dd4784f6149e@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
  - type: test
    ref: local/5c56ca7525e5@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
  - type: test
    ref: local/62202734cc00@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/eb8dc89ec332@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/b6b27c3cc273@88e97b3
    tree: scope/da5798490749
    criteria: a0995660f325
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 4
---
