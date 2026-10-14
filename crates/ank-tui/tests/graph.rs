//! Screen 3 is GRAPH, through the binary, at the desk and on the dashboard
//! (TASK-a9cd362ee5e2, ADR-559eebf5c6f5, ADR-ac6be1ebe9aa).
//!
//! The digit, the ring and the title moved: `3` reaches the graph, `Tab` walks
//! claims, entities, graph and queue, and the document is what a row opens
//! into, titled `BODY` with no digit in front. `src/view.rs` asserts each of
//! those of the render and of the key table on every platform; this asserts
//! them of `ank tui` on a pseudo-terminal, at the two windows the criterion
//! names -- eighty by twenty-four, which is under [`ank_tui::view::DASHBOARD`]
//! and draws one region, and a hundred and sixty by forty-five, which draws the
//! listings in a left column beside the right panel.
//!
//! `#[cfg(unix)]` for the reason every sibling suite gives: a pseudo-terminal
//! on Windows is ConPTY, and reaching it means the console API this workspace
//! does not otherwise call.

#![cfg(unix)]

mod terminal;

use terminal::{Live, Repo};

/// The desk, under the dashboard's width: one region.
const DESK: (u16, u16) = (80, 24);

/// The dashboard: the left column and the right panel beside it.
const WIDE: (u16, u16) = (160, 45);

/// The two windows the criterion names.
const WINDOWS: [(u16, u16); 2] = [DESK, WIDE];

/// Whether a window draws the dashboard.
fn wide(columns: u16) -> bool {
    columns >= ank_tui::view::DASHBOARD
}

/// What says a screen of the ring is the one keys reach, at this window.
///
/// Under the dashboard's width the region draws one screen, so its title is
/// the whole of it. On the dashboard every title of the ring is on every frame,
/// and the one with the focus is the one the marker stands in front of.
fn reached(columns: u16, title: &str) -> String {
    match wide(columns) {
        true => terminal::focused(title),
        false => title.to_string(),
    }
}

/// The session, open on the entities with the corpus read.
fn opened(live: &Live, columns: u16) -> String {
    live.until("the session to open", |t| {
        t.contains(&reached(columns, "2 ENTITIES")) && !t.contains(terminal::UNREAD)
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

/// **`3` reaches `3 GRAPH`, and what the graph holds is one sentence**, at both
/// windows (TASK-a9cd362ee5e2).
#[test]
fn three_reaches_the_graph() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        opened(&live, columns);
        live.send("3");
        let frame = live.until("'3' to reach the graph", |t| {
            t.contains(&reached(columns, "3 GRAPH"))
        });
        assert!(
            frame.contains(ank_tui::view::GRAPH_UNDRAWN),
            "the graph is a border around nothing at {columns}x{rows}:\n{frame}"
        );
        assert!(
            !frame.contains(terminal::BODY),
            "'3' reached the document at {columns}x{rows}:\n{frame}"
        );
        if !wide(columns) {
            assert!(
                !frame.contains("2 ENTITIES"),
                "the listing is drawn beside the graph at {columns}x{rows}:\n{frame}"
            );
        }
        live.quit();
    }
}

/// **`Tab` from the entities reaches the graph, then the queue, then the
/// claims**, at both windows (TASK-a9cd362ee5e2): the ring is claims,
/// entities, graph, queue, and it never lands on the document.
#[test]
fn tab_from_the_entities_walks_graph_queue_claims() {
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
                !frame.contains(terminal::BODY),
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
#[test]
fn enter_opens_the_document_and_b_gives_the_listing_back_on_the_same_row() {
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
        let short = terminal::short_of(&row);

        live.send("\r");
        let open = live.until("the document to open", |t| {
            t.contains(&terminal::body_naming(&short))
        });
        for n in 0..=9 {
            assert!(
                !open.contains(&format!("{n} BODY")),
                "the document carries a digit at {columns}x{rows}:\n{open}"
            );
        }

        live.send("b");
        let back = live.until("the listing to come back", |t| {
            !t.contains(terminal::BODY)
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
#[test]
fn on_the_wide_frame_the_right_panel_is_the_graph_and_then_the_document() {
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

    let short = terminal::short_of(&selected(&rest));
    live.send("\r");
    let open = live.until("the document to open beside the listings", |t| {
        t.contains(&terminal::body_naming(&short))
    });
    let body = column_of(&open, terminal::BODY).unwrap_or_else(|| panic!("no body:\n{open}"));
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
