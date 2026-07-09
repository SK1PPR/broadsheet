//! Reusable data-structure widgets: one call declares a whole structure
//! (nodes, edges, labels, indices) with conventional entity ids, and returns
//! a handle that hands those ids back for animation.
//!
//! ```ignore
//! let arr = widgets::array(&mut m, "arr", &["7", "3", "9"], v(640., 300.));
//! m.play(act().highlight(&arr.cell(1), m.role(Role::Active)));
//! ```
//!
//! Widgets are sugar over [`crate::scene::SceneBuilder`] — everything they
//! declare is a plain entity you can restyle or animate directly. Missing
//! structure (Bloom filter, trie, B-tree page) composes from these: a Bloom
//! filter is a [`bit_array`], a trie is a [`tree`].

use macroquad::prelude::Vec2;

use crate::layout;
use crate::movie::Movie;

// ---- arrays & tables ------------------------------------------------------

/// Handle for [`array`], [`bit_array`], and [`hash_table`]. Ids follow the
/// [`crate::scene::SceneBuilder::cells`] convention.
pub struct ArrayView {
    pub id: String,
    pub n: usize,
}

impl ArrayView {
    /// The cell rect: `"{id}{i}"`.
    pub fn cell(&self, i: usize) -> String {
        format!("{}{}", self.id, i)
    }

    /// The text riding on cell `i`: `"{id}{i}.label"`.
    pub fn label(&self, i: usize) -> String {
        format!("{}{}.label", self.id, i)
    }

    /// The faded index digit under cell `i`: `"{id}{i}.idx"`.
    pub fn index(&self, i: usize) -> String {
        format!("{}{}.idx", self.id, i)
    }
}

/// A labeled array/list/queue: one 64×64 cell per entry, centred on
/// `center`, indices underneath. All cells carry tag `id`.
pub fn array(m: &mut Movie, id: &str, labels: &[&str], center: Vec2) -> ArrayView {
    m.scene().cells(
        id,
        labels.len(),
        center,
        Vec2::new(64.0, 64.0),
        10.0,
        Some(labels),
    );
    ArrayView {
        id: id.into(),
        n: labels.len(),
    }
}

/// `n` compact bit cells, all showing `"0"`. Flip one with
/// `act().set_text(&bits.label(i), "1")` plus a role highlight.
pub fn bit_array(m: &mut Movie, id: &str, n: usize, center: Vec2) -> ArrayView {
    let zeros = vec!["0"; n];
    m.scene()
        .cells(id, n, center, Vec2::new(48.0, 48.0), 6.0, Some(&zeros));
    ArrayView { id: id.into(), n }
}

/// `n` empty buckets with indices — a hash table before any inserts. Fill a
/// bucket with `act().set_text(&table.label(i), "k7")`.
pub fn hash_table(m: &mut Movie, id: &str, n: usize, center: Vec2) -> ArrayView {
    m.scene()
        .cells(id, n, center, Vec2::new(64.0, 64.0), 6.0, None);
    ArrayView { id: id.into(), n }
}

// ---- linked list ------------------------------------------------------------

/// Handle for [`linked_list`].
pub struct LinkedListView {
    pub id: String,
    pub n: usize,
    /// Position of node `i` (for retargeting pointers).
    pub pos: Vec<Vec2>,
}

impl LinkedListView {
    /// The node rect: `"{id}.n{i}"`.
    pub fn node(&self, i: usize) -> String {
        format!("{}.n{}", self.id, i)
    }

    /// The node's value text: `"{id}.n{i}.label"`.
    pub fn label(&self, i: usize) -> String {
        format!("{}.n{}.label", self.id, i)
    }

    /// The pointer arrow from node `i` to node `i + 1`: `"{id}.a{i}"`.
    /// Retarget it with `act().retarget(&list.next(i), list.pos[j])`.
    pub fn next(&self, i: usize) -> String {
        format!("{}.a{}", self.id, i)
    }
}

/// A singly-linked list: value boxes left to right from `start` (centre of
/// node 0), pointer arrows between consecutive nodes. Nodes carry tag `id`.
pub fn linked_list(m: &mut Movie, id: &str, labels: &[&str], start: Vec2) -> LinkedListView {
    let (w, h, gap) = (72.0, 48.0, 48.0);
    let stride = w + gap;
    let pos: Vec<Vec2> = (0..labels.len())
        .map(|i| Vec2::new(start.x + stride * i as f32, start.y))
        .collect();
    {
        let mut s = m.scene();
        for (i, label) in labels.iter().enumerate() {
            s.rect(&format!("{id}.n{i}"), pos[i], w, h)
                .tag(id)
                .label(label);
        }
        for i in 0..labels.len().saturating_sub(1) {
            s.arrow(
                &format!("{id}.a{i}"),
                pos[i] + Vec2::new(w / 2.0, 0.0),
                pos[i + 1] - Vec2::new(w / 2.0, 0.0),
            )
            .tag(id);
        }
    }
    LinkedListView {
        id: id.into(),
        n: labels.len(),
        pos,
    }
}

// ---- tree -------------------------------------------------------------------

/// Handle for [`tree`].
pub struct TreeView {
    pub id: String,
    pub n: usize,
    /// Position of node `i`.
    pub pos: Vec<Vec2>,
    parents: Vec<Option<usize>>,
}

impl TreeView {
    /// The node circle: `"{id}.n{i}"`.
    pub fn node(&self, i: usize) -> String {
        format!("{}.n{}", self.id, i)
    }

    /// The node's value text: `"{id}.n{i}.label"`.
    pub fn label(&self, i: usize) -> String {
        format!("{}.n{}.label", self.id, i)
    }

    /// The edge line from node `i` up to its parent (`None` for roots):
    /// `"{id}.e{i}"`. Its animatable endpoint (`grow_to`/`retarget`) is at
    /// the parent, so re-parenting node `i` is one retarget.
    pub fn edge(&self, i: usize) -> Option<String> {
        self.parents[i].map(|_| format!("{}.e{}", self.id, i))
    }
}

/// A tree/forest from parent links (`parents[i] = None` for roots), laid out
/// with [`layout::tree`]. Edges draw beneath the node circles; nodes carry
/// tag `id`. Works for binary trees, tries, and union-find forests alike.
pub fn tree(
    m: &mut Movie,
    id: &str,
    parents: &[Option<usize>],
    labels: &[&str],
    top: Vec2,
    dx: f32,
    dy: f32,
) -> TreeView {
    assert_eq!(parents.len(), labels.len(), "one label per node");
    let pos = layout::tree(parents, top, dx, dy);
    {
        let mut s = m.scene();
        for (i, p) in parents.iter().enumerate() {
            if let Some(p) = p {
                s.line(&format!("{id}.e{i}"), pos[i], pos[*p]).z(-1).tag(id);
            }
        }
        for (i, label) in labels.iter().enumerate() {
            s.circle(&format!("{id}.n{i}"), pos[i], 26.0)
                .tag(id)
                .label(label);
        }
    }
    TreeView {
        id: id.into(),
        n: parents.len(),
        pos,
        parents: parents.to_vec(),
    }
}

// ---- graph --------------------------------------------------------------------

/// Handle for [`graph`].
pub struct GraphView {
    pub id: String,
    pub n: usize,
    /// Position of node `i`.
    pub pos: Vec<Vec2>,
}

impl GraphView {
    /// The node circle: `"{id}.n{i}"`.
    pub fn node(&self, i: usize) -> String {
        format!("{}.n{}", self.id, i)
    }

    /// The node's value text: `"{id}.n{i}.label"`.
    pub fn label(&self, i: usize) -> String {
        format!("{}.n{}.label", self.id, i)
    }

    /// The edge line between nodes `a` and `b` (declaration order):
    /// `"{id}.e{a}-{b}"`.
    pub fn edge(&self, a: usize, b: usize) -> String {
        format!("{}.e{}-{}", self.id, a, b)
    }
}

/// A node-and-edge graph. Pass explicit `positions`, or `None` to lay it out
/// with the deterministic [`layout::graph`] spring layout inside the canvas
/// centre. Edges draw beneath nodes; nodes carry tag `id`.
pub fn graph(
    m: &mut Movie,
    id: &str,
    labels: &[&str],
    edges: &[(usize, usize)],
    positions: Option<&[Vec2]>,
) -> GraphView {
    let n = labels.len();
    let pos: Vec<Vec2> = match positions {
        Some(p) => {
            assert_eq!(p.len(), n, "one position per node");
            p.to_vec()
        }
        None => {
            let center = Vec2::new(m.width as f32 / 2.0, m.height as f32 / 2.0 + 30.0);
            let radius = (m.height as f32) * 0.30;
            layout::graph(n, edges, center, radius, 1)
        }
    };
    {
        let mut s = m.scene();
        for (a, b) in edges {
            s.line(&format!("{id}.e{a}-{b}"), pos[*a], pos[*b])
                .z(-1)
                .tag(id);
        }
        for (i, label) in labels.iter().enumerate() {
            s.circle(&format!("{id}.n{i}"), pos[i], 26.0)
                .tag(id)
                .label(label);
        }
    }
    GraphView {
        id: id.into(),
        n,
        pos,
    }
}

// ---- hash ring ------------------------------------------------------------------

/// Handle for [`hash_ring`].
pub struct HashRingView {
    pub id: String,
    pub n: usize,
    pub center: Vec2,
    pub r: f32,
    fractions: Vec<f32>,
}

impl HashRingView {
    /// The ring outline circle: `"{id}.ring"`.
    pub fn ring(&self) -> String {
        format!("{}.ring", self.id)
    }

    /// Node `i`'s circle: `"{id}.n{i}"`.
    pub fn node(&self, i: usize) -> String {
        format!("{}.n{}", self.id, i)
    }

    /// Node `i`'s label: `"{id}.n{i}.label"`.
    pub fn label(&self, i: usize) -> String {
        format!("{}.n{}.label", self.id, i)
    }

    /// Point on the ring at `frac` of a turn clockwise from 12 o'clock —
    /// where a key with that hash lands. Use to move a key dot to the ring.
    pub fn at(&self, frac: f32) -> Vec2 {
        ring_point(self.center, self.r, frac)
    }

    /// Node `i`'s position on the ring.
    pub fn node_pos(&self, i: usize) -> Vec2 {
        self.at(self.fractions[i])
    }
}

fn ring_point(center: Vec2, r: f32, frac: f32) -> Vec2 {
    let a = std::f32::consts::TAU * frac - std::f32::consts::FRAC_PI_2;
    center + Vec2::new(a.cos(), a.sin()) * r
}

/// A consistent-hash ring: a faded circle outline plus one labeled node per
/// `(fraction-of-turn, label)` entry, clockwise from 12 o'clock. Nodes carry
/// tag `id`.
pub fn hash_ring(
    m: &mut Movie,
    id: &str,
    center: Vec2,
    r: f32,
    nodes: &[(f32, &str)],
) -> HashRingView {
    let faded = m.theme.faded;
    {
        let mut s = m.scene();
        s.circle(&format!("{id}.ring"), center, r)
            .outlined()
            .outline_color(faded)
            .stroke(2.0)
            .tag(id);
        for (i, (frac, label)) in nodes.iter().enumerate() {
            s.circle(&format!("{id}.n{i}"), ring_point(center, r, *frac), 20.0)
                .tag(id)
                .label(label);
        }
    }
    HashRingView {
        id: id.into(),
        n: nodes.len(),
        center,
        r,
        fractions: nodes.iter().map(|(f, _)| *f).collect(),
    }
}

// ---- LSM levels --------------------------------------------------------------------

/// Handle for [`lsm_levels`].
pub struct LsmLevelsView {
    pub id: String,
    /// Runs per level.
    pub counts: Vec<usize>,
    /// `pos[l][i]` = centre of run `i` on level `l`.
    pub pos: Vec<Vec<Vec2>>,
}

impl LsmLevelsView {
    /// Run block `i` on level `l`: `"{id}.l{l}b{i}"`.
    pub fn block(&self, l: usize, i: usize) -> String {
        format!("{}.l{}b{}", self.id, l, i)
    }

    /// The run block's text: `"{id}.l{l}b{i}.label"`.
    pub fn label(&self, l: usize, i: usize) -> String {
        format!("{}.l{}b{}.label", self.id, l, i)
    }

    /// The faded `L{l}` tag at the left of level `l`: `"{id}.l{l}.tag"`.
    pub fn level_tag(&self, l: usize) -> String {
        format!("{}.l{}.tag", self.id, l)
    }
}

/// LSM-tree levels (or any level hierarchy): `counts[l]` run blocks per
/// level, centred rows widening downward from `top`, with a faded `L{l}`
/// tag on the left. Blocks carry tag `id`.
pub fn lsm_levels(m: &mut Movie, id: &str, counts: &[usize], top: Vec2) -> LsmLevelsView {
    let (bw, bh) = (64.0, 38.0);
    let (dx, dy) = (bw + 10.0, bh + 26.0);
    let pos = layout::levels(counts, top, dx, dy);
    let faded = m.theme.faded;
    let widest = counts.iter().copied().max().unwrap_or(0) as f32;
    let tag_x = top.x - dx * (widest - 1.0) / 2.0 - bw / 2.0 - 40.0;
    {
        let mut s = m.scene();
        for (l, row) in pos.iter().enumerate() {
            s.text(
                &format!("{id}.l{l}.tag"),
                Vec2::new(tag_x, top.y + l as f32 * dy),
                &format!("L{l}"),
            )
            .size(18.0)
            .color(faded)
            .tag(id);
            for (i, p) in row.iter().enumerate() {
                s.rect(&format!("{id}.l{l}b{i}"), *p, bw, bh)
                    .tag(id)
                    .label("");
            }
        }
    }
    LsmLevelsView {
        id: id.into(),
        counts: counts.to_vec(),
        pos,
    }
}

// ---- skip list ----------------------------------------------------------------------

/// Handle for [`skip_list`].
pub struct SkipListView {
    pub id: String,
    /// Number of express lanes (excluding the base lane).
    pub lanes: usize,
    /// `pos[k]` = centre of column `c`'s node on lane `k` (lane 0 = base).
    base: Vec2,
    dx: f32,
    dy: f32,
}

impl SkipListView {
    /// Node at column `c` on lane `k` (lane 0 = base row): `"{id}.l{k}n{c}"`.
    pub fn node(&self, k: usize, c: usize) -> String {
        format!("{}.l{}n{}", self.id, k, c)
    }

    /// The node's value text: `"{id}.l{k}n{c}.label"`.
    pub fn label(&self, k: usize, c: usize) -> String {
        format!("{}.l{}n{}.label", self.id, k, c)
    }

    /// The lane-`k` arrow leaving column `c` for the next present column:
    /// `"{id}.l{k}a{c}"`.
    pub fn next(&self, k: usize, c: usize) -> String {
        format!("{}.l{}a{}", self.id, k, c)
    }

    /// Centre of the (possibly absent) slot at column `c`, lane `k` — handy
    /// for tracing a search path with a moving dot.
    pub fn slot(&self, k: usize, c: usize) -> Vec2 {
        self.base + Vec2::new(self.dx * c as f32, -self.dy * k as f32)
    }
}

/// A skip list: `values` fill the base lane (lane 0, at `base`); each entry
/// of `lanes` lists the column indices promoted to that express lane, one
/// lane per entry stacked upward. Arrows link consecutive nodes within a
/// lane. Nodes carry tag `id`.
pub fn skip_list(
    m: &mut Movie,
    id: &str,
    values: &[&str],
    lanes: &[Vec<usize>],
    base: Vec2,
) -> SkipListView {
    let (w, h) = (56.0, 40.0);
    let (dx, dy) = (w + 26.0, h + 30.0);
    let all: Vec<usize> = (0..values.len()).collect();
    {
        let mut s = m.scene();
        for (k, cols) in std::iter::once(&all).chain(lanes.iter()).enumerate() {
            let y = base.y - dy * k as f32;
            for (j, &c) in cols.iter().enumerate() {
                assert!(
                    c < values.len(),
                    "lane {k} references column {c} out of range"
                );
                let p = Vec2::new(base.x + dx * c as f32, y);
                s.rect(&format!("{id}.l{k}n{c}"), p, w, h)
                    .tag(id)
                    .label(values[c]);
                if let Some(&next_c) = cols.get(j + 1) {
                    s.arrow(
                        &format!("{id}.l{k}a{c}"),
                        p + Vec2::new(w / 2.0, 0.0),
                        Vec2::new(base.x + dx * next_c as f32 - w / 2.0, y),
                    )
                    .tag(id);
                }
            }
        }
    }
    SkipListView {
        id: id.into(),
        lanes: lanes.len(),
        base,
        dx,
        dy,
    }
}
