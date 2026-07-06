//! Union-Find / Disjoint Set: unions build a tree, then find(0) walks to the
//! root and path compression flattens the walk.
//!
//! Run live:    cargo run --example union_find
//! Record:      cargo run --example union_find -- --record out/uf --fps 60

use broadsheet::prelude::*;

const N: usize = 6;
const ROW_Y: f32 = 580.0;

fn row_pos(i: usize) -> Vec2 {
    v(215.0 + 170.0 * i as f32, ROW_Y)
}

fn node(i: usize) -> String {
    format!("n{i}")
}

/// Parent-pointer arrow of node `i` (exists only for nodes that get parents).
fn parrow(i: usize) -> String {
    format!("p{i}")
}

/// One union step: name it in the caption, flash the two roots, grow the
/// child's parent arrow into the parent.
fn union(m: &mut Movie, child: usize, parent: usize, caption: &str) {
    m.play(act().set_text("caption", caption));
    m.play(par![
        act().highlight(&node(child), ACCENT).dur(0.6),
        act().highlight(&node(parent), ACCENT).dur(0.6),
    ]);
    m.play(par![
        act().fade_in(&parrow(child)).dur(0.15),
        act().grow_to(&parrow(child), row_pos(parent)).dur(0.5).ease(InOutCubic),
    ]);
    m.wait(0.35);
}

fn main() {
    let mut m = Movie::new("Union-Find", 1280, 720);

    // final forest layout (after the re-layout step)
    let tree: [(usize, Vec2); 6] = [
        (0, v(250.0, 630.0)),
        (1, v(340.0, 520.0)),
        (2, v(580.0, 520.0)),
        (3, v(460.0, 390.0)),
        (4, v(820.0, 390.0)),
        (5, v(640.0, 240.0)),
    ];
    let tree_pos = |i: usize| tree[i].1;

    // ---- cast ----------------------------------------------------------
    {
        let mut s = m.scene();
        // parent arrows first (z below nodes so heads tuck under circles)
        for i in [0usize, 1, 2, 3, 4] {
            s.arrow(&parrow(i), row_pos(i), row_pos(i))
                .color(INK)
                .stroke(2.5)
                .z(1)
                .hidden()
                .follow(&node(i), v(0.0, 0.0));
        }
        for i in 0..N {
            s.circle(&node(i), row_pos(i), 34.0)
                .stroke(3.0)
                .z(5)
                .hidden()
                .label(&i.to_string());
        }
        s.text("caption", v(640.0, 140.0), "").size(22.0).color(FADED).wrap(1080.0).hidden();
    }

    // ---- intro: six disjoint sets ---------------------------------------
    let mut cascade = Vec::new();
    for i in 0..N {
        cascade.push(Clip::from(act().fade_in(&node(i)).dur(0.3)).shift(0.08 * i as f32));
    }
    m.play(Clip::par(cascade));
    m.play(act().set_text("caption", "6 elements, 6 disjoint sets — every node is its own root"));
    m.wait(1.2);

    // ---- unions (arrow = parent pointer) ---------------------------------
    m.section("Union");
    union(&mut m, 0, 1, "union(0, 1)  →  parent[0] = 1");
    union(&mut m, 2, 3, "union(2, 3)  →  parent[2] = 3");
    union(&mut m, 1, 3, "union(1, 3)  →  parent[1] = 3");
    union(&mut m, 4, 5, "union(4, 5)  →  parent[4] = 5");
    union(&mut m, 3, 5, "union(3, 5)  →  parent[3] = 5   (one tree, root 5)");
    m.wait(0.5);

    // ---- re-layout into the tree shape -----------------------------------
    m.play(act().set_text("caption", "same pointers, drawn as the tree they form"));
    let mut moves: Vec<Clip> = (0..N)
        .map(|i| act().move_to(&node(i), tree_pos(i)).dur(0.9).ease(InOutCubic).into())
        .collect();
    // arrows follow their child automatically; their heads must chase the
    // parent's new position in the same breath
    for (child, parent) in [(0usize, 1usize), (1, 3), (2, 3), (3, 5), (4, 5)] {
        moves.push(act().grow_to(&parrow(child), tree_pos(parent)).dur(0.9).ease(InOutCubic).into());
    }
    m.play(Clip::par(moves));
    m.wait(1.0);

    // ---- find(0) with path compression ------------------------------------
    m.section("find(0) + path compression");
    m.play(act().set_text("caption", "find(0): follow parent pointers to the root"));
    for step in [
        (node(0), parrow(0)),
        (node(1), parrow(1)),
        (node(3), parrow(3)),
    ] {
        m.play(seq![
            act().highlight(&step.0, BLUE).dur(0.55),
            act().highlight(&step.1, BLUE).dur(0.55),
        ]);
    }
    m.play(par![
        act().color_to(&node(5), ACCENT).dur(0.3),
        act().color_to(&format!("{}.label", node(5)), PAPER).dur(0.3),
        act().pulse(&node(5)).dur(0.6),
    ]);
    m.play(act().set_text("caption", "root found. now compress: point every visited node at the root"));
    m.wait(0.8);

    m.play(par![
        act().move_to(&node(0), v(200.0, 390.0)).dur(0.8).ease(InOutCubic),
        act().move_to(&node(1), v(320.0, 390.0)).dur(0.8).ease(InOutCubic),
        act().retarget(&parrow(0), tree_pos(5)).dur(0.8).ease(InOutCubic),
        act().retarget(&parrow(1), tree_pos(5)).dur(0.8).ease(InOutCubic),
    ]);
    m.play(par![
        act().pulse(&node(0)).dur(0.5),
        act().pulse(&node(1)).dur(0.5),
    ]);
    m.play(act().set_text(
        "caption",
        "find(0) is now a single hop — near-O(1) amortized (inverse Ackermann)",
    ));
    m.wait(2.5);

    broadsheet::run(m);
}
