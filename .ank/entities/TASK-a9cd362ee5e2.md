---
id: TASK-a9cd362ee5e2
type: task
slug: screen-3-is-graph-the-digit-the-ring-and-the-tit
title: "Screen 3 is GRAPH: the digit, the ring and the title move, and the document keeps no digit"
created: 2026-10-14T09:15:28Z
author: ankwork
status: in_progress
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/bindings.rs
  - crates/ank-tui/src/keys.rs
  - crates/ank-tui/src/input.rs
  - crates/ank-tui/tests/**
  - crates/ank-cli/tests/tui.rs
blocked_by: []
done_criteria: |
  Focus gains Graph, numbered 3 and named GRAPH on its title; the ring Tab and BackTab walk is CLAIMS, ENTITIES, GRAPH, QUEUE, and never lands on the document; Focus::of_digit('3') is Graph and no digit reaches Body. The binding of the key 3 is worded graph, the key list ? opens prints it, and the document is reached as before by Enter or Right on a row of any listing and left by Back, which returns to the listing it came from with the same row selected; its title is BODY (or the CONFIG and constraints headings it already carries) with no digit in front. Under DASHBOARD columns the region shows GRAPH when 3 is pressed; at DASHBOARD or wider the right panel is titled 3 GRAPH while no document is open in it and BODY once a row has been opened, and Back gives the right panel back to GRAPH; claims, entities and queue stay in the left column throughout and the cursor is drawn in exactly one panel. Until a later task draws the DAG, the GRAPH panel holds one sentence saying the graph has not been drawn yet, never a border around nothing. The module docs of view.rs and lib.rs describe the four screens as they now are, GRAPH third and the document reached by opening. Every test in crates/ank-tui/tests and in crates/ank-cli/tests/tui.rs that named 3 BODY is rewritten against the new titles, and the pseudo-terminal suite asserts at 80x24 and at 160x45 that: 3 reaches 3 GRAPH; Tab from ENTITIES reaches GRAPH, then QUEUE, then CLAIMS; Enter on an entity row opens BODY naming that entity; b returns to the listing with the row still under the cursor; and on the wide frame the title 3 GRAPH stands on the right panel at rest and BODY stands there after Enter. The frame drawn with paint and without it is still identical character for character. `cargo test --workspace` is green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Option A of the GRAPH screen: only the slot, the digit and the name of screen 3 move. The document keeps its place as what a row opens into (ADR-559eebf5c6f5: "Opening a row replaces the list with the document it names"), and loses the digit that reached it directly -- which a document never needed, since there is nothing to show in it until a row has been opened.

Why this is its own task and comes before the drawing: the eight test files that pin `3 BODY` (dashboard.rs, phone.rs, opening.rs, chrome.rs, colour.rs, press.rs, confirmation.rs, edit.rs, and two assertions in ank-cli/tests/tui.rs) are the whole of the risk, and they can be rewritten against a GRAPH panel that holds one sentence as well as against one that holds the DAG. Landing this first also keeps the merge small against the dashboard work another contributor is doing in view.rs (TASK-cae5c8ec9a69, ADR-ac6be1ebe9aa).

The wide frame is where the decision bites. TASK-cae5c8ec9a69 made the right panel the detail, drawn at rest as the anchor logo and "Enter opens the selected row". This task puts GRAPH there at rest and BODY there once a row is opened: the panel with nothing to show becomes the one listing that needs the width lanes cost. ADR-ac6be1ebe9aa (proposed, non-binding) says "beside a detail panel that shows the selected row"; it does not say what that panel shows while nothing is open, and the dashboard task filled it with a logo. If ADR-ac6be1ebe9aa is revised before ratification, one sentence saying the detail panel draws the graph while no document is open in it would close the gap.

Where Body sits in Focus::ALL and how the cursors array is indexed are the implementer's to settle: what the criterion fixes is what a key does and what a title says.
