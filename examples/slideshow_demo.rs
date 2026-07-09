//! A widget tour presented as a slideshow, in the midnight theme.
//!
//! Present it live:
//!
//! ```sh
//! cargo run --example slideshow_demo -- --slideshow
//! ```
//!
//! `Space`/`→` animates to the next slide, `←` snaps back one. Without
//! `--slideshow` it plays through like any other movie (and records the
//! same way: `-- --record out/tour`).

use broadsheet::prelude::*;
use broadsheet::widgets;

/// Keep `tag`'s entities invisible from t = 0 (zero-duration fade at start).
fn hide_at_start(m: &mut Movie, tag: &str) {
    let ids = m.tagged(tag);
    m.at(0.0, all(&ids, |id| act().fade_out(id).dur(0.0)));
}

fn reveal(m: &mut Movie, tag: &str) {
    let ids = m.tagged(tag);
    m.play(all(&ids, |id| act().fade_in(id).dur(0.4)));
}

fn conceal(m: &mut Movie, tag: &str) {
    let ids = m.tagged(tag);
    m.play(all(&ids, |id| act().fade_out(id).dur(0.3)));
}

fn main() {
    let mut m = Movie::new("Data Structure Tour", 1280, 720);
    m.set_theme(Theme::midnight());

    m.scene().text("cap", v(640., 580.), "").size(24.).hidden();

    // ---- slide 1: array ------------------------------------------------
    m.slide("array");
    let arr = widgets::array(
        &mut m,
        "arr",
        &["4", "8", "15", "16", "23", "42"],
        v(640., 320.),
    );
    m.play(par![
        act().set_text("cap", "Linear search: scan until the value hits."),
        act().fade_in("cap"),
    ]);
    for i in 0..3 {
        m.play(
            act()
                .highlight(&arr.cell(i), m.role(Role::Visited))
                .dur(0.55),
        );
    }
    m.play(act().color_to(&arr.cell(3), m.role(Role::Found)));
    m.play(act().pulse(&arr.cell(3)));
    m.wait(0.4);

    // ---- slide 2: linked list -------------------------------------------
    m.slide("linked list");
    conceal(&mut m, "arr");
    let list = widgets::linked_list(&mut m, "list", &["A", "B", "C", "D"], v(400., 320.));
    hide_at_start(&mut m, "list");
    reveal(&mut m, "list");
    m.play(act().set_text("cap", "Delete C: repoint B, drop the node."));
    m.play(act().color_to(&list.node(2), m.role(Role::Deleted)));
    m.play(par![
        act()
            .retarget(&list.next(1), list.pos[3] - v(36., 0.))
            .dur(0.6),
        act().fade_out(&list.node(2)),
        act().fade_out(&list.next(2)),
    ]);
    m.wait(0.4);

    // ---- slide 3: tree -----------------------------------------------------
    m.slide("tree");
    conceal(&mut m, "list");
    let parents = [None, Some(0), Some(0), Some(1), Some(1), Some(2), Some(2)];
    let bst = widgets::tree(
        &mut m,
        "bst",
        &parents,
        &["16", "8", "23", "4", "15", "19", "42"],
        v(640., 220.),
        95.,
        105.,
    );
    hide_at_start(&mut m, "bst");
    reveal(&mut m, "bst");
    m.play(act().set_text("cap", "BST lookup for 15: left, then right."));
    for i in [0usize, 1, 4] {
        let role = if i == 4 { Role::Found } else { Role::Active };
        m.play(act().color_to(&bst.node(i), m.role(role)).dur(0.45));
    }
    m.play(act().pulse(&bst.node(4)));
    m.wait(0.4);

    // ---- slide 4: graph -----------------------------------------------------
    m.slide("graph");
    conceal(&mut m, "bst");
    let edges = [(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (1, 4)];
    let g = widgets::graph(&mut m, "g", &["a", "b", "c", "d", "e"], &edges, None);
    hide_at_start(&mut m, "g");
    reveal(&mut m, "g");
    m.play(act().set_text(
        "cap",
        "Deterministic spring layout: same seed, same picture.",
    ));
    m.play(act().color_to(&g.node(0), m.role(Role::Active)));
    m.play(par![
        act().color_to(&g.edge(0, 1), m.role(Role::Visited)),
        act().color_to(&g.edge(0, 2), m.role(Role::Visited)),
    ]);
    m.play(par![
        act().color_to(&g.node(1), m.role(Role::Visited)),
        act().color_to(&g.node(2), m.role(Role::Visited)),
    ]);
    m.wait(0.4);

    // ---- slide 5: hash ring ---------------------------------------------------
    m.slide("hash ring");
    conceal(&mut m, "g");
    let ring = widgets::hash_ring(
        &mut m,
        "ring",
        v(640., 330.),
        170.,
        &[(0.05, "n1"), (0.30, "n2"), (0.60, "n3"), (0.85, "n4")],
    );
    m.scene()
        .circle("key", v(640., 330.), 10.)
        .role(Role::Active)
        .filled()
        .z(5)
        .hidden();
    hide_at_start(&mut m, "ring");
    reveal(&mut m, "ring");
    m.play(act().set_text(
        "cap",
        "Consistent hashing: the key walks to its clockwise successor.",
    ));
    m.play(act().fade_in("key"));
    m.play(
        act()
            .move_to("key", ring.at(0.22))
            .dur(0.7)
            .ease(InOutCubic),
    );
    m.play(
        act()
            .move_to("key", ring.node_pos(1))
            .dur(0.6)
            .ease(OutBack),
    );
    m.play(act().pulse(&ring.node(1)));
    m.wait(0.4);

    // ---- slide 6: skip list ---------------------------------------------------
    m.slide("skip list");
    conceal(&mut m, "ring");
    m.play(act().fade_out("key").dur(0.3));
    let sl = widgets::skip_list(
        &mut m,
        "sl",
        &["3", "7", "12", "19", "25"],
        &[vec![0, 2, 4], vec![0, 4]],
        v(420., 460.),
    );
    hide_at_start(&mut m, "sl");
    reveal(&mut m, "sl");
    m.play(act().set_text(
        "cap",
        "Skip list search for 19: ride the express lanes down.",
    ));
    for (k, c) in [(2usize, 0usize), (1, 0), (1, 2), (0, 2), (0, 3)] {
        let role = if (k, c) == (0, 3) {
            Role::Found
        } else {
            Role::Active
        };
        m.play(act().color_to(&sl.node(k, c), m.role(role)).dur(0.4));
    }
    m.wait(0.4);

    // ---- slide 7: LSM levels ----------------------------------------------------
    m.slide("lsm levels");
    conceal(&mut m, "sl");
    let lsm = widgets::lsm_levels(&mut m, "lsm", &[1, 2, 4], v(640., 220.));
    hide_at_start(&mut m, "lsm");
    reveal(&mut m, "lsm");
    m.play(act().set_text("cap", "Compaction: the L0 run merges down into L1."));
    m.play(act().color_to(&lsm.block(0, 0), m.role(Role::Stale)));
    m.play(par![
        act().move_to(&lsm.block(0, 0), lsm.pos[1][0]).dur(0.7),
        act().highlight(&lsm.block(1, 0), m.role(Role::Active)),
    ]);
    m.play(act().fade_out(&lsm.block(0, 0)));
    m.wait(0.8);

    broadsheet::run(m);
}
