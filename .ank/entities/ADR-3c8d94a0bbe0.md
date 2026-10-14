---
id: ADR-3c8d94a0bbe0
type: adr
slug: the-ank-directory-is-reached-only-through-the-cl
title: The .ank directory is reached only through the CLI, and the contract says which verbs write
created: 2026-10-14T10:16:07Z
author: claude-code/fable-5.1+plan
status: proposed
scope:
  - .ank/**
  - crates/ank-contract/src/verbs.rs
constraint: |
  Entities in .ank/ are read and written through the ank CLI only. No agent opens, greps, lists or edits a file under .ank/ directly. This constrains the agent and not the tool: a human with an editor keeps every power they had, and check remains what notices. Which verbs write is not a list this document keeps: every CommandSpec declares writes, one of nothing, files, refs or both, ank help --json carries it on every verb, and a test holds every verb to a declaration confronted with what the verb leaves on disk and in refs/ank. The reading side is the verbs that declare nothing, the writing side is the rest, and a verb added later is on one of the two sides on the day it dispatches.
supersedes: ADR-e45e1a29fe91
schema: 4
version: 1
---

## Context

ADR-e45e1a29fe91 carries the rule of ADR-01b6dd05f0db and two enumerations of
verbs, reading and writing, and says of them: "This is a list of verbs inside a
constraint, and the tree it describes moves. It went stale exactly once per new
route, silently, and what noticed was a drift audit rather than anything
mechanical."

Measured on 2026-10-14 (c859d01f): `ank help --json` dispatches 30 verbs.
`archive` and `migrate` write `.ank/` and appear on neither list; `init`,
`watch`, `update` and `skills` appear on neither. ADR-5c6db7a161b8 amended the
list once already to add `signers`. The enumeration went stale again, as the
body predicted, and the mitigation it named as "worth proposing on its own",
`check` comparing the lists against the dispatch table, is the one taken here,
in the form that needs no list at all.

`CommandSpec` (crates/ank-contract/src/verbs.rs) already declares two facts of
this kind per verb: `coordinates`, whether git is required, and `renews`,
whether the verb renews a claim. Whether a verb writes is the third, and it is
the one an agent reading the rule needs.

## Decision

The rule stays word for word. The lists go. Each verb declares `writes` in the
contract, `help --json` carries it, and a test measures each verb on a scratch
corpus by counting the paths under `.ank/` and the refs under `refs/ank/` it
leaves behind, the measurement ADR-e45e1a29fe91 itself made by hand, against
the declaration. An agent that wants the lists runs `ank help --json`, which is
the surface ADR-6fd69efb629c says cannot fall behind the binary.

## Consequences

- `writes` joins `CommandSpec`; `help --json` gains a field within contract 1.
- The nine tracked files citing ADR-e45e1a29fe91 (SECURITY.md,
  docs/conventions.md, and seven under crates/) name this document before it
  can be accepted (ADR-3b6ba766a42e).
- The skill contract keeps saying `.ank/` is opaque; it never carried the
  lists.

## Rejected

- **A third enumeration, complete today.** Correct for one release.
- **Deriving the lists in `check` from the dispatch table.** The table does
  not know what a verb writes unless the verb says so; a declaration measured
  by a test is the same fact, held where it cannot go stale silently.
