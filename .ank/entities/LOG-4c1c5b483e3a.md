---
id: LOG-4c1c5b483e3a
type: log
title: "drift audit 2026-10-14, measured on c859d01f: crates/ank-tui is 28290 lines in 37 files and pulls"
created: 2026-10-14T09:36:13Z
author: claude-code/fable-5.1
scope:
  - crates/ank-cli/**
  - docs/**
about: ADR-8bd76e8d7c4e
seq: 1
schema: 4
version: 1
---

 83 crates (cargo tree -e normal), against 38 for ank-core and 4900 lines. It links ank-contract only, so the one-dispatch-path clause holds. The reader has had three ratified shapes and one proposed since 2026-08-24 (ADR-c07e, ADR-559e, ADR-ac6b). Holds as written; whether it belongs in the one executable is a question for the perimeter decision, not for this rule.
