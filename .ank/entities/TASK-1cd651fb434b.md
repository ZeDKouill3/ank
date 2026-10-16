---
id: TASK-1cd651fb434b
type: task
slug: the-painted-and-the-plain-colour-sessions-compar
title: The painted and the plain colour sessions compare the same settled screens
created: 2026-10-16T08:04:14Z
author: claude-code/opus-5.5+flake
status: done
scope:
  - crates/ank-tui/tests/colour.rs
  - crates/ank-tui/tests/terminal/**
blocked_by: []
done_criteria: |
  colour::without_no_color_it_paints_and_the_two_screens_are_the_same_characters passes 50 runs out of 50 under load, against the failure rate measured before the fix, with no retry and no longer wait added
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@6b22edb
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/0bcf55fb921a@88e97b3
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/opus-5.5+sweep
  - type: test
    ref: local/dd4784f6149e@88e97b3
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/opus-5.5+sweep
  - type: test
    ref: local/5c56ca7525e5@88e97b3
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/opus-5.5+sweep
  - type: test
    ref: local/d5ea7af27498@6b22edb
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/6d9c8875f8f4@6b22edb
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/e61b1acc9ec7@6b22edb
    tree: scope/f1e13e1fd02e
    criteria: 5538f41b43bd
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 6
---
