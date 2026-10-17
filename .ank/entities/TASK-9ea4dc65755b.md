---
id: TASK-9ea4dc65755b
type: task
slug: a-graph-row-says-ready-waits-n-in-progress-done
title: A graph row says ready, waits N, in progress, done or closed in place of the raw status, and the fixture holds a task under way
created: 2026-10-14T18:31:19Z
author: planner-b
status: done
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
blocked_by: []
done_criteria: |
  On GRAPH the column that held the raw status word holds a readiness word instead, composed by compose and by no second composer, in the slot between the identifier and the title: in progress for a task whose status is in_progress; done for done; closed for closed; ready for every other task all of whose blocked_by name tasks the graph document lists as done or closed; and waits N otherwise, N counting the blocked_by whose task is neither done nor closed, an identifier the document lists no task for counted among them, and N counted over every blocked_by the task declares whether or not the blocker is a row. The word is painted through the shared table and nothing else: in progress, done and closed in the role the table gives their status, ready in the role the table gives open, waits N in the Attention role; the frame painted and the frame drawn with NO_COLOR are identical character for character, and no row of GRAPH carries the raw word open. The count of blockers outside the layout stays after the title in the words TASK-67583993ef0d gave it, and [held] stays appended to the title where the claims say so. The fixture Repo::graphed in crates/ank-tui/tests/terminal/mod.rs gains an eleventh task, written through the CLI, blocked by the first of the chain and claimed through the CLI so that graph --json lists it as in_progress, and Graphed names it; the test that measured the fixture's document asserts eleven tasks, nine open, one in_progress, one closed and eight edges, and every other test of crates/ank-tui/tests/graph.rs is rewritten to the counts the fixture now has. Unit tests in crates/ank-tui/src/view.rs over the CLI-free seam, run on every platform, extend the literal graph document with one in_progress task and one edge to an identifier the document does not list, and assert as literal rows the five words, a waits N that counts a hidden blocker, and a waits N that counts an unlisted one. Tests in crates/ank-tui/tests/graph.rs drive the binary over the fixture at 80x24 and at 160x45 and assert: the row of the task under way reads in progress and [held]; the dependent of the closed blocker reads ready; the bottom of the diamond reads waits 2; the closed blocker reads closed; under the fold all, the task historied finishes reads done; no row of the panel carries the word open; and the painted frame and the plain one are the same characters. Every test of crates/ank-tui passes on Linux, measured by running cargo test -p ank-tui after cargo build --workspace in a rust:1.98-bookworm container and recorded with ank log. cargo test --workspace is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/38e417015b51@fcf1f0b
    tree: scope/94713d3db64c
    criteria: 77ddafbbc3f5
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@fcf1f0b
    tree: scope/94713d3db64c
    criteria: 77ddafbbc3f5
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

The readiness column of GRAPH option B: the row says what the graph knows
and the status column does not.

**Why a word and no longer a glyph.** TASK-643611645dea put readiness in the
node's cell (filled for ready, hollow for waiting) and kept the CLI's status
word in the column, on the reasoning that a word the shared table does not
know would be a second table (ADR-1f70ce2c3eac). The human read the result
and asked for the word: `ready` and `waits 2` are what a person opened the
graph to learn, and a cell is not legible enough to carry it. The table is
not crossed: `ready` and `waits N` are not statuses, they are facts the DAG
computes, and they are *painted* with roles the table already declares --
`ready` in the role of `open`, `waits N` as Attention, `in progress`, `done`
and `closed` in the role of their own status. So the paint still comes from
one place and the second renderer is still allowed; what this crate adds is
a word, not a meaning. The prototype painted ready cyan, waiting yellow,
finished grey and in progress green: those hues give way to the table's
registers, because colour carries nothing here (ADR-559eebf5c6f5) and the
table is where a hue is decided.

**Where the word stands.** The prototype puts it at the far right, after a
title cut to fit. compose has one elastic field and it is the last one; a
word after an elastic title would be a second composing rule, and the
rightmost droppable field is the first a narrow window gives up, which would
drop the word before the title. So the word takes the slot the status held,
between the identifier and the title, and keeps the width behaviour that
slot already has at every window the suite opens.

**`waits N` counts every blocker, not only the drawn ones.** A task shown
under the open fold beside a hidden blocker still waits on it; the layout's
outside count says how many edges were not drawn, and the word says how many
blockers are not finished. The two numbers answer two questions and both are
kept. An identifier the document lists no task for -- a peer task
(ADR-c23bef1cc93e), an id that names nothing -- is not known to be finished
and is counted as waiting, the rule the mark already applied.

**`done` and `closed` are two words.** The brief says `done`; the CLI says
`done` or `closed`, and ADR-6d8736c04cfa makes the asymmetry a decision. The
word on the row is the status's own, so a closed task reads `closed`.

**The fixture gains a task under way.** No test corpus of this crate holds a
task in progress on the graph, so the one word the human most wants to see
is unmeasured until one does. It is blocked by the first of the chain, which
gives that root two dependents: the task after this one needs exactly that
to draw a junction row above it. The counts every graph test asserts move by
one, and this task moves them.

The visual reference is the prototype the human approved:
C:/Users/nicoc/AppData/Local/Temp/claude/E--ClaudeRandom-AnkWork/cd1f51f4-51fb-4033-9e16-68f45f4fa7c0/scratchpad/graph_b.py
(its `role` table and the `waits` count in the printing loop).

Relies on: TASK-643611645dea (the row through compose, the join of the graph
document with the snapshot and the claims, the mark rule this word inherits),
TASK-67583993ef0d (the fold, the search, the outside count), TASK-44e35bd40c83
(graph_rows finding a row inside the panel whatever lanes cross it),
TASK-367de5144982 (the suites green on Linux, and the container recipe in its
log).
