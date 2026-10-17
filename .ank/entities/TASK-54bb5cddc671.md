---
id: TASK-54bb5cddc671
type: task
slug: an-in-progress-row-pulses-every-250-ms-while-it
title: An in-progress row pulses every 250 ms while it is on the screen, and the clock paints, reads nothing and never waits on a key
created: 2026-10-14T18:31:31Z
author: planner-b
status: done
scope:
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/term.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/graph.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
blocked_by: [TASK-de02dbcf9965]
done_criteria: |
  Wake gains Tick, and a ticker started by the session sends one Tick on the session's channel at most every 250 milliseconds and only while App says a pulse is due, which is when at least one task row whose status is in_progress stands inside the GRAPH panel's page on the frame last drawn, the panel being the region under DASHBOARD columns with the focus on GRAPH and the right panel at DASHBOARD columns or wider while no document is open in it; when no such row is on the screen no Tick is sent and no frame is drawn. A Tick advances a phase counter held by App by one and draws one frame in which the in_progress cells read phase modulo four of the pulse (· • ● •, under TERM=dumb . o O o) and nothing else differs from the frame before; the counter is the only clock the frame reads, App::tick is what a test drives, and a test in crates/ank-tui/src/lib.rs reads the sources of lib.rs and view.rs and finds neither Instant, SystemTime, thread::sleep nor recv_timeout in them. The session loop blocks on its one channel and never sleeps, so a key arriving between two ticks is answered when it arrives: a unit test in lib.rs drives the session over a corpus-free App with the wakes Tick, Key(j), Tick and asserts, over the frames the Painter received, that the cursor moved on the second frame and that the first and third differ from each other only in the pulse cells; and a driven sequence of Ticks over an App with no in_progress row on the screen draws no frame at all. A Tick spawns nothing: through the binary, the ANK_TRACE_READS file is byte for byte the same after three seconds of pulsing as before them. Tests in crates/ank-tui/tests/graph.rs drive the binary over Repo::graphed at 80x24 and at 160x45 and assert: with the task under way on the screen, every one of the four phase glyphs is seen in its cell within three seconds and at most thirteen distinct frames are painted in those three seconds; with the row of the task under way scrolled out of the page, and again with its document opened over the graph, no new frame is painted in two seconds; over a corpus with no task in progress no new frame is painted in two seconds on GRAPH; under TERM=dumb the four glyphs seen are . o O o; and every frame the painting session painted, read as characters, is a frame the NO_COLOR session painted too, so the painted frame and the plain one are identical character for character at every phase. The pseudo-terminal harness in crates/ank-tui/tests/terminal/mod.rs can wait for a frame settled modulo the pulse cells, and every test of crates/ank-tui/tests and of crates/ank-cli/tests/tui.rs that opens a frame where a task in progress pulses still passes. The module documentation of lib.rs, term.rs and view.rs no longer says that nothing in the crate is on a timer: it says the one clock paints the pulse, spawns no verb, renews nothing and stops when no in_progress row is on the screen. Every test of crates/ank-tui and crates/ank-cli/tests/tui.rs passes on Linux, measured by running cargo test --workspace in a rust:1.98-bookworm container and recorded with ank log. cargo test --workspace is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/b48661f0b90a@5d9a5b1
    tree: scope/577298166811
    criteria: ede175ea0159
    verifier: cargo-test@fcf98015afba
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@5d9a5b1
    tree: scope/577298166811
    criteria: ede175ea0159
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

The pulse of GRAPH option B: an in-progress row breathes, and the clock that
makes it breathe is allowed to do exactly one thing.

**Nothing forbids it, and two documents say it does not exist.** No accepted
ADR names a timer. What the corpus does forbid is a *read* on a clock:
ADR-8bd76e8d7c4e (the reader renews no claim on its own), ADR-0bb7ea8991bc (a
verb the holder runs renews the lease, so a reader polling `show` would hold
a claim for somebody who went home), and the stream module's whole argument
against a refresh loop (TASK-2f7777a1fdff). The module docs of lib.rs and
term.rs then say "nothing here is on a timer, and there is no timer to put
anything on", which was true and is a stronger sentence than the rule needed.
This task narrows the sentence to the rule: the pulse paints, and a clock
that paints spawns nothing, renews nothing and reads nothing -- which is what
ANK_TRACE_READS measures through the binary, the instrument TASK-643611645dea
and TASK-67583993ef0d already use for keys. The idle tests in
crates/ank-cli/tests/tui.rs measure refs and reads during a quiet session and
stay the measurement they were. No ADR is created, because none is crossed;
if the human wants the narrowed sentence binding, it is one `ank new adr`
away and this task's log will say so.

**Why performance is in the criterion.** The human saw lag in a first
prototype that redrew on every poll. The second prototype redraws only the
in-progress rows on a tick and nothing when none is visible, and that is
what felt right. So: no tick at all when no in-progress row is on the page,
at most four ticks a second, a frame per tick that differs only in the pulse
cells, and a key never queued behind a sleep. The loop already blocks on one
channel (`crate::session`) and the terminal reader already lives on a thread
(`term::typing`); the ticker is a third sender on the same channel, gated by
a flag App sets after each frame, so a key and a tick arrive in the order
they happened and nothing in the loop sleeps. The stream follower
(`stream::follow`) is the precedent for a thread on a quarter-second clock
that touches no corpus.

**Why time is injected and never read.** A test that watches wall time flakes
on a loaded runner, and a frame that reads a clock is a frame two runs cannot
compare. The counter in App is the clock the frame reads; the ticker only
increments it. The pseudo-terminal measurement counts frames and glyphs seen
inside a generous window and never asserts an interval; thirteen frames in
three seconds is twelve ticks and one settle, an upper bound a slow runner
can only undershoot. The `Live::frame` helper waits for two equal samples
200 ms apart and cannot settle on a pulsing screen; the harness gains a wait
that masks the pulse cells, and the suites that open a wide frame over a
corpus holding a claim (`Repo::seeded` in tui.rs holds one) are the ones to
check first.

**Painted equals plain, per phase.** Two sessions cannot share a tick, so the
measurement is over sets: every frame the painting session showed, read as
characters, is one the NO_COLOR session showed. ADR-559eebf5c6f5's clause
holds at every phase or it does not hold.

The visual reference is the prototype the human approved, run with
`--animate`:
C:/Users/nicoc/AppData/Local/Temp/claude/E--ClaudeRandom-AnkWork/cd1f51f4-51fb-4033-9e16-68f45f4fa7c0/scratchpad/graph_b.py
(`SPIN`, `TICK`, and `pulse`, which rewrites only the visible in-progress
rows).

Relies on: the screen task before this one (the in-progress cell at phase 0,
the junction layout, the fixture's task under way), TASK-fff0a98511b2 and
TASK-2f7777a1fdff (the session loop as an iterator of wakes and the
Painter seam its unit tests drive), TASK-643611645dea (ANK_TRACE_READS as the
instrument), TASK-367de5144982 (the container recipe in its log).
