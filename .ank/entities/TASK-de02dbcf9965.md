---
id: TASK-de02dbcf9965
type: task
slug: the-graph-screen-draws-one-block-per-group-of-li
title: The GRAPH screen draws one block per group of linked tasks on the junction lanes, the cursor walks task rows only, and the unlinked tasks come last
created: 2026-10-14T18:31:30Z
author: planner-b
status: done
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
blocked_by: [TASK-080f158e317e, TASK-9ea4dc65755b]
done_criteria: |
  The GRAPH panel is laid out with the junction-row layout of crates/ank-tui/src/graph.rs, and the first layout, the rows and the table entries only it used, and its tests are removed from that module, which declares one table again. The tasks shown are split into groups: the weakly connected components over the drawn edges, an edge being drawn between two shown tasks exactly as the layout draws it today. Every group of two or more tasks is one contiguous block of rows; a row of exactly one blank separates two blocks; the tasks in a group of one are gathered in a last block headed by one row whose text is unlinked, drawn whenever that block is not empty and preceded by a blank row when a block stands above it; no blank row stands first or last. Blocks stand in this order: blocks holding at least one task whose status is neither done nor closed before blocks holding none; then more tasks before fewer; then the block whose most recently created task is the newer first, created read from the find snapshot and a task the snapshot withholds counted as the oldest; the unlinked block last whatever it holds; and inside a block the rows are the layout's, a task above every task it is blocked by and the listing's own order where the edges leave two tasks free. A task row is composed by compose as before and carries the cell of its mark: ● for done or closed, ○ for every other status but in_progress, and the first phase of the pulse (·) for in_progress; on a terminal declaring TERM=dumb every cell, lane and junction glyph is the ASCII alphabet (@ o . and the connectors graph.rs declares); the readiness word, the outside count and [held] are as the task before this one left them. The cursor walks task rows only: j, k, Space, g, G, the arrows and a tap select a task row and never a junction, blank or heading row, the title counts task rows, the selected row is always inside the panel, and a tap on a junction, blank or heading row selects and opens nothing. f and / keep working: the fold and the search recompute the groups and the lanes over the surviving tasks, the cursor stays on its row where that row survives, and neither spawns a read (ANK_TRACE_READS, measured through the binary). The frame painted and the frame drawn with NO_COLOR are identical character for character. Unit tests in crates/ank-tui/src/view.rs over the CLI-free seam, run on every platform, assert over the literal graph document as literal frames: the blocks and their order at rest, the blank rows between them, the unlinked heading and its block, a junction row ├─┘ standing immediately above the first of the chain and one above the top of the diamond, that Down from the task row above a junction row lands on the task row under it and that the junction row is never selected, the title's count, that a search leaving one task draws it alone under unlinked, and that the fold all puts the finished-only groups after the alive ones. Tests in crates/ank-tui/tests/graph.rs drive the binary over Repo::graphed at 80x24 and at 160x45 and assert: the blocks at rest are, top first, the chain with the task under way, the diamond, the closed blocker with its dependent, then unlinked holding the lone task; exactly one blank row between consecutive blocks; a junction row carrying ├─┘ immediately above the first of the chain and one above the top of the diamond; Down from the second of the chain lands on the first of the chain and Enter there opens its document and Back returns with it selected; the cell of the task under way is · and the closed blocker's is ●; under TERM=dumb the same frame carries @, o, . and the ASCII connectors; j, k, f and a search grow the ANK_TRACE_READS file by nothing; under the fold all the finished history is a block after every alive block; the painted frame and the plain one are the same characters. Every test of crates/ank-tui passes on Linux, measured by running cargo test -p ank-tui after cargo build --workspace in a rust:1.98-bookworm container and recorded with ank log. cargo test --workspace is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/fc654c0e3635@ef1cdbe
    tree: scope/b4290ee6faf2
    criteria: b1cf589bc2b2
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@ef1cdbe
    tree: scope/b4290ee6faf2
    criteria: b1cf589bc2b2
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

The screen of GRAPH option B: what the layout task and the readiness task
built, joined and drawn. This is the heavy one, and it is one task because
the groups, the junction rows and the cursor over them are one change to how
a row of the graph is addressed, and splitting it would leave an in-between
tree in which the cursor could land on a row that is nobody's.

**Groups, and why they are blocks.** On this corpus on 2026-10-14 the open
fold shows 16 tasks: two linked groups and six tasks linked to nothing,
measured with the prototype. Drawn as one lane walk those six interleave with
the groups and a person reads edges between things that have none. The human
chose blocks: each group of linked tasks contiguous, a blank row between
blocks, and the tasks linked to nothing in a last block headed `unlinked`.
The heading is a row of text and never a border around nothing
(ADR-559eebf5c6f5), and it is English, as everything the reader draws is
(ADR-d3a8dcf3a56f); the prototype's French labels are only the prototype.

**The order of the blocks** is the order of ADR-559eebf5c6f5 applied to
groups: the work that is alive first, then the rest. Among alive groups the
bigger first (the largest structure is the one a person is orienting in),
then the most recently created. `created` is already in the find snapshot
(TASK-c3ec83ce25e9 relied on exactly that join); a task `find` withheld under
its budget has no `created` here and is counted oldest rather than guessed.

**Rows that are nobody's.** A junction row, a blank row and the heading are
screen lines and not rows of the listing: the cursor walks task rows only,
the title counts task rows, and paging counts screen lines so that the
selected row is always inside the panel. `Listing::total`, `count`,
`selected_id` and `keep_graph_on` read `drawn.rows` today and each has to
learn the difference; the frame is drawn by `graph_lines` over the page.
Where exactly the mapping between task index and screen line lives is the
implementer's; what the criterion fixes is what a key does and what a tap
does.

**The marks move to the new alphabet here**, which is why this task removes
the first layout from graph.rs: once the view reads the new one nothing reads
the old, and one table is the rule (TASK-9f11c7c06f72). In progress is drawn
at the pulse's first phase and does not move yet; the clock is the next task,
and until it lands a frame is still a function of the corpus and the keys.

**Fold and search** recompute the groups over the survivors, which falls out
of computing the groups inside relayout, where TASK-67583993ef0d already
narrows the shown set. A group the search cuts in two becomes two blocks or
two unlinked rows, and the row under the cursor stays under it where it
survives, as it does today.

The visual reference is the prototype the human approved:
C:/Users/nicoc/AppData/Local/Temp/claude/E--ClaudeRandom-AnkWork/cd1f51f4-51fb-4033-9e16-68f45f4fa7c0/scratchpad/graph_b.py
(`groups`, the `linked.sort` and `lone.sort` keys, `topo`, and the printing
loop that puts a blank between blocks and the heading before the lone ones).

Relies on: the layout task before this one (the junction-row layout, the
groups function, the alphabet), the readiness task before this one (the word
column and the fixture's task under way, which is what puts two dependents on
the first of the chain), TASK-643611645dea and TASK-67583993ef0d (the panel,
the read, the fold and the search), TASK-44e35bd40c83 (graph_rows reading rows
inside the panel), TASK-afcd4f0f56ad (the three states of an unread graph).
