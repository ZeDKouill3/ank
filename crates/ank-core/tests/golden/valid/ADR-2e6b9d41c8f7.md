---
id: ADR-2e6b9d41c8f7
type: adr
slug: every-request-is-logged
title: Every request is logged, and a test holds it
created: 2026-10-14T10:16:07Z
author: human:marie
status: accepted
scope:
  - src/http/**
constraint: |
  Every request the server answers is logged with its route and its status.
enforced_by:
  - src/http/tests.rs#every_request_is_logged
  - .github/workflows/ci.yml#log-audit
ratified: 4f2a9c1e7b03
schema: 4
version: 2
---

An ADR at schema 4 naming what holds it: `enforced_by` sits between `see` and
`amends`, a block list of `<path>#<symbol>` entries, and it is outside the
ratification anchor, which covers the constraint and the scope alone.
