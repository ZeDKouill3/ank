---
id: TASK-91cefb36af81
type: task
slug: env-leaves-the-verifier-block-config-yml-refuses
title: "env leaves the verifier block: config.yml refuses the key, the definition hash covers run, timeout and inputs, and the docs name no env"
created: 2026-10-14T12:43:27Z
author: claude-code/fable-5.1+plan
status: open
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/verify.rs
  - docs/config-keys.md
  - .ank/entities/SPEC-fe67fbaa2708.md
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Through the binary, per ADR-7224dea611c1: a config.yml carrying an env block under a verifier fails to load, and the error names env as the key the parser does not know; ank config verifiers.<name>.env.image, .platform, .network and .vars.<VAR> are refused by name, and ank config --help and the config-keys page list no env key; the verifier definition hash is unchanged for a verifier declaring run, timeout and inputs only, measured against the hash recorded in the proof of TASK-dc04757dfecb on the same definition; the hash still changes when inputs changes; docs/config-keys.md carries no env row and docs/ci.md names no image; SPEC-fe67fbaa2708 names inputs and no env; no symbol named Env, EnvFile or Network remains under crates/ank-cli/src/config.rs; cargo test --workspace and ank check are green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

TASK-dc04757dfecb finished on 2026-10-14 (46fa39c7) with the env keys of ADR-d8a328e311b9 parsed, validated and hashed, before ADR-7224dea611c1 withdrew them. This task takes them out again, and runs only once that ADR is accepted: until then ADR-d8a3 still binds, and removing what it asks for is a fault, not work. Measured on 46fa39c7: EnvFile and Env in crates/ank-cli/src/config.rs, four env rows in the key table of crates/ank-core/src/config.rs and docs/config-keys.md, the env arm of definition_hash in verify.rs with its test, and the env paragraph of SPEC-fe67. inputs stays whole, with its hash coverage and its ank config route.
