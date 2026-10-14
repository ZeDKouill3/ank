---
id: TASK-c3ec83ce25e9
type: task
slug: the-reader-reads-the-blocked-by-graph-through-an
title: The reader reads the blocked_by graph through ank graph --json, and the road stays a read
created: 2026-10-14T09:15:26Z
author: ankwork
status: in_progress
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/src/model.rs
blocked_by: []
done_criteria: |
  ank::READS names graph beside status, find, show, scope and review, and the gate tests that keep every act off the reading road still pass with it there; nothing else joins READS and nothing joins ACTS. model carries a Dag read out of one `ank graph --json` document: every row of `tasks` (id, short, status, title) and every row of `edges` (task, blocked_by), including an edge whose blocked_by names an id that is no row of tasks, such as a peer task written <id>@<peer>; the model derives nothing beyond what the document states, and in particular computes no order and no readiness. Unit tests in model.rs read a literal document carrying four tasks, three edges and one edge to an id outside the task list, and assert the counts and the ids read back; a document with an empty tasks array and an empty edges array reads as an empty Dag and not as an error. `cargo test -p ank-tui` is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

First of the GRAPH screen tasks: the data, and nothing drawn yet.

ADR-8bd76e8d7c4e is the whole reason this is a separate task: the reader reaches the corpus only by running the CLI with --json, so the DAG arrives as `ank graph --json` and never as a walk of blocked_by fields. Measured on this corpus on 2026-10-14 with the release binary: the document is 84 KB, 501 tasks, 210 edges, 1 git process (ADR-cc65f1388a71's table). The shape it has today -- tasks (id, short, status, title) and edges (task, blocked_by) -- is enough: `created`, which the topological tie-break needs, is already in the find snapshot the reader holds, and the join is by id. No CLI change, so ADR-3e6ce108edcd and ADR-6fd69efb629c are not touched: the listing already answers every row and no field moves.

Edges to a peer (`TASK-xxxx@peer`, ADR-c23bef1cc93e) name no row of tasks; the model keeps them, and the layout task decides what an edge to nowhere looks like. Keep the model as thin as Snapshot and Held are: the view derives, the model reads.
