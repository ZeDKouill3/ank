---
id: LOG-73b8def9d377
type: log
title: "Enforcement audit for TASK-74c6c7576a6f, at b3b568bc: no enforced_by declared, and the constraint"
created: 2026-10-15T19:43:55Z
author: claude-code/opus-5.5+74c6
scope:
  - crates/ank-cli/**
about: ADR-6fd69efb629c
seq: 1
schema: 4
version: 1
---

 stays served whole. The shape is held by cli.rs#every_golden_conforms_to_the_shape_its_verb_declares, schema.rs#every_verb_that_declares_a_document_has_a_golden_fixture and mcp.rs#the_verb_serves_the_handshake_the_table_and_the_binarys_own_document, but 'one writer and one escaper' is a code-structure rule no test holds (a hand-built format! document would pass), and 'anything else bumps the version' is a process rule: the shape table and the goldens can move together with CONTRACT_VERSION unchanged. Both bind whoever adds output to human.rs, so the text stays served.
