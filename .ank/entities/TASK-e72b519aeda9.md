---
id: TASK-e72b519aeda9
type: task
slug: every-commandspec-declares-writes-help-json-carr
title: Every CommandSpec declares writes, help --json carries it, and nothing outside .ank cites ADR-e45e1a29fe91
created: 2026-10-14T10:17:25Z
author: claude-code/fable-5.1+plan
status: open
scope:
  - crates/ank-contract/src/**
  - crates/ank-cli/tests/**
  - SECURITY.md
  - docs/conventions.md
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/paint.rs
blocked_by: []
done_criteria: |
  Through the binary, per ADR-3c8d94a0bbe0: ank help --json carries writes on every verb, one of nothing, files, refs or both, and the golden help fixtures carry it; a test in the workspace fails when a CommandSpec lacks a writes declaration, and fails when a verb declaring nothing is measured to create or change a path under .ank/ or a ref under refs/ank/ on a scratch corpus, counted after each verb as ADR-e45e1a29fe91 counted by hand; no tracked file outside .ank/ cites ADR-e45e1a29fe91, each former citation naming ADR-3c8d94a0bbe0, so that ank accept ADR-3c8d94a0bbe0 is not refused on citations.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Nine tracked files cite ADR-e45e1a29fe91 on c859d01f: SECURITY.md, docs/conventions.md, crates/ank-cli/src/commands.rs, git.rs, human.rs, paint.rs, crates/ank-cli/tests/cli.rs, tests/skill.rs and crates/ank-contract/src/verbs.rs; two are inside strings the binary prints, so their assertions move too. The declaration is the third fact of its kind on CommandSpec after coordinates and renews.
