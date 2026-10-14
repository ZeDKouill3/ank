---
id: TASK-dc04757dfecb
type: task
slug: a-verifier-declares-inputs-and-env-and-its-defin
title: A verifier declares inputs and env, and its definition hash covers both
created: 2026-10-13T10:11:05Z
author: claude-code/2.x
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/verify.rs
blocked_by: []
done_criteria: |
  Through the binary: ank config verifiers.<name>.inputs and the env keys of ADR-d8a328e311b9 are written to config.yml and read back; the verifier definition hash changes when inputs or env changes and is unchanged for a verifier declaring neither; an env image given by tag rather than sha256 digest is refused at parse time; check reports a changed inputs or env as it reports any verifier change; SPEC-fe67fbaa2708 names inputs and env and states what the definition hash covers.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/f8a7a2906ab6@6f7ceac
    tree: scope/d5a97b5b91ed
    criteria: 1feb5f22d6d3
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@6f7ceac
    tree: scope/d5a97b5b91ed
    criteria: 1feb5f22d6d3
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Implements ADR-d8a328e311b9, accepted and never coded. Today `VerifierFile`
(crates/ank-cli/src/config.rs) is `deny_unknown_fields` with `run`, `timeout`
and `default` only, so an `inputs` key in config.yml is refused, and
`verify::definition_hash` covers `run` and `timeout` only.

ank keeps running verifiers on the host: `env` is parsed, hashed and
validated, never executed here (the ADR leaves that to an executor).

SPEC-fe67fbaa2708 is proposed and still says the hash covers `run` and
`timeout`; it is amended here, before it is ratified, so it does not contradict
the accepted ADR.
