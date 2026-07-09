//! Position helpers: compute `Vec2`s, feed them to the scene builder.
//! Pure functions — no engine state involved.

use macroquad::prelude::Vec2;

/// `n` positions evenly spaced along the horizontal line `y`, spanning
/// `[x0, x1]` inclusive.
pub fn row(n: usize, y: f32, x0: f32, x1: f32) -> Vec<Vec2> {
    if n == 1 {
        return vec![Vec2::new((x0 + x1) / 2.0, y)];
    }
    (0..n)
        .map(|i| Vec2::new(x0 + (x1 - x0) * i as f32 / (n - 1) as f32, y))
        .collect()
}

/// `cols × rows` cell centres filling the rectangle `min..max`, row-major.
pub fn grid(cols: usize, rows: usize, min: Vec2, max: Vec2) -> Vec<Vec2> {
    let cw = (max.x - min.x) / cols as f32;
    let ch = (max.y - min.y) / rows as f32;
    (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                Vec2::new(min.x + cw * (c as f32 + 0.5), min.y + ch * (r as f32 + 0.5))
            })
        })
        .collect()
}

/// `n` positions on a circle, clockwise from 12 o'clock. The natural layout
/// for consistent-hash rings and circular buffers.
pub fn ring(n: usize, center: Vec2, r: f32) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
            center + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect()
}

/// Tree layout from parent links (`parents[i] = None` for roots).
/// Leaves get consecutive horizontal slots `dx` apart; internal nodes centre
/// over their children; depth `d` sits at `top.y + d * dy`. The whole forest
/// is centred on `top.x`.
pub fn tree(parents: &[Option<usize>], top: Vec2, dx: f32, dy: f32) -> Vec<Vec2> {
    let n = parents.len();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut roots = Vec::new();
    for (i, p) in parents.iter().enumerate() {
        match p {
            Some(p) => children[*p].push(i),
            None => roots.push(i),
        }
    }

    let mut x = vec![0.0f32; n];
    let mut depth = vec![0usize; n];
    let mut next_slot = 0.0f32;

    fn place(
        i: usize,
        d: usize,
        children: &[Vec<usize>],
        x: &mut [f32],
        depth: &mut [usize],
        next_slot: &mut f32,
    ) {
        depth[i] = d;
        if children[i].is_empty() {
            x[i] = *next_slot;
            *next_slot += 1.0;
            return;
        }
        for &c in &children[i] {
            place(c, d + 1, children, x, depth, next_slot);
        }
        let sum: f32 = children[i].iter().map(|&c| x[c]).sum();
        x[i] = sum / children[i].len() as f32;
    }

    for &r in &roots {
        place(r, 0, &children, &mut x, &mut depth, &mut next_slot);
    }

    let mid = (next_slot - 1.0).max(0.0) / 2.0;
    (0..n)
        .map(|i| Vec2::new(top.x + (x[i] - mid) * dx, top.y + depth[i] as f32 * dy))
        .collect()
}

/// One row of positions per level: `counts[l]` slots, horizontally centred
/// on `top.x`, level `l` at `top.y + l * dy`, slots `dx` apart. The natural
/// layout for skip lists, LSM levels and memory hierarchies.
pub fn levels(counts: &[usize], top: Vec2, dx: f32, dy: f32) -> Vec<Vec<Vec2>> {
    counts
        .iter()
        .enumerate()
        .map(|(l, &n)| {
            let y = top.y + l as f32 * dy;
            let x0 = top.x - dx * (n as f32 - 1.0) / 2.0;
            (0..n).map(|i| Vec2::new(x0 + dx * i as f32, y)).collect()
        })
        .collect()
}

/// `n` positions flowing row-major, `cols` per row, starting at `origin`
/// (the centre of slot 0) with `dx`/`dy` strides. Database pages, SSTable
/// blocks, column chunks.
pub fn blocks(n: usize, cols: usize, origin: Vec2, dx: f32, dy: f32) -> Vec<Vec2> {
    let cols = cols.max(1);
    (0..n)
        .map(|i| {
            Vec2::new(
                origin.x + (i % cols) as f32 * dx,
                origin.y + (i / cols) as f32 * dy,
            )
        })
        .collect()
}

/// Tiny deterministic PRNG (splitmix64): same seed, same sequence, on every
/// machine — jitter and scatter stay repeatable across renders. Returns a
/// closure yielding `f32` in `[0, 1)`.
pub fn rng(seed: u64) -> impl FnMut() -> f32 {
    let mut state = seed;
    move || {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        (z >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Deterministic force-directed layout for `n` nodes with undirected
/// `edges`. Nodes start on a seeded ring and relax through a fixed number of
/// spring iterations, then the result is rescaled to fit within `radius` of
/// `center`. Same inputs, same layout.
pub fn graph(
    n: usize,
    edges: &[(usize, usize)],
    center: Vec2,
    radius: f32,
    seed: u64,
) -> Vec<Vec2> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![center];
    }
    let mut rand = rng(seed);
    // seeded ring start breaks symmetry without losing determinism
    let mut pos: Vec<Vec2> = ring(n, Vec2::ZERO, radius)
        .into_iter()
        .map(|p| p + Vec2::new(rand() - 0.5, rand() - 0.5) * radius * 0.3)
        .collect();

    let k = radius / (n as f32).sqrt(); // ideal edge length
    let iters = 150;
    for it in 0..iters {
        let temp = radius * 0.1 * (1.0 - it as f32 / iters as f32);
        let mut disp = vec![Vec2::ZERO; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let d = pos[i] - pos[j];
                let dist = d.length().max(1e-3);
                let push = d / dist * (k * k / dist);
                disp[i] += push;
                disp[j] -= push;
            }
        }
        for &(a, b) in edges {
            let d = pos[a] - pos[b];
            let dist = d.length().max(1e-3);
            let pull = d / dist * (dist * dist / k);
            disp[a] -= pull;
            disp[b] += pull;
        }
        for i in 0..n {
            let len = disp[i].length().max(1e-3);
            pos[i] += disp[i] / len * len.min(temp);
        }
    }

    // recentre and rescale into the requested disc
    let centroid = pos.iter().copied().fold(Vec2::ZERO, |a, b| a + b) / n as f32;
    let max_r = pos
        .iter()
        .map(|p| (*p - centroid).length())
        .fold(1e-3, f32::max);
    pos.iter()
        .map(|p| center + (*p - centroid) * (radius / max_r))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_spans_inclusive() {
        let p = row(3, 100.0, 0.0, 200.0);
        assert_eq!(p[0].x, 0.0);
        assert_eq!(p[1].x, 100.0);
        assert_eq!(p[2].x, 200.0);
    }

    #[test]
    fn levels_centres_each_row() {
        let l = levels(&[1, 3], Vec2::new(100.0, 0.0), 50.0, 80.0);
        assert_eq!(l[0][0], Vec2::new(100.0, 0.0));
        assert_eq!(l[1][0].x, 50.0);
        assert_eq!(l[1][2].x, 150.0);
        assert_eq!(l[1][0].y, 80.0);
    }

    #[test]
    fn blocks_flow_row_major() {
        let b = blocks(5, 3, Vec2::new(0.0, 0.0), 10.0, 20.0);
        assert_eq!(b[2], Vec2::new(20.0, 0.0));
        assert_eq!(b[3], Vec2::new(0.0, 20.0));
        assert_eq!(b[4], Vec2::new(10.0, 20.0));
    }

    #[test]
    fn rng_is_deterministic_and_unit_range() {
        let mut a = rng(42);
        let mut b = rng(42);
        for _ in 0..100 {
            let x = a();
            assert_eq!(x, b());
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn graph_layout_is_deterministic_and_bounded() {
        let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)];
        let a = graph(4, &edges, Vec2::new(500.0, 400.0), 200.0, 7);
        let b = graph(4, &edges, Vec2::new(500.0, 400.0), 200.0, 7);
        assert_eq!(a, b);
        for p in &a {
            assert!((*p - Vec2::new(500.0, 400.0)).length() <= 200.0 + 1e-3);
        }
    }

    #[test]
    fn tree_centres_parent_over_children() {
        // 2 <- {0, 1}
        let p = tree(&[Some(2), Some(2), None], Vec2::new(0.0, 0.0), 100.0, 80.0);
        assert_eq!(p[2].x, (p[0].x + p[1].x) / 2.0);
        assert_eq!(p[2].y, 0.0);
        assert_eq!(p[0].y, 80.0);
    }
}
