---
id: TASK-afcd4f0f56ad
type: task
slug: the-graph-panel-says-it-could-not-be-read-rather
title: The graph panel says it could not be read rather than reading, and the cycle break waits on whole components
created: 2026-10-14T14:52:18Z
author: ankwork
status: open
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
blocked_by: []
done_criteria: |
  The GRAPH panel distinguishes a graph never asked, a graph being read and a graph whose read was refused: a refused read is worded on every width as the narrow frame already words it (could not be read, and u reads it again), never as reading; a frame resized to DASHBOARD columns or wider while no graph is held reads the graph once, as the opening does, and never words a graph nobody is reading as reading. Unit tests in view.rs over the CLI-free seam assert the three wordings and that a resize onto the wide frame with no graph held asks for one read and a second resize asks for none. In crates/ank-tui/src/graph.rs the order that breaks a cycle counts the dependents still waiting outside a strongly connected component per component, not per node, so a node is never drawn above a node that depends on it through another cycle; a unit test with two cycles, the second blocked by the first through one edge (nodes 0 and 1 a cycle, 2 and 3 a cycle, 2 blocked by 0), asserts as a literal that every node of the second cycle stands above every node of the first, and the property test asserts the order over transitive dependence across components, not direct edges only. cargo test --workspace is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Found by the review of the GRAPH screen (2026-10-14). title_of and graph_lines word a None dag as reading on the wide frame, which is false after a refused read and after a narrow session is resized wide. order() in graph.rs counts outside dependents per node: with S1={0,1}, S2={2,3}, 2 blocked by 0, it yields 1,2,0,3 and draws 1 above 2 although 2 reaches 1 through 0.
