---
id: TASK-3a7a73ccf2b7
type: task
slug: signers-move-into-config-yml-check-reads-signers
title: "Signers move into config.yml: check reads signers:, and ank signers migrate retires allowed_signers"
created: 2026-10-11T18:26:56Z
author: claude-code/opus-5.5
status: done
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - .ank/config.yml
  - .ank/allowed_signers
  - docs/signing.md
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  Through the binary, per ADR-5c6db7a161b8: in a scratch corpus with an .ank/allowed_signers holding a gpg line, two ssh lines and comments, ank signers migrate writes them under signers: in config.yml keyed by principal, each comment preserved in a note, deletes the file, and leaves every other byte of config.yml unchanged; ank check then judges the same ratification commits with the same outcomes as before the migration; a ratification signed by an ssh key whose valid_before precedes its committer date is a fault and one signed before it is clean, and the same holds for a gpg key; an .ank/allowed_signers present beside signers: is a fault naming ank signers migrate; ank config signers.x refuses and names ank signers; ank review lists the principals from signers:; valid-before is shown honoured on Linux, macOS and Windows CI at the git version each runner carries; this repository's own allowed_signers is migrated and its ank check stays at 0 faults; docs/signing.md names signers: instead of the file, and signers leaves NOT_YET_DISPATCHED in tests/skill.rs in the commit that ships it; the section 4 and section 8 documents that name it, SPEC-14cd3e35e927 and SPEC-fe67fbaa2708, are ratified before this task closes.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/ebaa419a3cde@d0f9b95
    tree: scope/17405bf830a5
    criteria: 29cecc4b9ca7
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@d0f9b95
    tree: scope/17405bf830a5
    criteria: 29cecc4b9ca7
    verifier: fmt-check@695ca51e4ab9
    via: verifier
schema: 4
version: 4
---

First task of ADR-5c6db7a161b8. The valid-before measurement the ADR rests on was taken with git 2.55; the floor in git.rs is 2.34 and is unmeasured.
