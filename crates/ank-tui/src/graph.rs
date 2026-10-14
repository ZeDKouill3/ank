//! The lane layout of the `blocked_by` DAG: which row each task takes, which
//! lane it sits in, and the characters drawn left of its title.
//!
//! **Pure arithmetic, and nothing else.** This module names no terminal, no
//! corpus and no screen: its input is a list of [`Node`]s the caller built,
//! its output a [`Layout`] of plain strings. What is shown, in which order the
//! ties break, and what colour any of it is painted belong to the view
//! (ADR-559eebf5c6f5); a module that only emits characters cannot make the
//! frame with paint and without it differ.
//!
//! # The walk
//!
//! The reference is `git log --graph`. The rows come in topological order, a
//! node above every node it is blocked by the way a commit sits above its
//! parents, and wherever the edges leave two nodes free of each other the
//! caller's order is kept. One pass over those rows then keeps a vector of
//! lanes, each lane waiting for the id it will reach:
//!
//! - a node takes the lane waiting for it, or else the first free lane;
//! - its first blocker continues that lane;
//! - each further blocker takes the first free lane, and the edge *leaves* the
//!   node sideways toward it;
//! - but a blocker some lane already waits for is not given a second one: the
//!   edge *joins* that lane, which is the merge. This holds for the first
//!   blocker too, so no two lanes ever wait for one id and a node never has to
//!   gather lanes arriving from above;
//! - a lane freed is reused, and trailing free lanes are dropped, so the width
//!   is what the rows need.
//!
//! **One row per node**, so a connector is drawn on the row of the node it
//! leaves: the node's cell, a horizontal across to the lane it reaches, and
//! there either a corner turning down into a new lane or a tee into a lane
//! that already runs. Lane `i` is drawn in column `2 * i`, the columns between
//! lanes carry the horizontals, and every prefix of a layout is
//! `2 * lanes - 1` columns wide.
//!
//! An edge to an id that is no node draws nothing and is counted on its row
//! ([`Row::unknown`]). A cycle is broken where it is met -- at the earliest
//! node in the caller's order that nothing outside its own cycle still waits
//! on -- so the walk terminates, every node is still a row, and the edge back
//! up to a node already drawn draws nothing.
//!
//! # The junction layout
//!
//! [`junction_layout`] is the second shape, the one GRAPH option B chose
//! (TASK-080f158e317e), standing beside [`layout`] until the view moves onto
//! it and the first one is removed. The rows come in the same order; what
//! changes is where an edge meets its lane. A node's further blockers open
//! new lanes to its right (`─┐`, `┬`), and lanes waiting for one node merge
//! on a row of their own immediately above it (`├─┘`, `┴`), a row that is
//! nobody's: a [`Line::Junction`]. A horizontal passing over a lane crosses
//! it (`┼`). Its cells say open, finished or in progress ([`Mark::cell`]),
//! readiness being a word for the view to write and not a cell.
//!
//! [`groups`] answers which nodes are linked at all: the weakly connected
//! components over the drawn edges, for the view to give each its block.

use std::collections::{BTreeSet, HashMap};

/// Where a task stands, as the graph draws its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Nothing stands in its way.
    Ready,
    /// Something it is blocked by is not finished.
    Waiting,
    /// Done, or otherwise closed.
    Finished,
    /// Claimed and under way. The junction layout draws it as one phase of a
    /// pulse the caller chooses ([`Mark::cell`]); the first layout, which
    /// has no phase, draws the first.
    InProgress,
}

/// One node, as the caller hands it in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The identifier the edges name.
    pub id: String,
    /// The cell drawn for it.
    pub mark: Mark,
    /// The ids it is blocked by. An id that is no node of the input draws no
    /// lane and is counted in [`Row::unknown`].
    pub blocked_by: Vec<String>,
}

/// One node, placed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The node's id.
    pub id: String,
    /// The node's mark.
    pub mark: Mark,
    /// The lane its cell sits in; its column in [`Row::prefix`] is
    /// `2 * lane`.
    pub lane: usize,
    /// What is drawn left of the title, the layout's width in columns
    /// ([`Layout::width`], [`JunctionLayout::width`]), one `char` per column.
    pub prefix: String,
    /// How many of its `blocked_by` name an id that is no node.
    pub unknown: usize,
}

/// The rows, one per node, top first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// One per node of the input.
    pub rows: Vec<Row>,
    /// The number of lanes the layout used: the most any one row needed.
    pub lanes: usize,
}

impl Layout {
    /// The columns every prefix takes: `2 * lanes - 1`, and 0 when there is
    /// no row.
    pub fn width(&self) -> usize {
        (2 * self.lanes).saturating_sub(1)
    }
}

/// The two alphabets a layout is drawn in (ADR-559eebf5c6f5): box-drawing
/// where the terminal can, ASCII where it has declared it cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alphabet {
    Boxes,
    Ascii,
}

/// What a column of a prefix can mean. The blank between lanes is a space in
/// both alphabets and is not a glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// The cell of a node marked [`Mark::Ready`].
    Ready,
    /// The cell of a node marked [`Mark::Waiting`].
    Waiting,
    /// The cell of a node marked [`Mark::Finished`].
    Finished,
    /// A lane passing through the row, crossed or not.
    Pass,
    /// The horizontal of an edge on its way to another lane.
    Across,
    /// An edge leaves the node rightwards and turns down into a new lane.
    LeaveRight,
    /// An edge leaves the node leftwards and turns down into a new lane.
    LeaveLeft,
    /// A new lane opens down while the horizontal runs on past it.
    LeaveThrough,
    /// An edge from the left joins a lane already waiting for its blocker.
    JoinRight,
    /// An edge from the right joins a lane already waiting for its blocker.
    JoinLeft,
    /// A lane is joined while the horizontal runs on past it.
    JoinThrough,
    // The junction layout's own rows. Pass, Across, LeaveRight and
    // LeaveThrough above mean the same there and are shared; the rest are
    // appended so the first layout's rows stand as they were.
    /// The cell of a node not finished and not under way, ready or waiting
    /// alike: readiness is the word on the row, not the cell.
    Open,
    /// The cell of a finished node: filled, because it is settled.
    Settled,
    /// The cell of a node under way, at the first phase of its pulse.
    InProgress0,
    /// The second phase of the pulse.
    InProgress1,
    /// The third phase of the pulse.
    InProgress2,
    /// The fourth phase of the pulse.
    InProgress3,
    /// On a junction row, the lane the node takes, which the others merge
    /// into.
    JoinKeep,
    /// On a junction row, the farthest lane merging.
    JoinFar,
    /// On a junction row, a lane merging while the horizontal runs on past
    /// it.
    JoinMiddle,
    /// A lane passing where a horizontal crosses it.
    Cross,
}

/// **The one table**: each glyph, then its box-drawing and its ASCII
/// character, in [`Glyph`]'s order. Every character is one column wide.
pub const TABLE: [(Glyph, char, char); 21] = [
    (Glyph::Ready, '●', '*'),
    (Glyph::Waiting, '○', 'o'),
    (Glyph::Finished, '·', '.'),
    (Glyph::Pass, '│', '|'),
    (Glyph::Across, '─', '-'),
    (Glyph::LeaveRight, '┐', '+'),
    (Glyph::LeaveLeft, '┌', '+'),
    (Glyph::LeaveThrough, '┬', '+'),
    (Glyph::JoinRight, '┤', '+'),
    (Glyph::JoinLeft, '├', '+'),
    (Glyph::JoinThrough, '┼', '+'),
    (Glyph::Open, '○', 'o'),
    (Glyph::Settled, '●', '@'),
    (Glyph::InProgress0, '·', '.'),
    (Glyph::InProgress1, '•', 'o'),
    (Glyph::InProgress2, '●', 'O'),
    (Glyph::InProgress3, '•', 'o'),
    (Glyph::JoinKeep, '├', '|'),
    (Glyph::JoinFar, '┘', '\''),
    (Glyph::JoinMiddle, '┴', '+'),
    (Glyph::Cross, '┼', '+'),
];

impl Alphabet {
    /// The character this alphabet draws for `glyph`.
    pub fn glyph(self, glyph: Glyph) -> char {
        let (_, boxes, ascii) = TABLE[glyph as usize];
        match self {
            Alphabet::Boxes => boxes,
            Alphabet::Ascii => ascii,
        }
    }
}

impl Mark {
    /// The cell the first layout draws.
    fn glyph(self) -> Glyph {
        match self {
            Mark::Ready => Glyph::Ready,
            Mark::Waiting => Glyph::Waiting,
            Mark::Finished => Glyph::Finished,
            Mark::InProgress => Glyph::InProgress0,
        }
    }

    /// The cell the junction layout draws: open for ready and waiting alike,
    /// settled when finished, and for a node under way the pulse at `phase`,
    /// taken modulo four. Which phase, and when it moves, are the caller's.
    pub fn cell(self, phase: usize) -> Glyph {
        const PULSE: [Glyph; 4] = [
            Glyph::InProgress0,
            Glyph::InProgress1,
            Glyph::InProgress2,
            Glyph::InProgress3,
        ];
        match self {
            Mark::Ready | Mark::Waiting => Glyph::Open,
            Mark::Finished => Glyph::Settled,
            Mark::InProgress => PULSE[phase % 4],
        }
    }
}

/// How an edge reaches the lane of its blocker on the node's row.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reach {
    Leave,
    Join,
}

/// Lays `nodes` out, top first, drawn in `alphabet`.
pub fn layout(nodes: &[Node], alphabet: Alphabet) -> Layout {
    let (blockers, unknown) = edges(nodes);

    let mut lanes: Vec<Option<usize>> = Vec::new();
    let mut drawn = vec![false; nodes.len()];
    let mut placed: Vec<(usize, usize, Vec<Option<Glyph>>)> = Vec::with_capacity(nodes.len());
    let mut most = 0;
    for x in order(&blockers) {
        drawn[x] = true;
        let lane = match lanes.iter().position(|w| *w == Some(x)) {
            Some(l) => l,
            None => free(&mut lanes, 0),
        };
        lanes[lane] = None;
        let through = lanes.clone();
        let mut reached: Vec<(usize, Reach)> = Vec::new();
        for &b in &blockers[x] {
            if drawn[b] {
                // Back up a cycle, to a row already drawn: nothing to draw.
                continue;
            }
            if let Some(t) = lanes.iter().position(|w| *w == Some(b)) {
                reached.push((t, Reach::Join));
            } else if lanes[lane].is_none() {
                lanes[lane] = Some(b);
            } else {
                let t = free(&mut lanes, 0);
                lanes[t] = Some(b);
                reached.push((t, Reach::Leave));
            }
        }
        let cells = row(lane, nodes[x].mark, &through, &reached, lanes.len());
        most = most.max(cells.len().div_ceil(2));
        while lanes.last() == Some(&None) {
            lanes.pop();
        }
        placed.push((x, lane, cells));
    }

    let width = (2 * most).saturating_sub(1);
    let rows = placed
        .into_iter()
        .map(|(x, lane, cells)| {
            let mut prefix: String = cells
                .iter()
                .map(|c| c.map_or(' ', |g| alphabet.glyph(g)))
                .collect();
            prefix.extend(std::iter::repeat_n(' ', width - cells.len()));
            Row {
                id: nodes[x].id.clone(),
                mark: nodes[x].mark,
                lane,
                prefix,
                unknown: unknown[x],
            }
        })
        .collect();
    Layout { rows, lanes: most }
}

/// One line of a [`JunctionLayout`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// A node's row.
    Task(Row),
    /// The lanes waiting for the node on the next line merging into one:
    /// a prefix and nobody's row.
    Junction(String),
}

impl Line {
    /// What is drawn left of the title, or the whole of a junction row.
    pub fn prefix(&self) -> &str {
        match self {
            Line::Task(row) => &row.prefix,
            Line::Junction(prefix) => prefix,
        }
    }
}

/// The lines of the junction layout, top first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JunctionLayout {
    /// One [`Line::Task`] per node of the input, and a [`Line::Junction`]
    /// immediately above each node two or more lanes wait for.
    pub lines: Vec<Line>,
    /// The number of lanes the layout used: the most any one line needed.
    pub lanes: usize,
}

impl JunctionLayout {
    /// The columns every prefix takes: `2 * lanes - 1`, and 0 when there is
    /// no line.
    pub fn width(&self) -> usize {
        (2 * self.lanes).saturating_sub(1)
    }
}

/// Lays `nodes` out the way `git log --graph` does, top first, drawn in
/// `alphabet`, a node under way drawn at pulse `phase`.
///
/// The rows come in the order [`layout`] draws them. One pass keeps a vector
/// of lanes, each waiting for the id it will reach:
///
/// - a node takes the leftmost lane waiting for it, or else the first free
///   lane; when two or more lanes wait for it they merge into that one on a
///   junction row standing immediately above its task row;
/// - its first blocker whose edge is drawn continues that lane;
/// - each further blocker opens the first free lane right of the node, never
///   left of it and never into a lane already waiting, even one waiting for
///   the same blocker: that merge is the blocker's junction row, below;
/// - trailing free lanes are dropped after every row.
///
/// An edge to an id that is no node, and an edge back up a cycle to a row
/// already drawn, draw nothing, as in [`layout`].
pub fn junction_layout(nodes: &[Node], alphabet: Alphabet, phase: usize) -> JunctionLayout {
    let (blockers, unknown) = edges(nodes);
    let mut lanes: Vec<Option<usize>> = Vec::new();
    let mut drawn = vec![false; nodes.len()];
    // Per line, the node it is the row of (None on a junction row), the
    // node's lane, and its cells.
    let mut placed: Vec<(Option<usize>, usize, Vec<Option<Glyph>>)> = Vec::new();
    for x in order(&blockers) {
        drawn[x] = true;
        let mine: Vec<usize> = (0..lanes.len()).filter(|&k| lanes[k] == Some(x)).collect();
        let lane = match mine.first() {
            Some(&l) => l,
            None => free(&mut lanes, 0),
        };
        if mine.len() > 1 {
            placed.push((None, lane, merge(&mine, &lanes)));
            for &k in &mine[1..] {
                lanes[k] = None;
            }
        }
        lanes[lane] = None;
        let through = lanes.clone();
        let mut opened: Vec<usize> = Vec::new();
        let mut continued = false;
        for &b in &blockers[x] {
            if drawn[b] {
                // Back up a cycle, to a row already drawn: nothing to draw.
                continue;
            }
            if !continued {
                lanes[lane] = Some(b);
                continued = true;
            } else {
                let t = free(&mut lanes, lane + 1);
                lanes[t] = Some(b);
                opened.push(t);
            }
        }
        let cell = nodes[x].mark.cell(phase);
        placed.push((Some(x), lane, leave(lane, cell, &through, &opened, &lanes)));
        while lanes.last() == Some(&None) {
            lanes.pop();
        }
    }

    let most = placed
        .iter()
        .map(|(_, _, cells)| cells.len().div_ceil(2))
        .max()
        .unwrap_or(0);
    let width = (2 * most).saturating_sub(1);
    let lines = placed
        .into_iter()
        .map(|(x, lane, cells)| {
            let mut prefix: String = cells
                .iter()
                .map(|c| c.map_or(' ', |g| alphabet.glyph(g)))
                .collect();
            prefix.extend(std::iter::repeat_n(' ', width - cells.len()));
            match x {
                None => Line::Junction(prefix),
                Some(x) => Line::Task(Row {
                    id: nodes[x].id.clone(),
                    mark: nodes[x].mark,
                    lane,
                    prefix,
                    unknown: unknown[x],
                }),
            }
        })
        .collect();
    JunctionLayout { lines, lanes: most }
}

/// The weakly connected groups of `nodes` over the edges that name a node,
/// as indices into `nodes`: each group lists its members in the caller's
/// order, the groups stand in the order of their first member, and a node
/// with no edge to any node is a group of one. How the groups are ordered on
/// a screen is the caller's.
pub fn groups(nodes: &[Node]) -> Vec<Vec<usize>> {
    let (blockers, _) = edges(nodes);
    // A union over the edges, each set named by its root.
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    fn root(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for (x, bs) in blockers.iter().enumerate() {
        for &b in bs {
            let (rx, rb) = (root(&mut parent, x), root(&mut parent, b));
            parent[rx.max(rb)] = rx.min(rb);
        }
    }
    let mut slot: HashMap<usize, usize> = HashMap::new();
    let mut out: Vec<Vec<usize>> = Vec::new();
    for x in 0..nodes.len() {
        let r = root(&mut parent, x);
        let g = *slot.entry(r).or_insert_with(|| {
            out.push(Vec::new());
            out.len() - 1
        });
        out[g].push(x);
    }
    out
}

/// The cells of a junction row: `mine` the lanes waiting for the node, left
/// to right, the first of them the one it takes, and `lanes` every lane as
/// it stands above the row.
fn merge(mine: &[usize], lanes: &[Option<usize>]) -> Vec<Option<Glyph>> {
    let (keep, far) = (mine[0], mine[mine.len() - 1]);
    let mut cells = Vec::with_capacity(2 * lanes.len());
    for (c, waits) in lanes.iter().enumerate() {
        if c > 0 {
            cells.push((keep < c && c <= far).then_some(Glyph::Across));
        }
        let runs = waits.is_some();
        cells.push(if c == keep {
            Some(Glyph::JoinKeep)
        } else if c == far {
            Some(Glyph::JoinFar)
        } else if mine.contains(&c) {
            Some(Glyph::JoinMiddle)
        } else if keep < c && c < far {
            Some(if runs { Glyph::Cross } else { Glyph::Across })
        } else {
            runs.then_some(Glyph::Pass)
        });
    }
    cells
}

/// The cells of a node's task row: `lane` is the node's and `cell` its
/// glyph, `through` the lanes running past it, `opened` the lanes its
/// further blockers open to its right, and `after` every lane as it stands
/// below the row.
fn leave(
    lane: usize,
    cell: Glyph,
    through: &[Option<usize>],
    opened: &[usize],
    after: &[Option<usize>],
) -> Vec<Option<Glyph>> {
    let far = opened.iter().copied().max().unwrap_or(lane);
    let count = after
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |k| k + 1)
        .max(lane + 1);
    let runs = |c: usize| through.get(c).is_some_and(Option::is_some);
    let mut cells = Vec::with_capacity(2 * count);
    for c in 0..count {
        if c > 0 {
            cells.push((lane < c && c <= far).then_some(Glyph::Across));
        }
        cells.push(if c == lane {
            Some(cell)
        } else if c == far {
            Some(Glyph::LeaveRight)
        } else if opened.contains(&c) {
            Some(Glyph::LeaveThrough)
        } else if lane < c && c < far {
            Some(if runs(c) { Glyph::Cross } else { Glyph::Across })
        } else {
            runs(c).then_some(Glyph::Pass)
        });
    }
    cells
}

/// Per node, the nodes it is blocked by, each once and in the order named,
/// and how many of its `blocked_by` name an id that is no node. A repeated
/// id counts once; two nodes sharing an id are reached through the first.
fn edges(nodes: &[Node]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut index: HashMap<&str, usize> = HashMap::new();
    for (i, n) in nodes.iter().enumerate() {
        index.entry(n.id.as_str()).or_insert(i);
    }
    let mut blockers: Vec<Vec<usize>> = Vec::with_capacity(nodes.len());
    let mut unknown: Vec<usize> = Vec::with_capacity(nodes.len());
    for n in nodes {
        let mut named: Vec<&str> = Vec::new();
        let (mut known, mut missing) = (Vec::new(), 0);
        for b in &n.blocked_by {
            if named.contains(&b.as_str()) {
                continue;
            }
            named.push(b);
            match index.get(b.as_str()) {
                Some(&i) if !known.contains(&i) => known.push(i),
                Some(_) => {}
                None => missing += 1,
            }
        }
        blockers.push(known);
        unknown.push(missing);
    }
    (blockers, unknown)
}

/// The first free lane at `from` or right of it, made at the right edge
/// when there is none.
fn free(lanes: &mut Vec<Option<usize>>, from: usize) -> usize {
    match (from..lanes.len()).find(|&k| lanes[k].is_none()) {
        Some(l) => l,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    }
}

/// The cells of one node's row: `lane` is the node's, `through` the lanes
/// running past it, `reached` the lanes its edges leave for or join, and
/// `after` how many lanes exist below it.
fn row(
    lane: usize,
    mark: Mark,
    through: &[Option<usize>],
    reached: &[(usize, Reach)],
    after: usize,
) -> Vec<Option<Glyph>> {
    let count = after.max(through.len()).max(lane + 1);
    let left = reached.iter().map(|&(t, _)| t).filter(|&t| t < lane).min();
    let right = reached.iter().map(|&(t, _)| t).filter(|&t| t > lane).max();
    let spanned = |c: usize| {
        left.is_some_and(|l| l <= c && c < lane) || right.is_some_and(|r| lane < c && c <= r)
    };
    let runs = |c: usize| through.get(c).is_some_and(Option::is_some);
    let mut cells = Vec::with_capacity(2 * count - 1);
    for c in 0..count {
        if c > 0 {
            let across = left.is_some_and(|l| l < c && c <= lane)
                || right.is_some_and(|r| lane < c && c <= r);
            cells.push(across.then_some(Glyph::Across));
        }
        let glyph = if c == lane {
            Some(mark.glyph())
        } else if let Some(&(_, how)) = reached.iter().find(|&&(t, _)| t == c) {
            let far = Some(c) == left || Some(c) == right;
            Some(match (how, far, c > lane) {
                (Reach::Leave, true, true) => Glyph::LeaveRight,
                (Reach::Leave, true, false) => Glyph::LeaveLeft,
                (Reach::Leave, false, _) => Glyph::LeaveThrough,
                (Reach::Join, true, true) => Glyph::JoinRight,
                (Reach::Join, true, false) => Glyph::JoinLeft,
                (Reach::Join, false, _) => Glyph::JoinThrough,
            })
        } else if runs(c) {
            Some(Glyph::Pass)
        } else if spanned(c) {
            Some(Glyph::Across)
        } else {
            None
        };
        cells.push(glyph);
    }
    cells
}

/// The rows' order: every node above the nodes it is blocked by, and the
/// input order wherever the edges leave it free.
///
/// **The walk waits on components, not on nodes** (TASK-afcd4f0f56ad). A
/// cycle is one strongly connected component, and a node on it is reached by
/// whatever reaches any other node of it, so a component is *open* -- its
/// nodes may be drawn at all -- only once every other component with an edge
/// into it has been drawn whole. Inside an open component a node is free once
/// its dependents in that component are drawn; where none is free the walk is
/// inside a cycle, and it is broken at the earliest node in the input of an
/// open component. Picking merely the earliest node left would be wrong: it
/// can be a node on no cycle at all, sitting below one, and drawing it then
/// puts it above its own dependents.
///
/// Waiting per node was wrong twice over. Breaking at a node nothing outside
/// its cycle waits on *directly*, and freeing a node once its own direct
/// dependents are drawn, both let a node through while another node of its
/// dependent's cycle still reached it: with 0 and 1 a cycle, 2 and 3 another,
/// and 2 blocked by 0, 1 was drawn above 2, which reaches it through 0. With
/// no cycle every component is one node, drawing it finishes it, and the walk
/// is the plain topological one in input order.
fn order(blockers: &[Vec<usize>]) -> Vec<usize> {
    let n = blockers.len();
    let cycle = components(blockers);
    // Indexed by component, which numbers at most one per node: its nodes,
    // and how many of them are not drawn yet.
    let mut members = vec![Vec::new(); n];
    for (x, &c) in cycle.iter().enumerate() {
        members[c].push(x);
    }
    let mut left: Vec<usize> = members.iter().map(Vec::len).collect();
    // Per node, its dependents in its own component not drawn yet; per
    // component, the edges into it from components not drawn whole yet.
    let mut inner = vec![0usize; n];
    let mut waiting = vec![0usize; n];
    for (x, bs) in blockers.iter().enumerate() {
        for &b in bs {
            match cycle[b] == cycle[x] {
                true => inner[b] += 1,
                false => waiting[cycle[b]] += 1,
            }
        }
    }
    let mut placed = vec![false; n];
    let mut free: BTreeSet<usize> = (0..n)
        .filter(|&i| waiting[cycle[i]] == 0 && inner[i] == 0)
        .collect();
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let x = match free.pop_first() {
            Some(x) => x,
            None => (0..n)
                .find(|&i| !placed[i] && waiting[cycle[i]] == 0)
                .expect("a cycle that nothing outside it waits on"),
        };
        placed[x] = true;
        out.push(x);
        let c = cycle[x];
        for &b in &blockers[x] {
            if cycle[b] == c {
                inner[b] -= 1;
                if inner[b] == 0 && !placed[b] {
                    free.insert(b);
                }
            }
        }
        left[c] -= 1;
        if left[c] > 0 {
            continue;
        }
        // The component is drawn whole: what it waited on may open.
        for &y in &members[c] {
            for &b in &blockers[y] {
                let d = cycle[b];
                if d == c {
                    continue;
                }
                waiting[d] -= 1;
                if waiting[d] == 0 {
                    free.extend(members[d].iter().copied().filter(|&z| inner[z] == 0));
                }
            }
        }
    }
    out
}

/// The strongly connected components of the `blocked_by` edges, as one
/// number per node: two nodes share it when each reaches the other, which is
/// what being on one cycle means. Kosaraju's two passes, iteratively, so a
/// long chain costs no stack.
fn components(blockers: &[Vec<usize>]) -> Vec<usize> {
    let n = blockers.len();
    let mut finished = Vec::with_capacity(n);
    let mut seen = vec![false; n];
    for start in 0..n {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![(start, 0)];
        while let Some(&(v, i)) = stack.last() {
            if let Some(&w) = blockers[v].get(i) {
                stack.last_mut().expect("a frame").1 += 1;
                if !seen[w] {
                    seen[w] = true;
                    stack.push((w, 0));
                }
            } else {
                finished.push(v);
                stack.pop();
            }
        }
    }
    let mut dependents = vec![Vec::new(); n];
    for (x, bs) in blockers.iter().enumerate() {
        for &b in bs {
            dependents[b].push(x);
        }
    }
    let mut component = vec![usize::MAX; n];
    let mut count = 0;
    for &root in finished.iter().rev() {
        if component[root] != usize::MAX {
            continue;
        }
        component[root] = count;
        let mut stack = vec![root];
        while let Some(v) = stack.pop() {
            for &d in &dependents[v] {
                if component[d] == usize::MAX {
                    component[d] = count;
                    stack.push(d);
                }
            }
        }
        count += 1;
    }
    component
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, mark: Mark, blocked_by: &[&str]) -> Node {
        Node {
            id: id.to_string(),
            mark,
            blocked_by: blocked_by.iter().map(|b| b.to_string()).collect(),
        }
    }

    fn ready(id: &str, blocked_by: &[&str]) -> Node {
        node(id, Mark::Ready, blocked_by)
    }

    /// The rows as (id, lane, prefix), the shape every literal below is
    /// written in.
    fn shape(layout: &Layout) -> Vec<(&str, usize, &str)> {
        layout
            .rows
            .iter()
            .map(|r| (r.id.as_str(), r.lane, r.prefix.as_str()))
            .collect()
    }

    #[test]
    fn a_chain_of_three_is_one_lane_with_each_dependent_above_its_blocker() {
        // Handed in bottom first, to show the order is the edges' and not the
        // input's where the edges decide it.
        let nodes = [ready("C", &[]), ready("B", &["C"]), ready("A", &["B"])];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(shape(&laid), [("A", 0, "●"), ("B", 0, "●"), ("C", 0, "●")]);
        assert_eq!(laid.lanes, 1);
    }

    fn diamond() -> [Node; 4] {
        [
            ready("D", &[]),
            ready("C", &["D"]),
            ready("A", &["B", "C"]),
            ready("B", &["D"]),
        ]
    }

    #[test]
    fn a_diamond_leaves_toward_a_second_lane_and_joins_the_lane_waiting_for_the_same_blocker() {
        // A is the only node nothing waits on, so it is first; B and C are
        // free of each other and keep the input's order, C before B. A's first
        // blocker B continues lane 0 and C leaves for lane 1. C's edge to D
        // continues lane 1; B's edge to D finds lane 1 already waiting for D
        // and joins it, so D is reached by one lane, and it is lane 1.
        let laid = layout(&diamond(), Alphabet::Boxes);
        assert_eq!(
            shape(&laid),
            [
                ("A", 0, "●─┐"),
                ("C", 1, "│ ●"),
                ("B", 0, "●─┤"),
                ("D", 1, "  ●")
            ]
        );
        assert_eq!(laid.lanes, 2);
    }

    #[test]
    fn a_node_cell_is_its_mark_in_either_alphabet() {
        let nodes = [
            node("R", Mark::Ready, &[]),
            node("W", Mark::Waiting, &[]),
            node("F", Mark::Finished, &[]),
        ];
        let boxes: Vec<String> = layout(&nodes, Alphabet::Boxes)
            .rows
            .into_iter()
            .map(|r| r.prefix)
            .collect();
        assert_eq!(boxes, ["●", "○", "·"]);
        let ascii: Vec<String> = layout(&nodes, Alphabet::Ascii)
            .rows
            .into_iter()
            .map(|r| r.prefix)
            .collect();
        assert_eq!(ascii, ["*", "o", "."]);
    }

    #[test]
    fn the_diamond_in_ascii_is_the_same_layout_in_other_glyphs() {
        let laid = layout(&diamond(), Alphabet::Ascii);
        assert_eq!(
            shape(&laid),
            [
                ("A", 0, "*-+"),
                ("C", 1, "| *"),
                ("B", 0, "*-+"),
                ("D", 1, "  *")
            ]
        );
        assert_eq!(laid.lanes, 2);
    }

    #[test]
    fn two_independent_roots_keep_the_input_order_on_one_lane() {
        let nodes = [ready("Y", &[]), ready("X", &[])];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(shape(&laid), [("Y", 0, "●"), ("X", 0, "●")]);
        assert_eq!(laid.lanes, 1);
    }

    #[test]
    fn two_independent_chains_interleave_in_input_order_on_two_lanes() {
        let nodes = [
            ready("A1", &["A2"]),
            ready("B1", &["B2"]),
            ready("A2", &[]),
            ready("B2", &[]),
        ];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(
            shape(&laid),
            [
                ("A1", 0, "●  "),
                ("B1", 1, "│ ●"),
                ("A2", 0, "● │"),
                ("B2", 1, "  ●"),
            ]
        );
        assert_eq!(laid.lanes, 2);
    }

    #[test]
    fn a_node_blocked_by_seven_opens_six_lanes_beside_its_own() {
        let nodes = [
            ready("X", &["B1", "B2", "B3", "B4", "B5", "B6", "B7"]),
            ready("B1", &[]),
            ready("B2", &[]),
            ready("B3", &[]),
            ready("B4", &[]),
            ready("B5", &[]),
            ready("B6", &[]),
            ready("B7", &[]),
        ];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(
            shape(&laid),
            [
                ("X", 0, "●─┬─┬─┬─┬─┬─┐"),
                ("B1", 0, "● │ │ │ │ │ │"),
                ("B2", 1, "  ● │ │ │ │ │"),
                ("B3", 2, "    ● │ │ │ │"),
                ("B4", 3, "      ● │ │ │"),
                ("B5", 4, "        ● │ │"),
                ("B6", 5, "          ● │"),
                ("B7", 6, "            ●"),
            ]
        );
        assert_eq!(laid.lanes, 7);
    }

    #[test]
    fn a_cycle_of_two_terminates_with_both_nodes_as_rows() {
        // Neither is free of the other; the earlier in the input goes first,
        // and B's edge back up to A draws nothing.
        let nodes = [ready("A", &["B"]), ready("B", &["A"])];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(shape(&laid), [("A", 0, "●"), ("B", 0, "●")]);
        assert_eq!(laid.lanes, 1);
    }

    /// Two cycles, the second blocked by the first through one edge: 2 waits
    /// on 0. Every node of the second cycle reaches every node of the first,
    /// 1 included through 0, so the whole of it stands above the whole of the
    /// first -- the break waits on the component and not on the node, or 1,
    /// which nothing outside its cycle waits on directly, is drawn above 2.
    #[test]
    fn a_cycle_blocked_by_another_cycle_stands_wholly_above_it() {
        let nodes = [
            ready("N0", &["N1"]),
            ready("N1", &["N0"]),
            ready("N2", &["N3", "N0"]),
            ready("N3", &["N2"]),
        ];
        let laid = layout(&nodes, Alphabet::Boxes);
        let ids: Vec<&str> = laid.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["N2", "N3", "N0", "N1"]);
    }

    #[test]
    fn an_edge_to_an_id_outside_the_set_draws_no_lane_and_is_counted() {
        let nodes = [ready("A", &["GONE", "B", "LOST"]), ready("B", &[])];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(shape(&laid), [("A", 0, "●"), ("B", 0, "●")]);
        assert_eq!(laid.lanes, 1);
        let unknown: Vec<usize> = laid.rows.iter().map(|r| r.unknown).collect();
        assert_eq!(unknown, [2, 0]);
    }

    #[test]
    fn an_edge_leaves_leftward_into_a_lane_freed_above() {
        // Q frees lane 0; U, on lane 1, keeps its lane for V and sends W
        // left into the free one.
        let nodes = [
            ready("P", &["Q"]),
            ready("R", &["U"]),
            ready("Q", &[]),
            ready("U", &["V", "W"]),
            ready("V", &[]),
            ready("W", &[]),
        ];
        let laid = layout(&nodes, Alphabet::Boxes);
        assert_eq!(
            shape(&laid),
            [
                ("P", 0, "●  "),
                ("R", 1, "│ ●"),
                ("Q", 0, "● │"),
                ("U", 1, "┌─●"),
                ("V", 1, "│ ●"),
                ("W", 0, "●  "),
            ]
        );
    }

    #[test]
    fn a_join_crosses_a_running_lane_and_a_middle_join_is_a_cross() {
        // E, on lane 2, joins lane 0 for B across lane 1, which keeps its
        // vertical.
        let crossing = [
            ready("A", &["B"]),
            ready("C", &["D"]),
            ready("E", &["B"]),
            ready("B", &[]),
            ready("D", &[]),
        ];
        let laid = layout(&crossing, Alphabet::Boxes);
        assert_eq!(
            shape(&laid),
            [
                ("A", 0, "●    "),
                ("C", 1, "│ ●  "),
                ("E", 2, "├─│─●"),
                ("B", 0, "● │  "),
                ("D", 1, "  ●  "),
            ]
        );
        // The same E blocked by D as well joins lane 1 on the way.
        let through = [
            ready("A", &["B"]),
            ready("C", &["D"]),
            ready("E", &["D", "B"]),
            ready("B", &[]),
            ready("D", &[]),
        ];
        let laid = layout(&through, Alphabet::Boxes);
        assert_eq!(laid.rows[2].prefix, "├─┼─●");
        assert_eq!(layout(&through, Alphabet::Ascii).rows[2].prefix, "+-+-*");
    }

    /// A deterministic spread of graphs: forward edges, edges back up (so
    /// cycles), self edges, repeated edges and ids that are no node.
    fn spread() -> Vec<Vec<Node>> {
        let mut seed: u64 = 0x9f11_c7c0_6f72;
        let mut next = move |bound: u64| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % bound
        };
        let marks = [Mark::Ready, Mark::Waiting, Mark::Finished, Mark::InProgress];
        (0..300)
            .map(|_| {
                let n = 1 + next(14) as usize;
                (0..n)
                    .map(|i| {
                        let mut blocked_by = Vec::new();
                        for _ in 0..next(5) {
                            let pick = next(n as u64 + 2) as usize;
                            blocked_by.push(match pick < n {
                                true => format!("N{pick}"),
                                false => "ELSEWHERE".to_string(),
                            });
                        }
                        Node {
                            id: format!("N{i}"),
                            mark: marks[next(4) as usize],
                            blocked_by,
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn every_node_is_exactly_one_row_and_sits_above_all_it_reaches_outside_its_cycle() {
        for nodes in spread() {
            let laid = layout(&nodes, Alphabet::Boxes);
            let mut ids: Vec<&str> = laid.rows.iter().map(|r| r.id.as_str()).collect();
            ids.sort_unstable();
            let mut want: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
            want.sort_unstable();
            assert_eq!(ids, want, "{nodes:?}");
            let at = |id: &str| laid.rows.iter().position(|r| r.id == id);
            // Over transitive dependence and not direct edges only: a node
            // that reaches another through the blocked_by edges, by any
            // path and across any number of cycles, stands above it unless
            // that other reaches it back -- which is the two sharing a cycle.
            for n in &nodes {
                for m in &nodes {
                    if n.id == m.id || !reaches(&nodes, &n.id, &m.id) {
                        continue;
                    }
                    let (Some(top), Some(bottom)) = (at(&n.id), at(&m.id)) else {
                        continue;
                    };
                    if bottom < top {
                        assert!(
                            reaches(&nodes, &m.id, &n.id),
                            "{} depends on {} through no cycle and is drawn below it: {nodes:?}",
                            n.id,
                            m.id
                        );
                    }
                }
            }
        }
    }

    fn reaches(nodes: &[Node], from: &str, to: &str) -> bool {
        let mut seen = vec![from.to_string()];
        let mut stack = vec![from.to_string()];
        while let Some(id) = stack.pop() {
            for n in nodes.iter().filter(|n| n.id == id) {
                for b in &n.blocked_by {
                    if b == to {
                        return true;
                    }
                    if !seen.contains(b) {
                        seen.push(b.clone());
                        stack.push(b.clone());
                    }
                }
            }
        }
        false
    }

    #[test]
    fn edges_free_of_each_other_keep_the_input_order() {
        let nodes = [
            ready("C", &[]),
            ready("A", &[]),
            ready("D", &["E"]),
            ready("B", &[]),
            ready("E", &[]),
        ];
        let laid = layout(&nodes, Alphabet::Boxes);
        let ids: Vec<&str> = laid.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["C", "A", "D", "B", "E"]);
    }

    #[test]
    fn every_lane_between_a_node_and_its_blocker_is_continuous() {
        let vertical = [
            Alphabet::Boxes.glyph(Glyph::Pass),
            Alphabet::Boxes.glyph(Glyph::JoinLeft),
            Alphabet::Boxes.glyph(Glyph::JoinRight),
            Alphabet::Boxes.glyph(Glyph::JoinThrough),
        ];
        for nodes in spread() {
            let laid = layout(&nodes, Alphabet::Boxes);
            let at = |id: &str| laid.rows.iter().position(|r| r.id == id);
            for n in &nodes {
                let top = at(&n.id).unwrap();
                for b in &n.blocked_by {
                    let Some(bottom) = at(b) else { continue };
                    if bottom <= top {
                        continue;
                    }
                    let column = 2 * laid.rows[bottom].lane;
                    let cell = |r: usize| laid.rows[r].prefix.chars().nth(column).unwrap();
                    assert_ne!(cell(top), ' ', "{nodes:?}");
                    for r in top + 1..bottom {
                        assert!(
                            vertical.contains(&cell(r)),
                            "row {r} column {column} is {:?}: {nodes:?}",
                            cell(r)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn both_alphabets_draw_one_layout_at_one_width_differing_only_in_glyphs() {
        for nodes in spread() {
            let boxes = layout(&nodes, Alphabet::Boxes);
            let ascii = layout(&nodes, Alphabet::Ascii);
            assert_eq!(boxes.lanes, ascii.lanes);
            for (b, a) in boxes.rows.iter().zip(&ascii.rows) {
                assert_eq!((&b.id, b.lane, b.unknown), (&a.id, a.lane, a.unknown));
                assert_eq!(b.prefix.chars().count(), boxes.width());
                assert_eq!(a.prefix.chars().count(), boxes.width());
                for (x, y) in b.prefix.chars().zip(a.prefix.chars()) {
                    let paired = (x == ' ' && y == ' ')
                        || TABLE.iter().any(|&(_, bx, ax)| (bx, ax) == (x, y));
                    assert!(paired, "{x:?} against {y:?} in {nodes:?}");
                }
            }
        }
    }

    #[test]
    fn the_table_is_in_glyph_order_and_every_glyph_is_one_column() {
        for (i, &(glyph, boxes, ascii)) in TABLE.iter().enumerate() {
            assert_eq!(glyph as usize, i);
            assert!(ascii.is_ascii_graphic(), "{ascii:?}");
            // One column: the box-drawing block, and the three cells, all
            // narrow outside an East Asian ambiguous-width locale.
            assert!(
                ('\u{2500}'..='\u{257F}').contains(&boxes)
                    || ['\u{25CF}', '\u{25CB}', '\u{00B7}', '\u{2022}'].contains(&boxes),
                "{boxes:?}"
            );
        }
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Ready), '\u{25CF}');
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Waiting), '\u{25CB}');
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Finished), '\u{00B7}');
    }

    #[test]
    fn the_junction_layouts_glyphs_in_both_alphabets() {
        let both = |g: Glyph| (Alphabet::Boxes.glyph(g), Alphabet::Ascii.glyph(g));
        assert_eq!(both(Glyph::Open), ('○', 'o'));
        assert_eq!(both(Glyph::Settled), ('●', '@'));
        assert_eq!(both(Glyph::InProgress0), ('·', '.'));
        assert_eq!(both(Glyph::InProgress1), ('•', 'o'));
        assert_eq!(both(Glyph::InProgress2), ('●', 'O'));
        assert_eq!(both(Glyph::InProgress3), ('•', 'o'));
        assert_eq!(both(Glyph::Pass), ('│', '|'));
        assert_eq!(both(Glyph::Across), ('─', '-'));
        assert_eq!(both(Glyph::LeaveRight), ('┐', '+'));
        assert_eq!(both(Glyph::LeaveThrough), ('┬', '+'));
        assert_eq!(both(Glyph::JoinKeep), ('├', '|'));
        assert_eq!(both(Glyph::JoinFar), ('┘', '\''));
        assert_eq!(both(Glyph::JoinMiddle), ('┴', '+'));
        assert_eq!(both(Glyph::Cross), ('┼', '+'));
    }

    #[test]
    fn a_mark_draws_open_settled_or_the_pulse_phase_taken_modulo_four() {
        let cell = |m: Mark, phase: usize| m.cell(phase);
        assert_eq!(cell(Mark::Ready, 0), Glyph::Open);
        assert_eq!(cell(Mark::Waiting, 3), Glyph::Open);
        assert_eq!(cell(Mark::Finished, 1), Glyph::Settled);
        assert_eq!(cell(Mark::InProgress, 0), Glyph::InProgress0);
        assert_eq!(cell(Mark::InProgress, 1), Glyph::InProgress1);
        assert_eq!(cell(Mark::InProgress, 2), Glyph::InProgress2);
        assert_eq!(cell(Mark::InProgress, 3), Glyph::InProgress3);
        assert_eq!(cell(Mark::InProgress, 4), Glyph::InProgress0);
        assert_eq!(cell(Mark::InProgress, 9), Glyph::InProgress1);
    }

    // The junction layout (TASK-080f158e317e).

    /// The lines as (id, prefix), a junction row's id written "".
    fn lines(laid: &JunctionLayout) -> Vec<(&str, &str)> {
        laid.lines
            .iter()
            .map(|l| match l {
                Line::Task(row) => (row.id.as_str(), row.prefix.as_str()),
                Line::Junction(prefix) => ("", prefix.as_str()),
            })
            .collect()
    }

    fn junctions(nodes: &[Node]) -> JunctionLayout {
        junction_layout(nodes, Alphabet::Boxes, 0)
    }

    #[test]
    fn a_chain_of_three_is_three_task_rows_on_one_lane_and_no_junction() {
        let nodes = [ready("C", &[]), ready("B", &["C"]), ready("A", &["B"])];
        let laid = junctions(&nodes);
        assert_eq!(lines(&laid), [("A", "○"), ("B", "○"), ("C", "○")]);
        assert_eq!(laid.lanes, 1);
    }

    /// A blocked by B and C, both blocked by D, handed in top first.
    fn kite() -> [Node; 4] {
        [
            ready("A", &["B", "C"]),
            ready("B", &["D"]),
            ready("C", &["D"]),
            ready("D", &[]),
        ]
    }

    #[test]
    fn a_diamond_opens_a_lane_right_and_merges_it_on_a_junction_row_above_its_bottom() {
        let laid = junctions(&kite());
        assert_eq!(
            lines(&laid),
            [
                ("A", "○─┐"),
                ("B", "○ │"),
                ("C", "│ ○"),
                ("", "├─┘"),
                ("D", "○  "),
            ]
        );
        assert_eq!(laid.lanes, 2);
        let lanes: Vec<usize> = laid
            .lines
            .iter()
            .filter_map(|l| match l {
                Line::Task(row) => Some(row.lane),
                Line::Junction(_) => None,
            })
            .collect();
        assert_eq!(lanes, [0, 0, 1, 0]);
    }

    #[test]
    fn the_diamond_in_ascii_is_the_same_junction_layout_in_other_glyphs() {
        let laid = junction_layout(&kite(), Alphabet::Ascii, 0);
        assert_eq!(
            lines(&laid),
            [
                ("A", "o-+"),
                ("B", "o |"),
                ("C", "| o"),
                ("", "|-'"),
                ("D", "o  "),
            ]
        );
    }

    #[test]
    fn three_nodes_waiting_on_one_merge_three_lanes_on_one_junction_row() {
        let nodes = [
            ready("X", &["W"]),
            ready("Y", &["W"]),
            ready("Z", &["W"]),
            ready("W", &[]),
        ];
        let laid = junctions(&nodes);
        assert_eq!(
            lines(&laid),
            [
                ("X", "○    "),
                ("Y", "│ ○  "),
                ("Z", "│ │ ○"),
                ("", "├─┴─┘"),
                ("W", "○    "),
            ]
        );
        assert_eq!(laid.lanes, 3);
    }

    #[test]
    fn a_lane_opened_across_a_passing_lane_crosses_it_on_the_task_row() {
        // C, on lane 0, opens a lane for E to the right of B's lane, which
        // waits for D and passes under the horizontal.
        let nodes = [
            ready("A", &["C"]),
            ready("B", &["D"]),
            ready("C", &["D", "E"]),
            ready("D", &[]),
            ready("E", &[]),
        ];
        let laid = junctions(&nodes);
        assert_eq!(
            lines(&laid),
            [
                ("A", "○    "),
                ("B", "│ ○  "),
                ("C", "○─┼─┐"),
                ("", "├─┘ │"),
                ("D", "○   │"),
                ("E", "    ○"),
            ]
        );
    }

    #[test]
    fn a_lane_passing_inside_a_merge_crosses_it_on_the_junction_row() {
        let nodes = [
            ready("X", &["W"]),
            ready("Y", &["V"]),
            ready("Z", &["W"]),
            ready("W", &[]),
            ready("V", &[]),
        ];
        let laid = junctions(&nodes);
        assert_eq!(
            lines(&laid),
            [
                ("X", "○    "),
                ("Y", "│ ○  "),
                ("Z", "│ │ ○"),
                ("", "├─┼─┘"),
                ("W", "○ │  "),
                ("V", "  ○  "),
            ]
        );
        assert_eq!(
            junction_layout(&nodes, Alphabet::Ascii, 0).lines[3],
            Line::Junction("|-+-'".to_string())
        );
    }

    #[test]
    fn two_independent_roots_are_two_task_rows_on_one_lane() {
        let nodes = [ready("Y", &[]), ready("X", &[])];
        let laid = junctions(&nodes);
        assert_eq!(lines(&laid), [("Y", "○"), ("X", "○")]);
        assert_eq!(laid.lanes, 1);
    }

    #[test]
    fn a_node_blocked_by_seven_opens_six_lanes_to_its_right() {
        let nodes = [
            ready("X", &["B1", "B2", "B3", "B4", "B5", "B6", "B7"]),
            ready("B1", &[]),
            ready("B2", &[]),
            ready("B3", &[]),
            ready("B4", &[]),
            ready("B5", &[]),
            ready("B6", &[]),
            ready("B7", &[]),
        ];
        let laid = junctions(&nodes);
        assert_eq!(
            lines(&laid),
            [
                ("X", "○─┬─┬─┬─┬─┬─┐"),
                ("B1", "○ │ │ │ │ │ │"),
                ("B2", "  ○ │ │ │ │ │"),
                ("B3", "    ○ │ │ │ │"),
                ("B4", "      ○ │ │ │"),
                ("B5", "        ○ │ │"),
                ("B6", "          ○ │"),
                ("B7", "            ○"),
            ]
        );
        assert_eq!(laid.lanes, 7);
    }

    #[test]
    fn a_cycle_of_two_is_two_task_rows_and_the_edge_back_up_draws_nothing() {
        let nodes = [ready("A", &["B"]), ready("B", &["A"])];
        let laid = junctions(&nodes);
        assert_eq!(lines(&laid), [("A", "○"), ("B", "○")]);
        assert_eq!(laid.lanes, 1);
    }

    #[test]
    fn an_edge_outside_the_set_draws_no_lane_in_the_junction_layout_and_is_counted() {
        let nodes = [ready("A", &["GONE", "B", "LOST"]), ready("B", &[])];
        let laid = junctions(&nodes);
        assert_eq!(lines(&laid), [("A", "○"), ("B", "○")]);
        let unknown: Vec<usize> = laid
            .lines
            .iter()
            .filter_map(|l| match l {
                Line::Task(row) => Some(row.unknown),
                Line::Junction(_) => None,
            })
            .collect();
        assert_eq!(unknown, [2, 0]);
    }

    #[test]
    fn a_node_under_way_is_drawn_at_the_phase_handed_in_modulo_four() {
        let nodes = [
            node("R", Mark::InProgress, &["F"]),
            node("F", Mark::Finished, &[]),
        ];
        let at = |alphabet: Alphabet, phase: usize| -> Vec<String> {
            junction_layout(&nodes, alphabet, phase)
                .lines
                .iter()
                .map(|l| l.prefix().to_string())
                .collect()
        };
        assert_eq!(at(Alphabet::Boxes, 0), ["·", "●"]);
        assert_eq!(at(Alphabet::Boxes, 1), ["•", "●"]);
        assert_eq!(at(Alphabet::Boxes, 2), ["●", "●"]);
        assert_eq!(at(Alphabet::Boxes, 3), ["•", "●"]);
        assert_eq!(at(Alphabet::Boxes, 4), ["·", "●"]);
        assert_eq!(at(Alphabet::Ascii, 0), [".", "@"]);
        assert_eq!(at(Alphabet::Ascii, 1), ["o", "@"]);
        assert_eq!(at(Alphabet::Ascii, 2), ["O", "@"]);
        assert_eq!(at(Alphabet::Ascii, 3), ["o", "@"]);
        assert_eq!(at(Alphabet::Ascii, 4), [".", "@"]);
        let mark = match &junction_layout(&nodes, Alphabet::Boxes, 2).lines[0] {
            Line::Task(row) => row.mark,
            Line::Junction(_) => panic!("a junction first"),
        };
        assert_eq!(mark, Mark::InProgress);
    }

    #[test]
    fn ready_and_waiting_share_the_open_cell() {
        let nodes = [node("R", Mark::Ready, &[]), node("W", Mark::Waiting, &[])];
        let laid = junction_layout(&nodes, Alphabet::Boxes, 0);
        assert_eq!(lines(&laid), [("R", "○"), ("W", "○")]);
        let laid = junction_layout(&nodes, Alphabet::Ascii, 0);
        assert_eq!(lines(&laid), [("R", "o"), ("W", "o")]);
    }

    fn task_rows(laid: &JunctionLayout) -> Vec<&Row> {
        laid.lines
            .iter()
            .filter_map(|l| match l {
                Line::Task(row) => Some(row),
                Line::Junction(_) => None,
            })
            .collect()
    }

    #[test]
    fn the_junction_layout_draws_every_node_once_in_the_order_the_module_computes() {
        for nodes in spread() {
            let first: Vec<String> = layout(&nodes, Alphabet::Boxes)
                .rows
                .into_iter()
                .map(|r| r.id)
                .collect();
            let laid = junctions(&nodes);
            let rows: Vec<String> = task_rows(&laid).iter().map(|r| r.id.clone()).collect();
            assert_eq!(rows, first, "{nodes:?}");
        }
    }

    #[test]
    fn every_junction_prefix_is_the_layouts_width_and_no_trailing_lane_is_free() {
        for nodes in spread() {
            let laid = junctions(&nodes);
            let width = 2 * laid.lanes - 1;
            for line in &laid.lines {
                assert_eq!(line.prefix().chars().count(), width, "{nodes:?}");
            }
            // The last lane is used by some line, so no free lane trails.
            let last = laid
                .lines
                .iter()
                .any(|l| l.prefix().chars().nth(width - 1) != Some(' '));
            assert!(last, "{:#?}\n{nodes:?}", lines(&laid));
            // Each task row carries its cell in its lane's column.
            for row in task_rows(&laid) {
                let cell = row.prefix.chars().nth(2 * row.lane);
                assert_eq!(cell, Some(Alphabet::Boxes.glyph(row.mark.cell(0))));
            }
        }
    }

    #[test]
    fn both_alphabets_draw_one_junction_layout_differing_only_in_glyphs() {
        for nodes in spread() {
            for phase in 0..4 {
                let boxes = junction_layout(&nodes, Alphabet::Boxes, phase);
                let ascii = junction_layout(&nodes, Alphabet::Ascii, phase);
                assert_eq!(boxes.lanes, ascii.lanes);
                assert_eq!(boxes.lines.len(), ascii.lines.len());
                for (b, a) in boxes.lines.iter().zip(&ascii.lines) {
                    if let (Line::Task(b), Line::Task(a)) = (b, a) {
                        assert_eq!((&b.id, b.lane, b.unknown), (&a.id, a.lane, a.unknown));
                    }
                    assert_eq!(b.prefix().chars().count(), a.prefix().chars().count());
                    for (x, y) in b.prefix().chars().zip(a.prefix().chars()) {
                        let paired = (x == ' ' && y == ' ')
                            || TABLE.iter().any(|&(_, bx, ax)| (bx, ax) == (x, y));
                        assert!(paired, "{x:?} against {y:?} in {nodes:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_junction_row_stands_only_immediately_above_a_node_two_lanes_or_more_wait_for() {
        let keep = Alphabet::Boxes.glyph(Glyph::JoinKeep);
        let far = Alphabet::Boxes.glyph(Glyph::JoinFar);
        for nodes in spread() {
            let laid = junctions(&nodes);
            for (i, line) in laid.lines.iter().enumerate() {
                let Line::Junction(prefix) = line else {
                    continue;
                };
                let Some(Line::Task(under)) = laid.lines.get(i + 1) else {
                    panic!("a junction row with no task row under it: {nodes:?}");
                };
                let cells: Vec<char> = prefix.chars().collect();
                assert_eq!(cells[2 * under.lane], keep, "{prefix:?} {nodes:?}");
                assert_eq!(cells.iter().filter(|&&c| c == keep).count(), 1);
                assert_eq!(cells.iter().filter(|&&c| c == far).count(), 1);
                let far_at = cells.iter().position(|&c| c == far).unwrap();
                assert!(far_at > 2 * under.lane, "{prefix:?} {nodes:?}");
            }
        }
    }

    #[test]
    fn every_lane_between_a_node_and_its_blocker_is_continuous_through_junction_rows() {
        let glyph = |g: Glyph| Alphabet::Boxes.glyph(g);
        let lane_glyphs = [
            Glyph::Pass,
            Glyph::Cross,
            Glyph::JoinKeep,
            Glyph::JoinFar,
            Glyph::JoinMiddle,
        ]
        .map(glyph);
        let leaves = [Glyph::LeaveRight, Glyph::LeaveThrough].map(glyph);
        let mut checked = 0;
        for nodes in spread() {
            let laid = junctions(&nodes);
            let at = |id: &str| {
                laid.lines
                    .iter()
                    .position(|l| matches!(l, Line::Task(r) if r.id == id))
            };
            let cell =
                |line: usize, column: usize| laid.lines[line].prefix().chars().nth(column).unwrap();
            for n in &nodes {
                let top = at(&n.id).unwrap();
                let Line::Task(row) = &laid.lines[top] else {
                    unreachable!()
                };
                // The edges drawn: to nodes below, each id once, in order.
                let mut below: Vec<&str> = Vec::new();
                for b in &n.blocked_by {
                    if at(b).is_some_and(|r| r > top) && !below.contains(&b.as_str()) {
                        below.push(b);
                    }
                }
                // The first continues the node's lane, the others leave for
                // the lanes opened, left to right.
                let mut columns = vec![2 * row.lane];
                columns.extend(
                    row.prefix
                        .chars()
                        .enumerate()
                        .filter(|&(c, g)| c > 2 * row.lane && leaves.contains(&g))
                        .map(|(c, _)| c),
                );
                assert_eq!(columns.len(), below.len().max(1), "{nodes:?}");
                for (b, &column) in below.iter().zip(&columns) {
                    let bottom = at(b).unwrap();
                    for line in top + 1..bottom {
                        assert!(
                            lane_glyphs.contains(&cell(line, column)),
                            "{} to {b}: line {line} column {column} is {:?}\n{:#?}\n{nodes:?}",
                            n.id,
                            cell(line, column),
                            lines(&laid)
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 1000, "only {checked} cells checked");
    }

    #[test]
    fn groups_are_the_linked_sets_in_the_order_of_their_first_member() {
        // Two linked pairs, B-D linked through an edge pointing up the input
        // and A-C through one pointing down, a lone node between them, and
        // edges to ids outside the set, which link nothing.
        let nodes = [
            ready("A", &["C"]),
            ready("B", &[]),
            ready("L", &["GONE"]),
            ready("C", &["ELSEWHERE"]),
            ready("D", &["B"]),
        ];
        let named: Vec<Vec<&str>> = groups(&nodes)
            .into_iter()
            .map(|g| g.into_iter().map(|i| nodes[i].id.as_str()).collect())
            .collect();
        assert_eq!(named, [vec!["A", "C"], vec!["B", "D"], vec!["L"]]);
    }

    #[test]
    fn a_group_reaches_across_shared_blockers_and_cycles() {
        // X and Y share a blocker; P and Q are a cycle; S edges only itself.
        let nodes = [
            ready("P", &["Q"]),
            ready("X", &["Z"]),
            ready("S", &["S"]),
            ready("Y", &["Z"]),
            ready("Q", &["P"]),
            ready("Z", &[]),
        ];
        assert_eq!(groups(&nodes), [vec![0, 4], vec![1, 3, 5], vec![2]]);
        assert_eq!(groups(&[]), Vec::<Vec<usize>>::new());
    }

    #[test]
    fn the_module_names_no_terminal_no_ank_and_no_app() {
        let source = include_str!("graph.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        for name in ["ratatui", "crossterm", "crate::", "App", "Ank"] {
            assert!(!code.contains(name), "graph.rs names {name}");
        }
    }
}
