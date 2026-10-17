---
id: TASK-5f7b49eaca61
type: task
slug: the-specification-says-a-ratification-is-a-tag-o
title: The specification says a ratification is a tag object under refs/ank/ratify, in six revised documents
created: 2026-10-17T14:35:02Z
author: haksolot@omarchy
status: open
scope:
  - docs/specification.md
  - docs/format.md
blocked_by: []
done_criteria: |
  Six proposed spec documents exist, each superseding one of SPEC-ac4aad6d1edb, SPEC-ae044e8ac714, SPEC-93531977642f, SPEC-15a56aeedcfd, SPEC-fe809eb257c0 and SPEC-99e23aead815, carrying the same scope and references and the same body except where ADR-5e47007a75be changes it. The data model says ratified holds the anchor hash and anchor names the tag object, and that check reaches a ratification by refs/ank/ratify/<id> first and by the ratify <id> commit walk only for an anchor made before ADR-5e47007a75be. Proof and authority says a ratification is an annotated tag object on the entity's blob, signed in the signed regime, and how its signature is judged against signers. Implementation says ank never commits, with no exception. Synchronisation names refs/ank/ratify/* as the third family, with its compare-and-swap push and its never-pruned-on-time rule. The CLI surface drops the branch precondition of accept, the variable-geometry sentence and --pr, and describes --reanchor. The intent document's revision list gains one entry naming these. ank check exits 0 and ank review lists the six.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Format first (ADR-63b59c5c26f7): the specification before the goldens, the
goldens before the code. A spec is revised by a successor that supersedes it
(see the history of SPEC-ae044e8ac714: "propose, superseding ... without env",
then "ratify"). Keep every paragraph the decision does not touch byte for
byte, so the diff of each successor against its predecessor reads as the
decision and nothing else.
