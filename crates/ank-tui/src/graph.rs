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
    /// What is drawn left of the title, [`Layout::width`] columns, one
    /// `char` per column.
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
}

/// **The one table**: each glyph, then its box-drawing and its ASCII
/// character, in [`Glyph`]'s order. Every character is one column wide.
pub const TABLE: [(Glyph, char, char); 11] = [
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
    fn glyph(self) -> Glyph {
        match self {
            Mark::Ready => Glyph::Ready,
            Mark::Waiting => Glyph::Waiting,
            Mark::Finished => Glyph::Finished,
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

    let mut lanes: Vec<Option<usize>> = Vec::new();
    let mut drawn = vec![false; nodes.len()];
    let mut placed: Vec<(usize, usize, Vec<Option<Glyph>>)> = Vec::with_capacity(nodes.len());
    let mut most = 0;
    for x in order(&blockers) {
        drawn[x] = true;
        let lane = match lanes.iter().position(|w| *w == Some(x)) {
            Some(l) => l,
            None => free(&mut lanes),
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
                let t = free(&mut lanes);
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

/// The first free lane, made at the right edge when there is none.
fn free(lanes: &mut Vec<Option<usize>>) -> usize {
    match lanes.iter().position(Option::is_none) {
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
        let marks = [Mark::Ready, Mark::Waiting, Mark::Finished];
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
                            mark: marks[next(3) as usize],
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
                    || ['\u{25CF}', '\u{25CB}', '\u{00B7}'].contains(&boxes),
                "{boxes:?}"
            );
        }
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Ready), '\u{25CF}');
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Waiting), '\u{25CB}');
        assert_eq!(Alphabet::Boxes.glyph(Glyph::Finished), '\u{00B7}');
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
