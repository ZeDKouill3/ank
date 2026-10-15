---
id: LOG-d81b7c152d5f
type: log
title: "Clause by clause at 2129db77: migrate, valid_before ssh and gpg, file beside signers:, config"
created: 2026-10-15T21:52:56Z
author: claude-code/opus-5.5+3a7a
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
about: TASK-3a7a73ccf2b7
seq: 4
schema: 4
version: 1
---

 refusal and review are tests/signers.rs through the binary, green here (Linux, git 2.47.3, OpenSSH 10.0p2). valid_before is judged by check itself for both formats as well as handed to git, so it does not rest on the runner's git honouring valid-before; git 2.47 honours it, measured in a scratch repo (U after the bound, with or without the Z suffix). The three-platform clause is NOT shown yet: ci.yml runs on pull_request and on push to main only, and this branch is pushed without a PR, so the macOS and Windows runs of tests/signers.rs happen when the PR is opened; whoever opens it reads those three jobs before merging. SPEC-14cd3e35e927 and SPEC-fe67fbaa2708 were ratified (4e43cbaa7b09, 0ca80570ee7b) and are since superseded by SPEC-fe809eb257c0 and SPEC-ae044e8ac714, both accepted and both naming signers: and ank signers. This repository's allowed_signers is migrated in this commit; ank check at 0 faults, 67 signals (66 before plus scope-not-yet on this task for the deleted file). Out of declared scope: cli.rs (dispatch, count 30->31), tests/cli.rs, tests/status.rs, tests/main.rs, tests/signers.rs (new), golden-json help/verify/signers, ank-core/tests/reference_pages.rs, docs/config-keys.md (generated), docs/format.md, docs/check.md, docs/claims.md, docs/quickstart.md, docs/integrating.md (the no-signers signal's subject is now signers, and check sorts it after corpus).
