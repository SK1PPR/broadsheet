//! The LSM Tree: writes land in a memtable (BST forming node by node),
//! flush to sorted runs on disk, compaction merges runs down the pyramid,
//! and a read walks newest-to-oldest.
//!
//! Run live:    cargo run --example lsm_tree
//! Record:      cargo run --example lsm_tree -- --record out/lsm --fps 60

use broadsheet::prelude::*;

const SPAWN: Vec2 = Vec2::new(280.0, 165.0);
const L0A: Vec2 = Vec2::new(545.0, 405.0);
const L0B: Vec2 = Vec2::new(735.0, 405.0);
const L1: Vec2 = Vec2::new(640.0, 480.0);
const L2: Vec2 = Vec2::new(640.0, 565.0);

fn node(k: u32) -> String {
    format!("k{k}")
}

fn edge(k: u32) -> String {
    format!("e{k}")
}

/// One `put(k)`: announce it, spawn the node, fly it to its BST slot, and
/// grow the parent edge behind it.
fn put(m: &mut Movie, k: u32, slot: Vec2, parent_slot: Option<Vec2>) {
    m.play(act().set_text("inlab", &format!("put({k})")).dur(0.4));
    m.play(par![
        act().fade_in(&node(k)).dur(0.25),
        act().move_to(&node(k), slot).dur(0.7).ease(InOutCubic),
    ]);
    if let Some(_p) = parent_slot {
        m.play(par![
            act().fade_in(&edge(k)).dur(0.1),
            act().grow_to(&edge(k), slot).dur(0.4).ease(OutCubic),
        ]);
    }
    m.play(act().pulse(&node(k)).dur(0.4));
    m.wait(2.4);
}

fn main() {
    let mut m = Movie::new("The LSM Tree", 1280, 720);

    // BST layout inside the memtable box: insert order 17, 8, 25, 3, 12
    let slots: [(u32, Vec2, Option<u32>); 5] = [
        (17, v(640.0, 165.0), None),
        (8, v(555.0, 225.0), Some(17)),
        (25, v(725.0, 225.0), Some(17)),
        (3, v(490.0, 285.0), Some(8)),
        (12, v(600.0, 285.0), Some(8)),
    ];
    let slot_of = |k: u32| slots.iter().find(|s| s.0 == k).unwrap().1;

    // ---- cast ----------------------------------------------------------
    {
        let mut s = m.scene();

        // memory region
        s.rect("membox", v(640.0, 225.0), 440.0, 210.0)
            .outlined()
            .outline_color(INK)
            .stroke(2.5)
            .hidden();
        s.text("memlabel", v(640.0, 105.0), "MEMTABLE — RAM")
            .size(16.0)
            .color(FADED)
            .hidden();
        s.line("divider", v(60.0, 345.0), v(1220.0, 345.0))
            .color(FADED)
            .stroke(1.5)
            .hidden();
        s.text("ramlab", v(110.0, 328.0), "MEMORY")
            .size(13.0)
            .color(FADED)
            .hidden();
        s.text("disklab", v(100.0, 364.0), "DISK")
            .size(13.0)
            .color(FADED)
            .hidden();

        // BST edges under nodes
        for (k, _, parent) in slots.iter() {
            if let Some(p) = parent {
                let pslot = slots.iter().find(|s| s.0 == *p).unwrap().1;
                s.arrow(&edge(*k), pslot, pslot)
                    .color(INK)
                    .stroke(2.0)
                    .z(1)
                    .hidden();
            }
        }
        // BST nodes spawn off to the left
        for (k, _, _) in slots.iter() {
            s.circle(&node(*k), SPAWN, 26.0)
                .stroke(2.5)
                .z(5)
                .hidden()
                .label(&k.to_string());
        }
        s.text("inlab", v(170.0, 165.0), "")
            .mono_bold()
            .size(22.0)
            .color(ACCENT)
            .hidden();

        // disk levels (the pyramid)
        s.polygon(
            "pyramid",
            vec![v(640.0, 383.0), v(990.0, 590.0), v(290.0, 590.0)],
        )
        .outlined()
        .outline_color(FADED)
        .stroke(1.5)
        .z(0)
        .hidden();
        s.rect("run0a", L0A, 180.0, 36.0)
            .outline_color(INK)
            .stroke(2.0)
            .z(3)
            .hidden();
        s.text("run0a.tag", v(0.0, 0.0), "3 · 8 · 12 · 17 · 25")
            .size(13.0)
            .z(4)
            .follow("run0a", v(0.0, 0.0));
        s.rect("run0b", L0B, 180.0, 36.0)
            .outline_color(INK)
            .stroke(2.0)
            .z(3)
            .hidden();
        s.text("run0b.tag", v(0.0, 0.0), "5 · 9 · 21 · 40")
            .size(13.0)
            .z(4)
            .follow("run0b", v(0.0, 0.0));
        s.rect("run1", L1, 380.0, 40.0)
            .outline_color(INK)
            .stroke(2.0)
            .z(3)
            .hidden();
        s.text("run1.tag", v(0.0, 0.0), "3 … 40  (merged, sorted)")
            .size(13.0)
            .z(4)
            .follow("run1", v(0.0, 0.0));
        s.rect("run2", L2, 700.0, 44.0)
            .outline_color(INK)
            .stroke(2.0)
            .z(3)
            .hidden();
        s.text("run2.tag", v(0.0, 0.0), "0 … 99  (old, big, cold)")
            .size(13.0)
            .z(4)
            .follow("run2", v(0.0, 0.0));
        for (id, pos) in [("l0lab", L0A), ("l1lab", L1), ("l2lab", L2)] {
            s.text(id, v(240.0, pos.y), &id[..2].to_uppercase())
                .size(15.0)
                .color(FADED)
                .hidden();
        }

        // read path
        s.text("getlab", v(1080.0, 165.0), "get(12)")
            .mono_bold()
            .size(24.0)
            .color(BLUE)
            .hidden();
        s.text("found", v(1010.0, 480.0), "FOUND")
            .serif()
            .size(34.0)
            .color(ACCENT)
            .rot(-6.0)
            .z(50)
            .hidden();

        s.text("caption", v(640.0, 650.0), "")
            .size(20.0)
            .color(FADED)
            .wrap(1060.0)
            .hidden();
    }

    // ---- intro -----------------------------------------------------------
    m.wait(3.0);
    m.play(act().set_text(
        "caption",
        "a write-optimized storage engine: writes never touch disk directly",
    ));
    m.wait(5.0);

    // ---- §1 the memtable ---------------------------------------------------
    m.section("The Memtable");
    m.play(par![
        act().fade_in("membox").dur(0.6),
        act().fade_in("memlabel").dur(0.6),
        act().fade_in("divider").dur(0.6),
        act().fade_in("ramlab").dur(0.6),
        act().fade_in("disklab").dur(0.6),
    ]);
    m.play(act().set_text(
        "caption",
        "every put() lands in an in-memory sorted structure — O(log n), no disk I/O",
    ));
    m.play(act().fade_in("inlab").dur(0.2));
    m.wait(0.8);
    for (k, slot, parent) in slots.iter() {
        put(&mut m, *k, *slot, parent.map(slot_of));
    }
    m.play(act().set_text(
        "caption",
        "sorted in memory, ready to be written out in one sequential pass",
    ));
    m.wait(6.0);

    // ---- §2 flush ----------------------------------------------------------
    m.section("Flush");
    m.play(act().set_text(
        "caption",
        "memtable full → freeze it and flush: one immutable sorted run (SSTable)",
    ));
    m.play(act().fade_out("inlab").dur(0.3));
    m.play(act().fade_in("l0lab").dur(0.4));
    {
        let mut merge: Vec<Clip> = Vec::new();
        for (k, _, parent) in slots.iter() {
            merge.push(
                act()
                    .move_to(&node(*k), L0A)
                    .dur(0.8)
                    .ease(InOutCubic)
                    .into(),
            );
            merge.push(act().fade_out(&node(*k)).dur(0.8).into());
            if parent.is_some() {
                merge.push(act().fade_out(&edge(*k)).dur(0.4).into());
            }
        }
        m.play(Clip::par(merge));
    }
    m.play(par![
        act().fade_in("run0a").dur(0.5),
        act().pulse("run0a").dur(0.5)
    ]);
    m.wait(4.5);
    m.play(act().set_text(
        "caption",
        "writes keep coming — the next memtable flushes beside it",
    ));
    m.play(par![
        act().fade_in("run0b").dur(0.5),
        act().pulse("run0b").dur(0.5)
    ]);
    m.wait(6.5);

    // ---- §3 compaction -------------------------------------------------------
    m.section("Compaction");
    m.play(act().set_text(
        "caption",
        "L0 runs overlap — merge-sort them downward into one bigger run",
    ));
    m.play(par![
        act().highlight("run0a", ACCENT).dur(0.8),
        act().highlight("run0b", ACCENT).dur(0.8),
    ]);
    m.play(act().fade_in("l1lab").dur(0.4));
    m.play(par![
        act().move_to("run0a", L1).dur(0.9).ease(InOutCubic),
        act().move_to("run0b", L1).dur(0.9).ease(InOutCubic),
        act().fade_out("run0a").dur(0.9),
        act().fade_out("run0b").dur(0.9),
    ]);
    m.play(par![
        act().fade_in("run1").dur(0.5),
        act().pulse("run1").dur(0.5)
    ]);
    m.wait(5.0);

    m.play(act().set_text(
        "caption",
        "older data lives lower: bigger, colder, rarely rewritten",
    ));
    m.play(par![
        act().fade_in("l2lab").dur(0.4),
        act().fade_in("run2").dur(0.6)
    ]);
    m.wait(4.0);
    m.play(act().fade_in("pyramid").dur(0.8));
    m.play(act().set_text(
        "caption",
        "the shape is the point: small & fresh on top, wide & old below — a pyramid of sorted runs",
    ));
    m.wait(6.5);

    // a fresh flush lands on top so the read path has an L0 run to probe
    m.play(act().move_to("run0a", L0A).dur(0.01));
    m.play(par![
        act().fade_in("run0a").dur(0.5),
        act().set_text("run0a.tag", "7 · 33 · 48").dur(0.4),
    ]);
    m.play(act().set_text("caption", "meanwhile, new flushes keep arriving on top"));
    m.wait(3.0);

    // ---- §4 reads --------------------------------------------------------------
    m.section("The Read Path");
    m.play(act().set_text(
        "caption",
        "get(12): check newest data first, walk down until found",
    ));
    m.play(act().fade_in("getlab").dur(0.4));
    m.wait(1.5);
    m.play(seq![
        act().highlight("membox", BLUE).dur(0.9),
        wait(0.8),
        act().highlight("run0a", BLUE).dur(0.9),
        wait(0.8),
    ]);
    m.play(act().set_text(
        "caption",
        "not in the memtable, not in L0 … (bloom filters skip most of these probes)",
    ));
    m.wait(2.5);
    m.play(seq![
        act().highlight("run1", BLUE).dur(0.9),
        par![
            act().color_to("run1", ACCENT).dur(0.4),
            act().pulse("run1").dur(0.6),
        ],
    ]);
    m.play(par![
        act().fade_in("found").dur(0.3),
        act().pulse("found").dur(0.6)
    ]);
    m.play(act().set_text(
        "caption",
        "found in L1 — worst case one probe per level, so keep the pyramid shallow",
    ));
    m.wait(8.0);

    // ---- outro -------------------------------------------------------------
    m.play(par![
        act().fade_out("getlab").dur(0.5),
        act().fade_out("found").dur(0.5),
        act().color_to("run1", PAPER).dur(0.5),
    ]);
    m.play(act().set_text(
        "caption",
        "sequential writes, merge-sorted reads — the engine inside RocksDB, Cassandra & Kafka's cousin, the log",
    ));
    m.wait(9.0);

    broadsheet::run(m);
}
