//! Screen 3 is GRAPH, and it draws the `blocked_by` DAG as lanes, through
//! the binary, at the desk and on the dashboard (TASK-a9cd362ee5e2,
//! TASK-643611645dea, ADR-559eebf5c6f5, ADR-ac6be1ebe9aa).
//!
//! The digit, the ring and the title moved first: `3` reaches the graph,
//! `Tab` walks claims, entities, graph and queue, and the document is what a
//! row opens into, titled `BODY` with no digit in front. Then the graph was
//! drawn into the panel: one row per task shown, placed by the layout module
//! above everything it is blocked by, with the node's character saying what
//! the status word cannot and a row of it opening that task's document.
//! `src/view.rs` asserts each of those of the render and of the key table on
//! every platform; this asserts them of `ank tui` on a pseudo-terminal, at
//! the two windows the criterion names -- eighty by twenty-four, which is
//! under [`ank_tui::view::DASHBOARD`] and draws one region, and a hundred and
//! sixty by forty-five, which draws the listings in a left column beside the
//! right panel.
//!
//! **What a read costs is counted and never timed.** `ANK_TRACE_READS` is
//! pointed at an absolute path, every verb the reader spawns appends one
//! `index` line to it on a warm corpus (measured on 2026-10-14: `find`,
//! `graph`, `status` and `review` one each, `show` one plus an `entity` line
//! per file it parsed), and the file is compared before and after the keys.
//! A key that spawned a read would grow it; the keys that move a cursor
//! leave it byte for byte.
//!
//! Every session here is `#[cfg(unix)]` for the reason every sibling suite
//! gives: a pseudo-terminal on Windows is ConPTY, and reaching it means the
//! console API this workspace does not otherwise call. **The instrument that
//! reads the frame is not** (TASK-44e35bd40c83): [`graph_rows`] is pure text,
//! and it is held to a literal frame on every platform, so the one piece of
//! this suite that every PTY count rests on is measured where the suite was
//! written and not only where it runs.

// What only the sessions call is unused where the sessions are not compiled.
#![cfg_attr(not(unix), allow(dead_code))]

#[cfg(unix)]
use crate::terminal::{short_of, Graphed, Live, Repo, ONE_AT_A_TIME};

/// The desk, under the dashboard's width: one region.
const DESK: (u16, u16) = (80, 24);

/// The dashboard: the left column and the right panel beside it.
const WIDE: (u16, u16) = (160, 45);

/// The two windows the criterion names.
const WINDOWS: [(u16, u16); 2] = [DESK, WIDE];

/// The arrow keys, as a terminal sends them.
const DOWN: &str = "\x1b[B";
const UP: &str = "\x1b[A";

/// Whether a window draws the dashboard.
fn wide(columns: u16) -> bool {
    columns >= ank_tui::view::DASHBOARD
}

/// What says a screen of the ring is the one keys reach, at this window.
///
/// Under the dashboard's width the region draws one screen, so its title is
/// the whole of it. On the dashboard every title of the ring is on every frame,
/// and the one with the focus is the one the marker stands in front of.
#[cfg(unix)]
fn reached(columns: u16, title: &str) -> String {
    match wide(columns) {
        true => crate::terminal::focused(title),
        false => title.to_string(),
    }
}

/// The session, open on the entities with the corpus read.
#[cfg(unix)]
fn opened(live: &Live, columns: u16) -> String {
    live.until("the session to open", |t| {
        t.contains(&reached(columns, "2 ENTITIES")) && !t.contains(crate::terminal::UNREAD)
    });
    live.frame()
}

/// The row the cursor is on, by the identifier it carries.
///
/// The cursor is `> ` against a panel's left border, and only the panel that
/// has the focus draws one, so the first identifier after it is the row's.
fn selected(frame: &str) -> String {
    under_cursor(frame).unwrap_or_else(|| panic!("no row is under the cursor:\n{frame}"))
}

/// [`selected`], answering `None` rather than failing: a frame waited on may
/// be one the terminal has only half handed over.
fn under_cursor(frame: &str) -> Option<String> {
    let line = frame
        .lines()
        .find(|l| l.contains("\u{2503}>") || l.contains("\u{2502}>"))?;
    let at = line.find("\u{2503}>").or_else(|| line.find("\u{2502}>"))?;
    line[at..]
        .split_whitespace()
        .find(|word| {
            ["TASK-", "ADR-", "SPEC-", "LOG-"]
                .iter()
                .any(|kind| word.starts_with(kind))
        })
        .map(|word| word.to_string())
}

/// The column a title starts at, on whichever row carries it.
fn column_of(frame: &str, title: &str) -> Option<usize> {
    frame
        .lines()
        .find_map(|l| l.find(title).map(|at| l[..at].chars().count()))
}

/// Whether the graph is drawn on this frame: titled with its counts, and
/// not still being read.
fn graph_drawn(frame: &str) -> bool {
    frame.contains("3 GRAPH")
        && frame.contains("tasks in the corpus)")
        && !frame.contains("(reading)")
}

/// The graph, reached and drawn, with the focus on it.
///
/// Under the dashboard's width `3` puts it in the region and the arrival
/// reads it; on the dashboard it was read on the opening and `3` only moves
/// the focus onto it.
#[cfg(unix)]
fn on_graph(live: &mut Live, columns: u16) -> String {
    live.send("3");
    live.until("the graph to be drawn with the focus on it", |t| {
        t.contains(&reached(columns, "3 GRAPH")) && graph_drawn(t) && under_cursor(t).is_some()
    })
}

/// One row of the graph panel, as the panel holds it: the marker, the lane
/// field, the identifier, the status and the title, borders taken off.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphRow {
    /// Which line of the frame it is on.
    line: usize,
    /// The panel's own columns of it.
    text: String,
}

impl GraphRow {
    fn short(&self) -> String {
        self.text
            .split_whitespace()
            .find(|w| w.starts_with("TASK-"))
            .map(|w| w.to_string())
            .expect("a graph row names a task")
    }

    /// The lane field: what stands between the marker and the identifier,
    /// less the gap in front of the identifier.
    fn lanes(&self) -> String {
        let at = self.text.find("TASK-").expect("a graph row names a task");
        let before: String = self.text[..at].chars().skip(2).collect();
        before.trim_end().to_string()
    }

    /// How many lanes cross this row: the cells of the lane field that are
    /// not blank.
    fn lane_cells(&self) -> usize {
        self.lanes().chars().filter(|c| !c.is_whitespace()).count()
    }
}

/// The rows of the graph panel, top first.
///
/// **The panel is a rectangle, and a row is what lies inside it.** Under the
/// dashboard's width the panel is the region, against the window's left edge;
/// on the dashboard it is the right panel, found by the column its title
/// starts at -- the title is drawn three columns in from the panel's corner,
/// after the focus mark. Either way it runs to the window's right edge, so its
/// right border is the window's last column. A line is a row when a vertical
/// stands at the left border, and its text is every column between the two
/// borders.
///
/// Never the first vertical glyph after the border (TASK-44e35bd40c83): `│`
/// and `|` are the layout's Pass glyph as well as the border's, so a row
/// cut there loses the identifier behind a passing lane.
fn graph_rows(frame: &str, columns: u16) -> Vec<GraphRow> {
    let left = match wide(columns) {
        true => column_of(frame, "3 GRAPH").expect("the graph's title") - 3,
        false => 0,
    };
    let right = usize::from(columns) - 1;
    let vertical = |c: char| c == '\u{2503}' || c == '\u{2502}' || c == '|';
    frame
        .lines()
        .enumerate()
        .filter_map(|(line, l)| {
            let cells: Vec<char> = l.chars().collect();
            if !cells.get(left).is_some_and(|c| vertical(*c)) {
                return None;
            }
            let inside: String = cells[left + 1..right.min(cells.len())].iter().collect();
            inside.contains("TASK-").then(|| GraphRow {
                line,
                text: inside.trim_end().to_string(),
            })
        })
        .collect()
}

/// The row carrying one task.
#[cfg(unix)]
fn row_of<'a>(rows: &'a [GraphRow], id: &str) -> &'a GraphRow {
    let short = short_of(id);
    rows.iter()
        .find(|r| r.short() == short)
        .unwrap_or_else(|| panic!("{id} is not a row of the graph: {rows:?}"))
}

/// A dashboard frame, a hundred and twenty columns in the box-drawing
/// alphabet: the header, the rule, the entities in a left column forty-eight
/// wide, and the graph's panel beside them to the window's edge, focused,
/// with lanes passing through most of its rows.
const DRAWN_WIDE: &str = "\
ank  10 tasks   TASK-c3a3 is held
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
╭ 2 ENTITIES ──────────────────────────────────╮┏▶ 3 GRAPH all 10   (10 tasks in the corpus) ━━━━━━━━━━━━━━━━━━━━━━━━━━┓
│  TASK-a10e   open   A task with no edge      │┃> ●      TASK-a10e   open          A task with no edge                ┃
│  TASK-de9e   open   A dependent of the c     │┃  ●      TASK-de9e   open          A dependent of the closed one      ┃
│  TASK-b077   open   The bottom of a diam     │┃  │ ○─┐  TASK-b077   open          The bottom of a diamond            ┃
│                                              │┃  │ │ ○  TASK-4167   open          The right of a diamond             ┃
│                                              │┃  │ ○─┤  TASK-1ef7   open          The left of a diamond              ┃
│                                              │┃  │   ●  TASK-7090   open          The top of a diamond               ┃
│                                              │┃  │ ○    TASK-c3a3   open          The third of a chain               ┃
│                                              │┃  │ ○    TASK-c2a2   open          The second of a chain              ┃
│                                              │┃  │ ●    TASK-c1a1   open          The first of a chain               ┃
│                                              │┃  ·      TASK-c105   closed        A blocker that was closed          ┃
│                                              │┃                                                                      ┃
│                                              │┃                                                                      ┃
╰──────────────────────────────────────────────╯┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛";

/// The same graph at the desk, eighty columns, on a terminal that declared
/// itself dumb: one region from the window's left edge to its right, ruled in
/// ASCII, where a passing lane is the very `|` the border is drawn with.
const DRAWN_DESK: &str = "\
ank  10 tasks   TASK-c3a3 is held
--------------------------------------------------------------------------------
+> 3 GRAPH all 10   (10 tasks in the corpus) ==================================+
|> *      TASK-a10e   open          A task with no edge                        |
|  *      TASK-de9e   open          A dependent of the closed one              |
|  | o-+  TASK-b077   open          The bottom of a diamond                    |
|  | | o  TASK-4167   open          The right of a diamond                     |
|  | o-+  TASK-1ef7   open          The left of a diamond                      |
|  |   *  TASK-7090   open          The top of a diamond                       |
|  | o    TASK-c3a3   open          The third of a chain                       |
|  | o    TASK-c2a2   open          The second of a chain                      |
|  | *    TASK-c1a1   open          The first of a chain                       |
|  .      TASK-c105   closed        A blocker that was closed                  |
|                                                                              |
|                                                                              |
+==============================================================================+";

/// **The rows of the graph are found inside the panel's rectangle and never
/// end at the first vertical glyph** (TASK-44e35bd40c83), on every platform.
///
/// `│` and `|` are the border's and the layout's Pass glyph both, so a row
/// whose lane field carries a passing lane is the row an instrument cutting
/// at the first vertical loses. Two literal frames, one per alphabet and one
/// per arrangement, and every row, identifier and lane field of each comes
/// back -- while the left column's rows and the header, which name tasks too,
/// are not rows of the graph.
#[test]
fn the_graph_rows_are_found_inside_the_panel_whatever_lanes_pass_through_them() {
    let shorts = [
        "TASK-a10e",
        "TASK-de9e",
        "TASK-b077",
        "TASK-4167",
        "TASK-1ef7",
        "TASK-7090",
        "TASK-c3a3",
        "TASK-c2a2",
        "TASK-c1a1",
        "TASK-c105",
    ];
    let boxes = [
        "●",
        "●",
        "│ ○─┐",
        "│ │ ○",
        "│ ○─┤",
        "│   ●",
        "│ ○",
        "│ ○",
        "│ ●",
        "·",
    ];
    let ascii = [
        "*", "*", "| o-+", "| | o", "| o-+", "|   *", "| o", "| o", "| *", ".",
    ];
    for (frame, columns, lanes, third) in [
        (
            DRAWN_WIDE,
            120,
            boxes,
            "  │ ○─┐  TASK-b077   open          The bottom of a diamond",
        ),
        (
            DRAWN_DESK,
            80,
            ascii,
            "  | o-+  TASK-b077   open          The bottom of a diamond",
        ),
    ] {
        let rows = graph_rows(frame, columns);
        assert_eq!(
            rows.iter().map(GraphRow::short).collect::<Vec<_>>(),
            shorts,
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(
            rows.iter().map(|r| r.line).collect::<Vec<_>>(),
            (3..=12).collect::<Vec<_>>(),
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(
            rows.iter().map(GraphRow::lanes).collect::<Vec<_>>(),
            lanes,
            "at {columns} columns: {rows:?}"
        );
        assert_eq!(rows[2].text, third, "at {columns} columns");
    }
}

/// The lines of the trace file, which is what a read costs in the one unit
/// this suite counts.
fn trace(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// How many verbs a trace holds: one `index` line each on a warm corpus.
fn reads(lines: &[String]) -> usize {
    lines.iter().filter(|l| l.starts_with("index ")).count()
}

/// The lines `after` holds past `before`, which is what the keys between the
/// two readings cost.
fn added(before: &[String], after: &[String]) -> Vec<String> {
    assert!(
        after.starts_with(before),
        "the trace was rewritten rather than appended to:\n{before:?}\n{after:?}"
    );
    after[before.len()..].to_vec()
}

/// The scopes an entity declares, read off the `content` of what
/// `ank show --json` printed: the items under `scope:` in its frontmatter.
///
/// Written here rather than borrowed from the reader, because what is being
/// measured is the reader's cost against the verbs a person would run, and
/// an instrument that asked the reader which verbs to run would be measuring
/// the reader against itself.
fn declared_scopes(shown: &str) -> Vec<String> {
    let doc: serde_json::Value = serde_json::from_str(shown).expect("show printed JSON");
    let content = doc["content"].as_str().expect("show printed the content");
    let mut lines = content
        .lines()
        .skip(1)
        .take_while(|l| l.trim_end() != "---")
        .skip_while(|l| !l.starts_with("scope:"));
    match lines.next().map(|l| l["scope:".len()..].trim()) {
        None => Vec::new(),
        Some(flow) if flow.starts_with('[') => flow
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(_) => lines
            .map_while(|l| l.trim_start().strip_prefix("- "))
            .map(|item| item.trim().to_string())
            .collect(),
    }
}

/// **`3` reaches `3 GRAPH`, and what the graph holds is one row per open
/// task of the fixture, placed above everything it waits on**, at both
/// windows (TASK-a9cd362ee5e2, TASK-643611645dea).
///
/// Ten tasks and ten rows: nine open, and the closed blocker drawn because
/// its dependent is open, under that dependent. The row of the task with two
/// blockers leaves toward a second lane and the row under it carries both;
/// the two tasks sharing a blocker join one lane before the blocker's row,
/// which gathers none. And the title counts what is shown and what the corpus
/// holds, as the entities title does.
#[cfg(unix)]
#[test]
fn three_reaches_the_graph_which_draws_one_row_per_open_task_above_its_blockers() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        assert!(
            !frame.contains(crate::terminal::BODY),
            "'3' reached the document at {columns}x{rows}:\n{frame}"
        );
        if !wide(columns) {
            assert!(
                !frame.contains("2 ENTITIES"),
                "the listing is drawn beside the graph at {columns}x{rows}:\n{frame}"
            );
        }
        assert!(
            frame.contains("3 GRAPH all 10   (10 tasks in the corpus)"),
            "the title does not count the rows and the tasks at {columns}x{rows}:\n{frame}"
        );

        let drawn = graph_rows(&frame, columns);
        assert_eq!(
            drawn.len(),
            10,
            "ten rows at {columns}x{rows}: {drawn:?}\n{frame}"
        );
        for id in fixture.open() {
            assert_eq!(
                drawn.iter().filter(|r| r.short() == short_of(id)).count(),
                1,
                "{id} is not exactly one row at {columns}x{rows}:\n{frame}"
            );
        }
        let at = |id: &str| row_of(&drawn, id).line;

        // The closed blocker is drawn, and under its dependent.
        assert!(
            at(&fixture.dependent) < at(&fixture.closed),
            "the dependent is not above its closed blocker at {columns}x{rows}:\n{frame}"
        );
        // Every edge of the fixture, read the same way.
        let [chain_1, chain_2, chain_3] = &fixture.chain;
        let [top, left, right, bottom] = &fixture.diamond;
        for (task, blocker) in [
            (chain_2, chain_1),
            (chain_3, chain_2),
            (left, top),
            (right, top),
            (bottom, left),
            (bottom, right),
        ] {
            assert!(
                at(task) < at(blocker),
                "{task} is not above {blocker} at {columns}x{rows}:\n{frame}"
            );
        }
        // The lanes are read by column and never counted across the row: the
        // rows come in the listing's order, which within one second of
        // `created` is the order of the identifiers, so which other lanes pass
        // beside the diamond is the hash's to decide (TASK-44e35bd40c83).
        use ank_tui::graph::Glyph;
        let glyph = |g: Glyph| ank_tui::graph::Alphabet::Boxes.glyph(g);
        let leaves = [Glyph::LeaveRight, Glyph::LeaveLeft, Glyph::LeaveThrough].map(glyph);
        let joins = [Glyph::JoinRight, Glyph::JoinLeft, Glyph::JoinThrough].map(glyph);
        let cells = |r: &GraphRow| r.lanes().chars().collect::<Vec<char>>();
        // Two blockers: the row leaves to a second lane, and the row under it
        // carries both lanes, the node's own and the one it left toward.
        let bottom_row = row_of(&drawn, bottom);
        let bottom_cells = cells(bottom_row);
        let node = bottom_cells
            .iter()
            .position(|c| *c == glyph(Glyph::Waiting))
            .unwrap_or_else(|| panic!("the diamond's bottom has no node: {bottom_row:?}"));
        let corner = bottom_cells
            .iter()
            .position(|c| leaves.contains(c))
            .unwrap_or_else(|| {
                panic!(
                    "the diamond's bottom leaves toward no second lane at \
                     {columns}x{rows}: {:?}\n{frame}",
                    bottom_row.lanes()
                )
            });
        let under = drawn
            .iter()
            .find(|r| r.line == bottom_row.line + 1)
            .unwrap_or_else(|| panic!("no row under the diamond's bottom:\n{frame}"));
        for column in [node, corner] {
            assert!(
                cells(under).get(column).is_some_and(|c| *c != ' '),
                "the lane in column {column} stops under a task with two blockers: \
                 {:?} then {:?} at {columns}x{rows}:\n{frame}",
                bottom_row.lanes(),
                under.lanes()
            );
        }
        // One blocker shared: the second of the two tasks waiting on it joins
        // the lane already waiting, so the blocker's row gathers nothing.
        let second = match at(left) > at(right) {
            true => left,
            false => right,
        };
        assert!(
            row_of(&drawn, second)
                .lanes()
                .chars()
                .any(|c| joins.contains(&c)),
            "{second} does not join the lane already waiting for {top} at \
             {columns}x{rows}: {:?}\n{frame}",
            row_of(&drawn, second).lanes()
        );
        assert!(
            !row_of(&drawn, top)
                .lanes()
                .chars()
                .any(|c| joins.contains(&c) || leaves.contains(&c)),
            "the lanes did not join before the shared blocker's row: {:?} at \
             {columns}x{rows}:\n{frame}",
            row_of(&drawn, top).lanes()
        );
        live.quit();
    }
}

/// **The node character says what the status word cannot: Ready, Waiting or
/// Finished** (TASK-643611645dea), at both windows.
///
/// The status column keeps the word the CLI gave -- `open` on the dependent
/// whose blocker is closed and on the chain's third, which waits -- and the
/// cell on the lane tells them apart.
#[cfg(unix)]
#[test]
fn the_node_character_says_ready_waiting_or_finished() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    let glyph = |g: ank_tui::graph::Glyph| ank_tui::graph::Alphabet::Boxes.glyph(g);
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        let drawn = graph_rows(&frame, columns);
        let cell = |id: &str, g: ank_tui::graph::Glyph, word: &str| {
            let row = row_of(&drawn, id);
            assert!(
                row.lanes().contains(glyph(g)),
                "{id} does not carry {:?} ({g:?}) at {columns}x{rows}: {:?}\n{frame}",
                glyph(g),
                row.text
            );
            assert!(
                row.text.contains(&format!("  {word}  ")),
                "{id} lost the CLI's word {word:?}: {:?}\n{frame}",
                row.text
            );
        };
        cell(&fixture.lone, ank_tui::graph::Glyph::Ready, "open");
        cell(&fixture.chain[0], ank_tui::graph::Glyph::Ready, "open");
        cell(&fixture.dependent, ank_tui::graph::Glyph::Ready, "open");
        cell(&fixture.chain[2], ank_tui::graph::Glyph::Waiting, "open");
        cell(&fixture.diamond[3], ank_tui::graph::Glyph::Waiting, "open");
        cell(&fixture.closed, ank_tui::graph::Glyph::Finished, "closed");
        live.quit();
    }
}

/// **The frame painted and the frame drawn with `NO_COLOR` are identical
/// character for character** (TASK-643611645dea, ADR-1f70ce2c3eac), at both
/// windows.
#[cfg(unix)]
#[test]
fn the_painted_graph_and_the_plain_one_are_the_same_characters() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut painted = Live::painting(&fixture.repo, columns, rows);
        opened(&painted, columns);
        let with = on_graph(&mut painted, columns);
        painted.quit();
        let mut plain = Live::open(&fixture.repo, columns, rows);
        opened(&plain, columns);
        let without = on_graph(&mut plain, columns);
        plain.quit();
        assert_eq!(
            with, without,
            "the painted graph says something the plain one does not at {columns}x{rows}"
        );
        assert_eq!(
            graph_rows(&with, columns).len(),
            10,
            "the frames compared hold no graph:\n{with}"
        );
    }
}

/// **On a terminal declaring `TERM=dumb` the lanes and nodes are the ASCII
/// alphabet the layout module declares** (TASK-643611645dea,
/// ADR-559eebf5c6f5), at both windows.
#[cfg(unix)]
#[test]
fn a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::dumb(&fixture.repo, columns, rows);
        live.until("the session to open", |t| {
            t.contains("2 ENTITIES") && !t.contains(crate::terminal::UNREAD)
        });
        live.send("3");
        let frame = live.until("the graph to be drawn", |t| {
            t.contains("3 GRAPH") && graph_drawn(t) && t.contains("> ")
        });
        let drawn = graph_rows(&frame, columns);
        assert_eq!(drawn.len(), 10, "ten rows in ASCII:\n{frame}");
        for (glyph, boxes, ascii) in ank_tui::graph::TABLE {
            assert!(
                !frame.contains(boxes),
                "{glyph:?} is drawn as {boxes:?} on a dumb terminal:\n{frame}"
            );
            if matches!(
                glyph,
                ank_tui::graph::Glyph::Ready
                    | ank_tui::graph::Glyph::Waiting
                    | ank_tui::graph::Glyph::Finished
                    | ank_tui::graph::Glyph::Pass
            ) {
                assert!(
                    drawn.iter().any(|r| r.lanes().contains(ascii)),
                    "{glyph:?} is never drawn as {ascii:?}:\n{frame}"
                );
            }
        }
        live.quit();
    }
}

/// **j, k, Space, g and the arrows move the cursor over the rows; Enter on
/// a row opens that task's document and Back returns to GRAPH with the same
/// row selected; and none of it, nor a resize, spawns a graph read**
/// (TASK-643611645dea), at both windows.
///
/// Measured with `ANK_TRACE_READS`: on a wide frame the graph is read on
/// the opening, in the pass after `find` has answered and together with the
/// claims and the queue -- four reads, and a frame that already carried the
/// rows while the graph still said it was being read. Under that width the
/// opening costs one read and the arrival at the graph one more; a second
/// arrival costs nothing. Every movement leaves the trace byte for byte.
/// Enter costs exactly what the document costs to read: one `ank show`, and
/// one `ank scope` for every scope the task declares, which is how the
/// document finds the constraints binding it -- read off those verbs run by
/// hand (TASK-44e35bd40c83). `u` reads the graph again, in the pass it runs,
/// and the document still held after Back with it.
#[cfg(unix)]
#[test]
fn the_cursor_walks_the_rows_enter_opens_a_task_and_nothing_but_u_reads_the_graph_again() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    fixture.repo.warm_find();
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("reads-{columns}.trace"));
        let _ = std::fs::remove_file(&trace_path);
        let traced = trace_path.to_str().expect("a UTF-8 path");
        let mut live = Live::with(&fixture.repo, columns, rows, &[("ANK_TRACE_READS", traced)]);
        let first = opened(&live, columns);
        if wide(columns) {
            // Read after `find` answered: a frame existed with the rows on it
            // and the graph still being read.
            live.ever("carried the rows while the graph was being read", |t| {
                t.contains("A task with no edge") && t.contains("3 GRAPH   (reading)")
            });
            let frame = live.until("the opening's second pass to land", |t| {
                graph_drawn(t) && !t.contains("not asked") && !t.contains("reading")
            });
            assert_eq!(
                reads(&trace(&trace_path)),
                4,
                "the opening of a wide frame is find, status, review and graph:\n{:?}\n{frame}",
                trace(&trace_path)
            );
        } else {
            assert!(
                !first.contains("3 GRAPH"),
                "the graph is drawn under the dashboard's width at rest:\n{first}"
            );
            assert_eq!(
                reads(&trace(&trace_path)),
                1,
                "the opening of a narrow frame is one find:\n{:?}",
                trace(&trace_path)
            );
        }
        let before_arrival = trace(&trace_path);
        let frame = on_graph(&mut live, columns);
        let after_arrival = trace(&trace_path);
        assert_eq!(
            reads(&added(&before_arrival, &after_arrival)),
            usize::from(!wide(columns)),
            "arriving at the graph costs one read under the width and none on it:\n{:?}",
            after_arrival
        );
        let drawn = graph_rows(&frame, columns);
        assert_eq!(drawn.len(), 10, "{frame}");
        let at = |n: usize| drawn[n].short();
        assert_eq!(selected(&frame), at(0), "the graph opens on its first row");

        let settled = trace(&trace_path);
        let moved = |live: &mut Live, key: &str, lands: &str| -> String {
            live.send(key);
            let frame = live.until(&format!("{key:?} to put the cursor on {lands}"), |t| {
                under_cursor(t).as_deref() == Some(lands)
            });
            assert_eq!(
                trace(&trace_path),
                settled,
                "{key:?} spawned a read at {columns}x{rows}"
            );
            frame
        };
        moved(&mut live, "j", &at(1));
        moved(&mut live, DOWN, &at(2));
        moved(&mut live, "k", &at(1));
        moved(&mut live, UP, &at(0));
        // A page is longer than the ten rows, so Space lands on the last.
        moved(&mut live, " ", &at(9));
        moved(&mut live, "g", &at(0));
        moved(&mut live, "j", &at(1));
        moved(&mut live, "j", &at(2));

        // The window, narrowed, and the next frame drawn to it.
        let narrower = columns - 10;
        live.resize(narrower, rows);
        let resized = moved(&mut live, "k", &at(1));
        let edge = resized
            .lines()
            .find(|l| l.contains("3 GRAPH"))
            .and_then(|l| l.chars().nth(narrower as usize - 1));
        assert!(
            edge.is_some_and(|c| c == '\u{2513}' || c == '\u{256e}'),
            "the frame was not drawn at {narrower} columns (edge {edge:?}):\n{resized}"
        );
        live.resize(columns, rows);
        moved(&mut live, "j", &at(2));

        // Enter opens the row's document, and costs one `show` and one
        // `scope` for every scope the task declares, which is how the document
        // finds the constraints binding it, and nothing else: measured against
        // the same verbs run by hand (TASK-44e35bd40c83).
        let row = at(2);
        let full = fixture
            .open()
            .into_iter()
            .map(String::from)
            .chain(std::iter::once(fixture.closed.clone()))
            .find(|id| short_of(id) == row)
            .expect("the row names a task of the fixture");
        let by_hand = fixture.repo.0.join(format!("show-{columns}.trace"));
        let _ = std::fs::remove_file(&by_hand);
        let hand_traced = [("ANK_TRACE_READS", by_hand.to_str().expect("a UTF-8 path"))];
        let shown = fixture.repo.tried(&["show", &full, "--json"], &hand_traced);
        assert!(shown.status.success(), "show by hand: {shown:?}");
        // Read off what `show` printed, and held to what the fixture wrote:
        // one scope, `src/**`, on every task.
        let globs = declared_scopes(&String::from_utf8_lossy(&shown.stdout));
        assert_eq!(globs, ["src/**"], "the scopes {full} declares: {shown:?}");
        for glob in &globs {
            let scoped = fixture.repo.tried(&["scope", glob, "--json"], &hand_traced);
            assert!(scoped.status.success(), "scope {glob} by hand: {scoped:?}");
        }
        let before_enter = trace(&trace_path);
        live.send("\r");
        let open = live.until("the document to open", |t| {
            t.contains(&crate::terminal::body_naming(&row))
        });
        let mut cost = added(&before_enter, &trace(&trace_path));
        let mut hand = trace(&by_hand);
        cost.sort();
        hand.sort();
        assert_eq!(
            cost, hand,
            "Enter cost something other than one show and a scope for each of \
             {globs:?} at {columns}x{rows}:\n{open}"
        );
        for n in 0..=9 {
            assert!(
                !open.contains(&format!("{n} BODY")),
                "the document carries a digit:\n{open}"
            );
        }

        // Back returns to the graph with the same row under the cursor, and
        // reads nothing.
        let before_back = trace(&trace_path);
        live.send("b");
        let back = live.until("the graph to come back", |t| {
            !t.contains(crate::terminal::BODY)
                && t.contains(&reached(columns, "3 GRAPH"))
                && under_cursor(t).is_some()
        });
        assert_eq!(selected(&back), row, "Back moved the cursor:\n{back}");
        assert_eq!(
            trace(&trace_path),
            before_back,
            "Back spawned a read at {columns}x{rows}"
        );

        // A second arrival reads nothing: the graph is held.
        live.send("2");
        live.until("the entities to have the focus", |t| {
            t.contains(&reached(columns, "2 ENTITIES"))
        });
        on_graph(&mut live, columns);
        assert_eq!(
            trace(&trace_path),
            before_back,
            "a second arrival at the graph read it again at {columns}x{rows}"
        );

        // `u` reads it again, in its pass: `find` and the graph under the
        // width, and the claims and the queue beside them on it. And the
        // document Enter opened is still held after Back, so the reload reads
        // it again too, as it does wherever one is held: the `show` and the
        // scopes run by hand above, once more (TASK-44e35bd40c83).
        // Waited on until the whole pass has landed, then settled before it
        // is counted: the old graph stays drawn while the pass runs, so a
        // frame that says "drawn" says nothing about which read is last, and a
        // read past the pass has had the settling to land in.
        let before_u = trace(&trace_path);
        let pass = if wide(columns) { 4 } else { 2 };
        let document = reads(&hand);
        live.send("u");
        live.until("the reload to land", |t| {
            reads(&trace(&trace_path)) >= reads(&before_u) + pass + document && graph_drawn(t)
        });
        let reloaded = live.frame();
        let cost = added(&before_u, &trace(&trace_path));
        assert_eq!(
            reads(&cost),
            pass + document,
            "u did not read the graph again, once, beside the {document} read(s) of the \
             document it holds, at {columns}x{rows}: {cost:?}\n{reloaded}"
        );
        live.quit();
    }
}

/// **A tap selects the row under it and a second tap on it opens it**
/// (TASK-643611645dea, ADR-559eebf5c6f5), at both windows.
#[cfg(unix)]
#[test]
fn a_tap_selects_a_graph_row_and_a_second_tap_opens_it() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&fixture.repo, columns, rows);
        opened(&live, columns);
        let frame = on_graph(&mut live, columns);
        let drawn = graph_rows(&frame, columns);
        let third = &drawn[2];
        let left = match wide(columns) {
            true => column_of(&frame, "3 GRAPH").expect("the graph's title") - 3,
            false => 0,
        } as u16;
        live.tap(left + 4, third.line as u16);
        let chosen = live.until("the tap to select the third row", |t| {
            under_cursor(t).as_deref() == Some(third.short().as_str())
        });
        assert!(
            !chosen.contains(crate::terminal::BODY),
            "one tap opened the row at {columns}x{rows}:\n{chosen}"
        );
        live.tap(left + 4, third.line as u16);
        live.until("the second tap to open the row", |t| {
            t.contains(&crate::terminal::body_naming(&third.short()))
        });
        live.quit();
    }
}

/// **`Tab` from the entities reaches the graph, then the queue, then the
/// claims**, at both windows (TASK-a9cd362ee5e2): the ring is claims,
/// entities, graph, queue, and it never lands on the document.
#[cfg(unix)]
#[test]
fn tab_from_the_entities_walks_graph_queue_claims() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        opened(&live, columns);
        for lands in ["3 GRAPH", "4 QUEUE", "1 CLAIMS"] {
            live.send("\t");
            let frame = live.until(&format!("Tab to reach {lands}"), |t| {
                t.contains(&reached(columns, lands))
            });
            assert!(
                !frame.contains(crate::terminal::BODY),
                "Tab reached the document at {columns}x{rows}:\n{frame}"
            );
        }
        live.quit();
    }
}

/// **Enter on an entity row opens `BODY` naming that entity, and `b` returns
/// to the listing with the row still under the cursor**, at both windows
/// (TASK-a9cd362ee5e2, TASK-252bf02de218).
///
/// The row is moved off the first one before it is opened, so a reader that
/// gave the listing back with its cursor reset could not pass.
#[cfg(unix)]
#[test]
fn enter_opens_the_document_and_b_gives_the_listing_back_on_the_same_row() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        let first = opened(&live, columns);
        let top = selected(&first);
        live.send("j");
        let moved = live.until("the cursor to move off the first row", |t| {
            under_cursor(t).is_some_and(|row| row != top)
        });
        let row = selected(&moved);
        let short = crate::terminal::short_of(&row);

        live.send("\r");
        let open = live.until("the document to open", |t| {
            t.contains(&crate::terminal::body_naming(&short))
        });
        for n in 0..=9 {
            assert!(
                !open.contains(&format!("{n} BODY")),
                "the document carries a digit at {columns}x{rows}:\n{open}"
            );
        }

        live.send("b");
        let back = live.until("the listing to come back", |t| {
            !t.contains(crate::terminal::BODY)
                && t.contains(&reached(columns, "2 ENTITIES"))
                && under_cursor(t).is_some()
        });
        assert_eq!(
            selected(&back),
            row,
            "the listing came back with a different row selected at \
             {columns}x{rows}:\n{back}"
        );
        live.quit();
    }
}

/// **On the wide frame the title `3 GRAPH` stands on the right panel at rest,
/// and `BODY` stands there after Enter** (TASK-a9cd362ee5e2).
///
/// The right panel is read off the column a title starts at: the listings'
/// titles open against the window's left edge, and the right panel's opens
/// past the column of listings, whatever width the layout gave that column.
#[cfg(unix)]
#[test]
fn on_the_wide_frame_the_right_panel_is_the_graph_and_then_the_document() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let (columns, rows) = WIDE;
    let mut live = Live::open(&repo, columns, rows);
    let rest = opened(&live, columns);
    let quarter = columns as usize / 4;
    for title in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE"] {
        let at = column_of(&rest, title).unwrap_or_else(|| panic!("no {title}:\n{rest}"));
        assert!(at < quarter, "{title} is not in the left column:\n{rest}");
    }
    let graph = column_of(&rest, "3 GRAPH").unwrap_or_else(|| panic!("no graph:\n{rest}"));
    assert!(
        graph > quarter,
        "3 GRAPH is not on the right panel at rest:\n{rest}"
    );

    let short = crate::terminal::short_of(&selected(&rest));
    live.send("\r");
    let open = live.until("the document to open beside the listings", |t| {
        t.contains(&crate::terminal::body_naming(&short))
    });
    let body =
        column_of(&open, crate::terminal::BODY).unwrap_or_else(|| panic!("no body:\n{open}"));
    assert!(
        body > quarter,
        "BODY is not on the right panel after Enter:\n{open}"
    );
    assert!(
        !open.contains("3 GRAPH"),
        "the graph is still drawn beside the document:\n{open}"
    );
    for title in ["1 CLAIMS", "2 ENTITIES", "4 QUEUE"] {
        let at = column_of(&open, title).unwrap_or_else(|| panic!("no {title}:\n{open}"));
        assert!(at < quarter, "{title} left the left column:\n{open}");
    }
    live.quit();
}

/// The fixture is what it says it is, read back through the CLI rather than
/// trusted: ten tasks, nine open, one closed, and the seven edges.
#[cfg(unix)]
#[test]
fn the_graphed_fixture_is_a_chain_a_diamond_a_closed_blocker_and_a_lone_task() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let fixture = Repo::graphed();
    let doc = fixture.repo.stdout(&["graph", "--json"]);
    let ids = crate::terminal::ids_of(&doc);
    assert_eq!(ids.len(), 10, "{doc}");
    assert_eq!(doc.matches("\"status\":\"open\"").count(), 9, "{doc}");
    assert_eq!(doc.matches("\"status\":\"closed\"").count(), 1, "{doc}");
    let edge = |task: &str, blocked_by: &str| {
        format!("\"task\":\"{task}\",\"blocked_by\":\"{blocked_by}\"")
    };
    let [chain_1, chain_2, chain_3] = &fixture.chain;
    let [top, left, right, bottom] = &fixture.diamond;
    for (task, blocker) in [
        (chain_2, chain_1),
        (chain_3, chain_2),
        (left, top),
        (right, top),
        (bottom, left),
        (bottom, right),
        (&fixture.dependent, &fixture.closed),
    ] {
        assert!(
            doc.contains(&edge(task, blocker)),
            "no edge {task} <- {blocker}:\n{doc}"
        );
    }
    assert_eq!(doc.matches("\"blocked_by\":").count(), 7, "{doc}");
    let _: &Graphed = &fixture;
}

// ---------------------------------------------------------------------------
// The fold, and the search, on GRAPH (TASK-67583993ef0d)
// ---------------------------------------------------------------------------

/// The title of the task [`historied`] finishes.
const HISTORY: &str = "A task finished with nothing waiting on it";

/// Escape, as a terminal sends it.
const ESCAPE: &str = "\u{1b}";

/// [`Repo::graphed`], and one task more: finished through `done` and named
/// by nothing, which is the history the graph folds away at rest.
///
/// Built here rather than in the fixture because the fixture's every task is
/// on the graph at rest -- its one closed task has an open dependent -- and
/// a fold measured over a corpus with nothing to fold would be green about
/// nothing. `done` and not `close`, because the criterion says *done*: a
/// claim, a commit for the proof to name, and the verb.
#[cfg(unix)]
fn historied() -> (Graphed, String) {
    let fixture = Repo::graphed();
    let repo = &fixture.repo;
    let made = repo.stdout(&[
        "new",
        "task",
        "--title",
        HISTORY,
        "--scope",
        "src/**",
        "--criteria",
        "The graph folds this row away until the fold is all.",
        "--json",
    ]);
    let ids = crate::terminal::ids_of(&made);
    assert_eq!(ids.len(), 1, "one task was written: {made}");
    let history = ids[0].clone();
    repo.ank(&["claim", &history]);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "the work the finished task proves"]);
    let sha = String::from_utf8_lossy(&repo.git(&["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();
    repo.ank(&["done", "--proof", &format!("commit:{sha}")]);
    let doc = repo.stdout(&["graph", "--json"]);
    assert!(
        doc.contains(&format!("\"id\":\"{history}\""))
            && doc.matches("\"status\":\"done\"").count() == 1,
        "the history is not one done task:\n{doc}"
    );
    (fixture, history)
}

/// Every task the corpus holds: the fixture's and the history.
#[cfg(unix)]
fn every_task(fixture: &Graphed, history: &str) -> Vec<String> {
    fixture
        .open()
        .into_iter()
        .map(String::from)
        .chain([fixture.closed.clone(), history.to_string()])
        .collect()
}

/// The key that cycles the kind elsewhere and the fold on GRAPH, read out of
/// the binding table rather than written here.
fn cycle_key() -> String {
    let binding = ank_tui::bindings::BINDINGS
        .iter()
        .find(|b| b.runs == ank_tui::bindings::Runs::Press(ank_tui::keys::Press::Cycle))
        .expect("a row of the table cycles the kind");
    match binding.key {
        ratatui::crossterm::event::KeyCode::Char(c) => c.to_string(),
        other => panic!("the cycle is bound to {other:?}, which this suite cannot type"),
    }
}

/// The cell the header draws for a fold, as the reader spells it.
fn cell(fold: ank_tui::bindings::Fold) -> String {
    ank_tui::view::fold_target(fold)
}

/// The header's first row, which is where the cell is drawn.
fn header(frame: &str) -> &str {
    frame.lines().next().unwrap_or_default()
}

/// The identifiers of the graph's rows, top first.
fn shorts(frame: &str, columns: u16) -> Vec<String> {
    graph_rows(frame, columns)
        .iter()
        .map(GraphRow::short)
        .collect()
}

/// The graph reached on a session traced by `ANK_TRACE_READS`, with every
/// read the opening asked for landed, and the trace at that moment.
#[cfg(unix)]
fn settled_on_graph(
    fixture: &Graphed,
    columns: u16,
    rows: u16,
    trace_path: &std::path::Path,
) -> (Live, String) {
    let _ = std::fs::remove_file(trace_path);
    let traced = trace_path.to_str().expect("a UTF-8 path");
    let mut live = Live::with(&fixture.repo, columns, rows, &[("ANK_TRACE_READS", traced)]);
    opened(&live, columns);
    if wide(columns) {
        live.until("the opening's second pass to land", |t| {
            graph_drawn(t) && !t.contains("not asked") && !t.contains("reading")
        });
    }
    let frame = on_graph(&mut live, columns);
    (live, frame)
}

/// **At rest the graph holds no row for a done task nothing open waits on;
/// the key that cycles the kind elsewhere shows every task, one row each,
/// with the title saying all and the header cell saying the fold; the key
/// again gives the frame at rest back; and none of it reads the graph or
/// moves the cursor off its row** (TASK-67583993ef0d), at both windows.
#[cfg(unix)]
#[test]
fn the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (fixture, history) = historied();
    fixture.repo.warm_find();
    let key = cycle_key();
    let every = every_task(&fixture, &history);
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("fold-{columns}.trace"));
        let (mut live, rest) = settled_on_graph(&fixture, columns, rows, &trace_path);
        let at_rest = shorts(&rest, columns);
        assert_eq!(
            at_rest.len(),
            10,
            "ten rows at rest at {columns}x{rows}:\n{rest}"
        );
        assert!(
            !at_rest.contains(&short_of(&history)),
            "the done task nothing waits on is a row at rest at {columns}x{rows}:\n{rest}"
        );
        assert!(
            header(&rest).contains(&cell(ank_tui::bindings::Fold::Open)),
            "the header cell does not say the fold at rest at {columns}x{rows}:\n{rest}"
        );
        assert!(
            rest.contains("3 GRAPH all 10   (11 tasks in the corpus)"),
            "\n{rest}"
        );

        // The cursor off the first row, so a fold that reset it is seen.
        live.send("j");
        live.send("j");
        let moved = live.until("the cursor to reach the third row", |t| {
            under_cursor(t).as_deref() == Some(at_rest[2].as_str())
        });
        let on = selected(&moved);
        let settled = trace(&trace_path);

        live.send(&key);
        let all = live.until("the fold to open to every task", |t| {
            t.contains("3 GRAPH all 11   (11 tasks in the corpus)")
                && header(t).contains(&cell(ank_tui::bindings::Fold::All))
                && graph_rows(t, columns).len() == 11
        });
        let unfolded = shorts(&all, columns);
        for id in &every {
            assert_eq!(
                unfolded.iter().filter(|s| **s == short_of(id)).count(),
                1,
                "{id} is not exactly one row after the key at {columns}x{rows}:\n{all}"
            );
        }
        assert_eq!(
            selected(&all),
            on,
            "the fold moved the cursor off its row at {columns}x{rows}:\n{all}"
        );

        live.send(&key);
        let back = live.until("the fold to come back to the open work", |t| {
            header(t).contains(&cell(ank_tui::bindings::Fold::Open))
                && graph_rows(t, columns).len() == 10
        });
        assert_eq!(
            graph_rows(&back, columns)
                .into_iter()
                .map(|r| r.text)
                .collect::<Vec<_>>(),
            graph_rows(&moved, columns)
                .into_iter()
                .map(|r| r.text)
                .collect::<Vec<_>>(),
            "the second press is not the graph as it was at {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            header(&back),
            header(&moved),
            "the header is not as it was at {columns}x{rows}"
        );
        assert_eq!(
            trace(&trace_path),
            settled,
            "changing the fold spawned a read at {columns}x{rows}"
        );
        live.quit();
    }
}

/// **A search for the title of one task leaves that task and only it on the
/// screen, with the edges to what it hid counted as outside; Escape gives
/// the fold back; and none of it reads the graph** (TASK-67583993ef0d), at
/// both windows.
///
/// Typed as a person types it, the search key and then the title. On the
/// dashboard the listing beside the graph narrows on the same needle, so
/// "only it on the screen" is held over the whole frame: no other task of
/// the corpus is named anywhere on it.
#[cfg(unix)]
#[test]
fn a_search_narrows_the_graph_to_one_task_and_escape_gives_the_fold_back() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (fixture, history) = historied();
    fixture.repo.warm_find();
    let every = every_task(&fixture, &history);
    let left = fixture.diamond[1].clone();
    let title = "The left of a diamond";
    for (columns, rows) in WINDOWS {
        let trace_path = fixture.repo.0.join(format!("search-{columns}.trace"));
        let (mut live, rest) = settled_on_graph(&fixture, columns, rows, &trace_path);
        let at_rest = shorts(&rest, columns);
        let settled = trace(&trace_path);

        live.send(&format!("{}{title}", ank_tui::keys::FIND));
        let narrowed = live.until("the graph to narrow to one row", |t| {
            t.contains(&format!("{}{title}", ank_tui::view::SEARCH))
                && graph_rows(t, columns).len() == 1
        });
        assert_eq!(
            shorts(&narrowed, columns),
            [short_of(&left)],
            "the search left another row at {columns}x{rows}:\n{narrowed}"
        );
        for id in every.iter().filter(|id| **id != left) {
            assert!(
                !narrowed.contains(&short_of(id)),
                "{id} is still on the screen at {columns}x{rows}:\n{narrowed}"
            );
        }
        let row = &graph_rows(&narrowed, columns)[0];
        assert_eq!(
            row.lane_cells(),
            1,
            "the lanes were not laid out over the one survivor at {columns}x{rows}: {:?}",
            row.text
        );
        // Seventy-seven columns of row in a region seventy-eight wide, so the
        // count is whole at the desk as well as on the dashboard.
        assert!(
            row.text.contains("(+1 blocker(s) outside)"),
            "the hidden blocker is not counted on the row at {columns}x{rows}: {:?}\n{narrowed}",
            row.text
        );

        live.send(ESCAPE);
        let back = live.until("the fold to come back", |t| {
            graph_rows(t, columns).len() == at_rest.len()
                && !t.contains(&format!("{}{title}", ank_tui::view::SEARCH))
        });
        assert_eq!(
            shorts(&back, columns),
            at_rest,
            "Escape did not give the fold back at {columns}x{rows}:\n{back}"
        );
        assert!(
            !back.contains("outside") && !back.contains("matching"),
            "the fold came back still narrowed at {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            selected(&back),
            short_of(&left),
            "Escape moved the cursor off the row it was on at {columns}x{rows}:\n{back}"
        );
        assert_eq!(
            trace(&trace_path),
            settled,
            "the search spawned a read at {columns}x{rows}"
        );
        live.quit();
    }
}
