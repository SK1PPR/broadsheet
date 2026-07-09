//! The Bloom Filter: a bit array that stores nothing, sometimes lies,
//! and skips the expensive lookup anyway. Beats follow
//! content/bloom-filter-script.md (mark names match the script's brackets).
//!
//! Run live:    cargo run --example bloom_filter
//! Record:      cargo run --example bloom_filter -- --record renders/bloom --fps 60 --grain

use broadsheet::prelude::*;

const BITS: usize = 32;
const BIT_Y: f32 = 330.0;
const STRIDE: f32 = 36.0;

fn bit_x(i: usize) -> f32 {
    640.0 - STRIDE * (BITS as f32 - 1.0) / 2.0 + STRIDE * i as f32
}

fn bit(i: usize) -> String {
    format!("bit{i}")
}

/// Point the three probe arrows at bit cells `ps` and light the hash boxes.
fn probe(m: &mut Movie, ps: [usize; 3], c: Color) {
    let hx = [430.0, 640.0, 850.0];
    for (j, p) in ps.iter().enumerate() {
        let from = v(hx[j], 202.0);
        let to = v(bit_x(*p), BIT_Y - 30.0);
        m.play(par![
            act().move_to(&format!("arr{j}"), from).dur(0.01),
            act().grow_to(&format!("arr{j}"), from).dur(0.01),
        ]);
        m.play(par![
            act().fade_in(&format!("arr{j}")).dur(0.15),
            act().pulse(&format!("h{}", j + 1)).dur(0.3),
            act()
                .grow_to(&format!("arr{j}"), to)
                .dur(0.5)
                .ease(OutCubic),
            act().highlight(&bit(*p), c).dur(0.6),
        ]);
    }
}

fn retract(m: &mut Movie) {
    m.play(par![
        act().fade_out("arr0").dur(0.3),
        act().fade_out("arr1").dur(0.3),
        act().fade_out("arr2").dur(0.3),
    ]);
}

/// Insert `word`: announce, probe, flip the three bits to 1.
fn insert(m: &mut Movie, word: &str, ps: [usize; 3]) {
    m.play(
        act()
            .set_text("word", &format!("insert(\"{word}\")"))
            .dur(0.3),
    );
    m.play(act().pulse("word").dur(0.4));
    probe(m, ps, ACCENT);
    let mut flips: Vec<Clip> = Vec::new();
    for p in ps {
        flips.push(
            act()
                .set_text(&format!("bit{p}.label"), "1")
                .dur(0.2)
                .into(),
        );
        flips.push(act().color_to(&bit(p), PAPER_SHADE).dur(0.6).into());
    }
    m.play(Clip::par(flips));
}

fn caption(m: &mut Movie, text: &str) {
    m.play(act().set_text("caption", text).dur(0.3));
}

fn main() {
    let mut m = Movie::new("The Bloom Filter", 1280, 720);

    // ---- cast ----------------------------------------------------------
    {
        let mut s = m.scene();

        s.text("caption", v(640.0, 655.0), "")
            .size(20.0)
            .color(FADED)
            .wrap(1100.0)
            .hidden();

        // §1 hook
        s.rect("db", v(250.0, 280.0), 240.0, 120.0)
            .outlined()
            .outline_color(INK)
            .stroke(2.5)
            .hidden()
            .label("DATABASE");
        s.text("key", v(-160.0, 280.0), "get(user_42)")
            .mono_bold()
            .size(24.0)
            .color(BLUE)
            .hidden();
        for i in 0..4 {
            s.rect(
                &format!("file{i}"),
                v(1030.0, 150.0 + 90.0 * i as f32),
                260.0,
                64.0,
            )
            .outline_color(INK)
            .stroke(2.0)
            .hidden()
            .label(&format!("SST-{}.db", i + 1));
            s.text(&format!("file{i}.ms"), v(0.0, 0.0), "~5 ms")
                .size(13.0)
                .color(FADED)
                .follow(&format!("file{i}"), v(160.0, 0.0));
        }
        s.rect("fbox", v(620.0, 280.0), 230.0, 96.0)
            .outlined()
            .outline_color(ACCENT)
            .stroke(3.0)
            .hidden()
            .label("BLOOM FILTER");
        s.text("fboxsub", v(0.0, 0.0), "~1 byte per key · in RAM")
            .size(14.0)
            .color(FADED)
            .follow("fbox", v(0.0, 68.0));
        s.text("ansno", v(620.0, 460.0), "\u{201C}DEFINITELY NOT.\u{201D}")
            .serif()
            .size(34.0)
            .color(INK)
            .rot(-3.0)
            .hidden();
        s.text("ansmaybe", v(620.0, 520.0), "\u{201C}PROBABLY.\u{201D}")
            .serif()
            .size(34.0)
            .color(ACCENT)
            .rot(-3.0)
            .hidden();

        // §2 history
        s.rect("paper", v(390.0, 300.0), 480.0, 230.0)
            .outlined()
            .outline_color(INK)
            .stroke(2.0)
            .hidden();
        s.text(
            "paper1",
            v(390.0, 240.0),
            "\u{201C}Space/Time Trade-offs in Hash Coding",
        )
        .serif()
        .size(22.0)
        .hidden();
        s.text("paper2", v(390.0, 272.0), "with Allowable Errors\u{201D}")
            .serif()
            .size(22.0)
            .hidden();
        s.text("paper3", v(390.0, 330.0), "Burton H. Bloom · CACM · 1970")
            .size(17.0)
            .color(FADED)
            .hidden();
        s.text(
            "paper4",
            v(390.0, 368.0),
            "5 pages. One equation. Everywhere.",
        )
        .size(15.0)
        .color(ACCENT)
        .hidden();
        s.rect("dict", v(960.0, 300.0), 330.0, 200.0)
            .outline_color(INK)
            .stroke(2.0)
            .hidden()
            .label("");
        s.text("dict1", v(960.0, 240.0), "hyphenation dictionary")
            .size(17.0)
            .hidden();
        s.text("dict2", v(960.0, 285.0), "500,000 words")
            .mono_bold()
            .size(24.0)
            .hidden();
        s.text("dict3", v(960.0, 330.0), "90% follow simple rules")
            .size(16.0)
            .color(FADED)
            .hidden();
        s.text("dict4", v(960.0, 362.0), "10% need the disk")
            .size(16.0)
            .color(ACCENT)
            .hidden();

        // §3 mechanics: the machine
        s.cells(
            "bit",
            BITS,
            v(640.0, BIT_Y),
            v(30.0, 36.0),
            6.0,
            Some(&["0"; BITS]),
        );
        for (i, x) in [430.0_f32, 640.0, 850.0].iter().enumerate() {
            s.rect(&format!("h{}", i + 1), v(*x, 180.0), 92.0, 44.0)
                .outline_color(INK)
                .stroke(2.0)
                .hidden()
                .label(&format!("h{}", i + 1));
        }
        s.text("word", v(170.0, 180.0), "")
            .mono_bold()
            .size(22.0)
            .color(ACCENT)
            .hidden();
        for j in 0..3 {
            s.arrow(&format!("arr{j}"), v(640.0, 202.0), v(640.0, 202.0))
                .color(INK)
                .stroke(2.0)
                .z(10)
                .hidden();
        }
        s.text("stampno", v(640.0, 520.0), "DEFINITELY NO")
            .serif()
            .size(42.0)
            .color(INK)
            .rot(-4.0)
            .z(50)
            .hidden();
        s.text("stampyes", v(640.0, 520.0), "PROBABLY YES")
            .serif()
            .size(42.0)
            .color(BLUE)
            .rot(-4.0)
            .z(50)
            .hidden();
        s.text("stampwrong", v(950.0, 520.0), "\u{2026}AND IT'S WRONG")
            .serif()
            .size(30.0)
            .color(ACCENT)
            .rot(-4.0)
            .z(50)
            .hidden();
        s.text("asym1", v(640.0, 250.0), "\u{201C}NO\u{201D} MEANS NO.")
            .serif()
            .size(52.0)
            .hidden();
        s.text("asym2", v(640.0, 340.0), "\u{201C}YES\u{201D} MEANS MAYBE.")
            .serif()
            .size(52.0)
            .color(ACCENT)
            .hidden();

        // §4 tuning
        for (i, (id, txt)) in [
            ("knobm", "m \u{2014} bits in the array"),
            ("knobn", "n \u{2014} items you will insert"),
            ("knobk", "k \u{2014} hash functions"),
        ]
        .iter()
        .enumerate()
        {
            s.text(id, v(330.0, 200.0 + 60.0 * i as f32), txt)
                .mono_bold()
                .size(24.0)
                .left()
                .hidden();
        }
        s.text(
            "formula",
            v(640.0, 430.0),
            "k_optimal = (m / n) \u{00B7} ln 2",
        )
        .mono_bold()
        .size(32.0)
        .color(ACCENT)
        .hidden();
        s.rect("rulebox", v(950.0, 250.0), 340.0, 190.0)
            .outlined()
            .outline_color(ACCENT)
            .stroke(2.5)
            .hidden();
        s.text("rule0", v(950.0, 185.0), "THE RULE OF THUMB")
            .size(15.0)
            .color(FADED)
            .hidden();
        s.text("rule1", v(950.0, 225.0), "10 bits per item")
            .mono_bold()
            .size(22.0)
            .hidden();
        s.text("rule2", v(950.0, 262.0), "7 hash functions")
            .mono_bold()
            .size(22.0)
            .hidden();
        s.text("rule3", v(950.0, 299.0), "\u{2248} 1% false positives")
            .mono_bold()
            .size(22.0)
            .color(ACCENT)
            .hidden();
        s.text("rulex", v(950.0, 480.0), "10\u{2078} URLs \u{2192} 120 MB")
            .size(20.0)
            .color(FADED)
            .hidden();

        // §5 variants: four cards
        let vx = [220.0_f32, 500.0, 780.0, 1060.0];
        for (i, (id, title, sub)) in [
            (
                "vc",
                "COUNTING",
                "bits \u{2192} counters\ndeletes work · 4\u{00D7} memory",
            ),
            (
                "vb",
                "BLOCKED",
                "k bits, one cache line\nwhat RocksDB ships",
            ),
            ("vs", "SCALABLE", "full? freeze it,\nstack a new layer"),
            ("vk", "CUCKOO", "fingerprint table\ndeletes + tighter space"),
        ]
        .iter()
        .enumerate()
        {
            s.rect(id, v(vx[i], 290.0), 250.0, 170.0)
                .outline_color(INK)
                .stroke(2.0)
                .hidden();
            s.text(&format!("{id}.t"), v(0.0, 0.0), title)
                .serif()
                .size(24.0)
                .follow(id, v(0.0, -55.0));
            s.text(&format!("{id}.s"), v(0.0, 0.0), &sub.replace('\n', " "))
                .size(15.0)
                .color(FADED)
                .wrap(220.0)
                .follow(id, v(0.0, 15.0));
        }

        // §6 uses
        for (i, (id, txt)) in [
            (
                "use1",
                "RocksDB · Cassandra · HBase \u{2014} skip SSTables on point reads",
            ),
            (
                "use2",
                "Chrome \u{2014} pre-screen URLs against the malware list",
            ),
            (
                "use3",
                "Akamai \u{2014} cache only what's been asked for twice",
            ),
            ("use4", "Bitcoin SPV · Medium recs · join pruning \u{2026}"),
        ]
        .iter()
        .enumerate()
        {
            s.text(id, v(180.0, 190.0 + 85.0 * i as f32), txt)
                .size(22.0)
                .left()
                .hidden();
        }
        s.text(
            "usehead",
            v(640.0, 120.0),
            "\u{201C}CAN I SKIP THE EXPENSIVE THING?\u{201D}",
        )
        .serif()
        .size(30.0)
        .hidden();

        // §7 outro
        s.text("outro1", v(640.0, 260.0), "Perfect answers are expensive.")
            .serif()
            .size(40.0)
            .hidden();
        s.text(
            "outro2",
            v(640.0, 330.0),
            "Calibrated doubt is nearly free.",
        )
        .serif()
        .size(40.0)
        .color(ACCENT)
        .hidden();
        s.text(
            "next",
            v(640.0, 480.0),
            "Next issue: THE SKIP LIST \u{2014} definitely subscribe. Probably.",
        )
        .size(20.0)
        .color(FADED)
        .hidden();
    }

    // cells() spawns visible; the bit array belongs to §3, so duck it at t=0
    {
        let ids = m.tagged("bit");
        m.at(0.0, all(&ids, |id| act().fade_out(id).dur(0.01)));
    }

    // ================= §1 THE HOOK — 0:00–0:55 =========================
    m.section("The Hook");
    m.mark("hook");
    m.wait(2.0);
    caption(
        &mut m,
        "a read arrives \u{2014} the key could be in any of a dozen files on disk",
    );
    m.play(par![
        act().fade_in("db").dur(0.6),
        act().fade_in("file0").dur(0.6),
        act().fade_in("file1").dur(0.6),
        act().fade_in("file2").dur(0.6),
        act().fade_in("file3").dur(0.6),
    ]);
    m.play(par![
        act().fade_in("key").dur(0.3),
        act()
            .move_to("key", v(250.0, 200.0))
            .dur(1.2)
            .ease(InOutCubic),
    ]);
    m.wait(3.0);
    caption(
        &mut m,
        "opening every file to check: milliseconds, plural \u{2014} an eternity",
    );
    m.play(seq![
        act().highlight("file0", ACCENT).dur(0.5),
        act().highlight("file1", ACCENT).dur(0.5),
        act().highlight("file2", ACCENT).dur(0.5),
        act().highlight("file3", ACCENT).dur(0.5),
    ]);
    m.wait(6.0);

    m.mark("hook-question");
    caption(
        &mut m,
        "so first it asks a tiny structure in RAM: \u{201C}have you seen this key?\u{201D}",
    );
    m.play(par![
        act().fade_in("fbox").dur(0.5),
        act().pulse("fbox").dur(0.6),
    ]);
    m.wait(4.0);
    m.play(act().fade_in("ansno").dur(0.4));
    m.wait(2.5);
    m.play(par![
        act().fade_out("ansno").dur(0.3),
        act().fade_in("ansmaybe").dur(0.4),
    ]);
    m.wait(3.5);

    m.mark("hook-reveal");
    caption(
        &mut m,
        "it can be wrong \u{2014} and being wrong in exactly one direction is the entire trick",
    );
    m.play(act().highlight("fbox", ACCENT).dur(0.8));
    m.wait(9.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §2 1970 — 0:55–1:50 =============================
    m.section("1970");
    m.mark("history");
    caption(
        &mut m,
        "memory measured in kilobytes, priced like real estate",
    );
    m.play(par![
        act().fade_in("paper").dur(0.7),
        act().fade_in("paper1").dur(0.7),
        act().fade_in("paper2").dur(0.7),
        act().fade_in("paper3").dur(0.7),
    ]);
    m.wait(8.0);

    m.mark("history-problem");
    caption(
        &mut m,
        "Bloom's problem: hyphenation \u{2014} the whole dictionary can't fit in memory",
    );
    m.play(par![
        act().fade_in("dict").dur(0.6),
        act().fade_in("dict1").dur(0.6),
        act().fade_in("dict2").dur(0.6),
    ]);
    m.wait(3.0);
    m.play(par![
        act().fade_in("dict3").dur(0.5),
        act().fade_in("dict4").dur(0.5),
    ]);
    caption(&mut m, "don't store the words \u{2014} only answer \u{201C}could this be one of the tricky ones?\u{201D}");
    m.wait(9.0);

    m.mark("history-insight");
    caption(&mut m, "allow a little error, save a lot of space");
    m.play(par![
        act().fade_in("paper4").dur(0.5),
        act().pulse("paper4").dur(0.6),
    ]);
    m.wait(9.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §3 HOW IT WORKS — 1:50–4:00 =====================
    m.section("The Machine");
    m.mark("mechanics");
    caption(
        &mut m,
        "the whole machine: a bit array, all zeros \u{2014} and a few deterministic scramblers",
    );
    {
        let ids = m.tagged("bit");
        m.play(stagger(
            0.02,
            ids.iter()
                .map(|id| act().fade_in(id).dur(0.3).into())
                .collect(),
        ));
    }
    m.play(par![
        act().fade_in("h1").dur(0.4),
        act().fade_in("h2").dur(0.4),
        act().fade_in("h3").dur(0.4),
        act().fade_in("word").dur(0.4),
    ]);
    m.wait(7.0);

    m.mark("insert-1");
    caption(&mut m, "insert \u{201C}cat\u{201D}: three hashes say 5, 11, 26 \u{2014} flip those bits. that's the whole insert.");
    insert(&mut m, "cat", [5, 11, 26]);
    m.wait(4.0);
    caption(&mut m, "we did not store \u{201C}cat\u{201D} \u{2014} no letters, no pointers. three ones in a sea of zeros.");
    m.wait(6.0);
    retract(&mut m);

    m.mark("insert-2");
    caption(
        &mut m,
        "insert \u{201C}dog\u{201D}: 3, 11, 20 \u{2014} bit 11 again. collisions are fine.",
    );
    insert(&mut m, "dog", [3, 11, 20]);
    m.play(act().pulse(&bit(11)).dur(0.5));
    m.wait(4.5);
    retract(&mut m);

    m.mark("insert-3");
    caption(
        &mut m,
        "insert \u{201C}fish\u{201D}: 8, 14, 26 \u{2014} overlapping fingerprints pile up",
    );
    insert(&mut m, "fish", [8, 14, 26]);
    m.wait(4.0);
    retract(&mut m);

    m.mark("query-miss");
    caption(&mut m, "query \u{201C}bird\u{201D}: 4, 14, 22 \u{2014} bit 4 is ZERO. stop. it was never inserted.");
    m.play(act().set_text("word", "query(\"bird\")").dur(0.3));
    probe(&mut m, [4, 14, 22], BLUE);
    m.wait(1.5);
    m.play(par![
        act().shake(&bit(4)).dur(0.6),
        act().highlight(&bit(4), ACCENT).dur(0.8),
    ]);
    m.play(par![
        act().fade_in("stampno").dur(0.3),
        act().pulse("stampno").dur(0.5),
    ]);
    caption(
        &mut m,
        "one zero is proof of absence \u{2014} no disk touched",
    );
    m.wait(7.0);
    retract(&mut m);
    m.play(act().fade_out("stampno").dur(0.4));

    m.mark("query-hit");
    caption(
        &mut m,
        "query \u{201C}cat\u{201D}: 5, 11, 26 \u{2014} all ones. answer: probably yes.",
    );
    m.play(act().set_text("word", "query(\"cat\")").dur(0.3));
    probe(&mut m, [5, 11, 26], BLUE);
    m.play(act().fade_in("stampyes").dur(0.3));
    m.wait(6.0);
    retract(&mut m);
    m.play(act().fade_out("stampyes").dur(0.4));

    m.mark("false-positive");
    caption(&mut m, "query \u{201C}cow\u{201D} \u{2014} never inserted. 3 lit by dog, 8 by fish, 20 by dog\u{2026}");
    m.play(act().set_text("word", "query(\"cow\")").dur(0.3));
    probe(&mut m, [3, 8, 20], BLUE);
    m.wait(1.0);
    m.play(act().fade_in("stampyes").dur(0.3));
    m.wait(1.5);
    m.play(par![
        act().fade_in("stampwrong").dur(0.3),
        act().pulse("stampwrong").dur(0.6),
    ]);
    caption(
        &mut m,
        "a FALSE POSITIVE: other members' fingerprints happen to cover yours",
    );
    m.wait(8.0);
    retract(&mut m);
    m.clear_all(0.8);
    m.wait(1.0);

    m.mark("asymmetry");
    m.play(act().fade_in("asym1").dur(0.6));
    m.wait(2.0);
    m.play(par![
        act().fade_in("asym2").dur(0.6),
        act().pulse("asym2").dur(0.8),
    ]);
    caption(
        &mut m,
        "never misses a real member; only occasionally cries wolf. no listing, no deletes.",
    );
    m.wait(10.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §4 TUNING — 4:00–5:20 ===========================
    m.section("Tuning");
    m.mark("tuning");
    caption(
        &mut m,
        "how wrong is it? that's a dial you set. three knobs.",
    );
    m.play(stagger(
        0.5,
        vec![
            act().fade_in("knobm").dur(0.5).into(),
            act().fade_in("knobn").dur(0.5).into(),
            act().fade_in("knobk").dur(0.5).into(),
        ],
    ));
    m.wait(9.0);

    m.mark("tuning-k");
    caption(
        &mut m,
        "more hashes = more specific fingerprints \u{2014} but too many lights up the whole array",
    );
    m.play(par![
        act().fade_in("formula").dur(0.5),
        act().pulse("formula").dur(0.7),
    ]);
    m.wait(14.0);

    m.mark("tuning-rule");
    caption(&mut m, "the numbers every engineer memorizes:");
    m.play(par![
        act().fade_in("rulebox").dur(0.5),
        act().fade_in("rule0").dur(0.5),
    ]);
    m.play(stagger(
        0.6,
        vec![
            act().fade_in("rule1").dur(0.4).into(),
            act().fade_in("rule2").dur(0.4).into(),
            act().fade_in("rule3").dur(0.4).into(),
        ],
    ));
    m.wait(5.0);
    m.play(act().fade_in("rulex").dur(0.5));
    caption(
        &mut m,
        "100 million crawled URLs \u{2192} 120 MB at 1% error. the URLs themselves: gigabytes.",
    );
    m.wait(12.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §5 UPGRADES — 5:20–6:35 =========================
    m.section("The Upgrades");
    m.mark("variants");
    caption(&mut m, "fifty years of bolted-on upgrades");
    m.wait(3.0);

    m.mark("variant-counting");
    m.play(act().fade_in("vc").dur(0.5));
    caption(
        &mut m,
        "counting: bits become counters \u{2014} deletes work, memory \u{00D7}4. nothing is free.",
    );
    m.wait(13.0);

    m.mark("variant-blocked");
    m.play(act().fade_in("vb").dur(0.5));
    caption(
        &mut m,
        "blocked: all k bits inside one 64-byte cache line \u{2014} one memory fetch, not seven",
    );
    m.wait(13.0);

    m.mark("variant-multilevel");
    m.play(act().fade_in("vs").dur(0.5));
    caption(
        &mut m,
        "scalable: filter full? freeze it, stack a fresh layer on top \u{2014} query newest-first",
    );
    m.wait(13.0);

    m.mark("variant-cuckoo");
    m.play(act().fade_in("vk").dur(0.5));
    caption(
        &mut m,
        "cuckoo: the young challenger \u{2014} deletes, tighter space under 3% error",
    );
    m.wait(13.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §6 WHERE IT LIVES — 6:35–7:40 ===================
    m.section("Where It Lives");
    m.mark("uses");
    m.play(act().fade_in("usehead").dur(0.6));
    m.wait(4.0);

    m.mark("use-lsm");
    m.play(act().fade_in("use1").dur(0.5));
    caption(&mut m, "every SSTable carries a filter of its keys \u{2014} \u{201C}definitely not\u{201D} skips the file");
    m.wait(16.0);

    m.mark("use-web");
    m.play(act().fade_in("use2").dur(0.5));
    m.wait(2.0);
    m.play(act().fade_in("use3").dur(0.5));
    caption(&mut m, "three-quarters of web objects are requested exactly once \u{2014} don't cache one-hit wonders");
    m.wait(16.0);

    m.mark("use-misc");
    m.play(act().fade_in("use4").dur(0.5));
    caption(
        &mut m,
        "anywhere the question is \u{201C}can I skip the expensive thing?\u{201D}",
    );
    m.wait(12.0);
    m.clear_all(0.8);
    m.wait(2.0);

    // ================= §7 CLOSE — 7:40–8:00 ============================
    m.section("The Close");
    m.mark("outro");
    m.play(act().fade_in("outro1").dur(0.8));
    m.wait(2.5);
    m.play(par![
        act().fade_in("outro2").dur(0.8),
        act().pulse("outro2").dur(0.8),
    ]);
    caption(&mut m, "");
    m.wait(6.0);

    m.mark("signoff");
    m.play(act().fade_in("next").dur(0.6));
    m.wait(8.0);

    broadsheet::run(m);
}
