//! The tech demo: every engine feature in one beat-synced reel.
//!
//! All timing is derived from `assets/beats.json` (a beat map extracted from
//! the soundtrack), so every cut, pulse and camera move lands on the music.
//! The engine itself never touches audio — the beat map is data, the timeline
//! stays a pure function of t, and the soundtrack is muxed in afterwards:
//!
//! ```sh
//! cargo run --release --example showcase -- --record renders/showcase --grain
//! ffmpeg -i renders/showcase/out.mp4 -i assets/music/track.mp3 \
//!        -c:v copy -c:a aac -shortest renders/showcase/final.mp4
//! ```

use broadsheet::prelude::*;

/// Beat map loaded from `assets/beats.json`. `bpm`/`offset` define the grid;
/// `beats` lists every detected beat timestamp for data-driven scheduling.
struct BeatMap {
    bpm: f32,
    offset: f32,
    beats: Vec<f32>,
}

impl BeatMap {
    fn load(path: &str) -> BeatMap {
        let src =
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
        let num = |key: &str| -> f32 {
            let at = src.find(&format!("\"{key}\"")).expect(key) + key.len() + 2;
            src[at..]
                .trim_start_matches([':', ' '])
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect::<String>()
                .parse()
                .expect(key)
        };
        let arr = src.find("\"beats\"").expect("beats");
        let open = arr + src[arr..].find('[').expect("beats [");
        let close = open + src[open..].find(']').expect("beats ]");
        let beats = src[open + 1..close]
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        BeatMap {
            bpm: num("bpm"),
            offset: num("offset"),
            beats,
        }
    }

    fn spb(&self) -> f32 {
        60.0 / self.bpm
    }

    /// Timestamp of (fractional) beat `b` on the grid.
    fn t(&self, b: f32) -> f32 {
        self.offset + b * self.spb()
    }

    /// First whole beat at or after time `t`.
    fn next_beat(&self, t: f32) -> f32 {
        ((t - self.offset) / self.spb()).ceil().max(0.0)
    }
}

const W: f32 = 1280.0;
const H: f32 = 720.0;
const CX: f32 = W / 2.0;
const CY: f32 = H / 2.0;

fn main() {
    let bm = BeatMap::load("assets/beats.json");
    let mut m = Movie::new("The Broadsheet Engine", W as u32, H as u32);

    // ---- data for the binary search scene ------------------------------
    let vals = [
        "2", "5", "8", "13", "21", "29", "34", "41", "47", "55", "60", "73",
    ];
    let n = vals.len();
    let cell = v(56.0, 56.0);
    let gap = 10.0;
    let stride = cell.x + gap;
    let cells_y = 470.0;
    let cell_x = |i: f32| CX - stride * (n as f32 - 1.0) / 2.0 + stride * i;
    // (mid, a[mid] < x, which pointer moves, its new index)
    let iters: [(usize, bool, &str, f32); 4] = [
        (6, true, "lo", 7.0),
        (9, false, "hi", 9.0),
        (8, false, "hi", 8.0),
        (7, false, "hi", 7.0),
    ];

    let ring_c = v(CX, 390.0);
    let ring_pos = layout::ring(8, ring_c, 175.0);

    let uf_parents = [None, Some(0), Some(0), Some(1), None, Some(4)];
    let uf_pos = layout::tree(&uf_parents, v(CX - 60.0, 250.0), 130.0, 105.0);
    let uf_edges: [(usize, usize); 4] = [(1, 0), (2, 0), (3, 1), (5, 4)];

    let eq_n = 12;
    let eq_pos = layout::row(eq_n, 430.0, CX - 380.0, CX + 380.0);

    // ---- scene: everything declared at t = 0 ----------------------------
    {
        let mut s = m.scene();

        // scene A (binary search) sits behind the intro cover, visible at t0
        s.code_block(
            "code",
            v(120.0, 170.0),
            &[
                "fn bsearch(a: &[i32], x: i32) -> usize {",
                "    let (mut lo, mut hi) = (0, a.len());",
                "    while lo < hi {",
                "        let mid = (lo + hi) / 2;",
                "        if a[mid] < x { lo = mid + 1 }",
                "        else { hi = mid }",
                "    }",
                "    lo",
                "}",
            ],
            19.0,
        );
        s.cells("a", n, v(CX, cells_y), cell, gap, Some(&vals));
        s.text("lo", v(cell_x(0.0), cells_y + 78.0), "lo")
            .mono_bold()
            .color(BLUE)
            .hidden();
        s.text("hi", v(cell_x(12.0), cells_y + 78.0), "hi")
            .mono_bold()
            .color(BLUE)
            .hidden();
        s.text("mid", v(cell_x(6.0), cells_y - 62.0), "mid")
            .mono_bold()
            .color(ACCENT)
            .hidden();
        s.text("target", v(1060.0, 190.0), "x = 41")
            .mono_bold()
            .size(30.0)
            .hidden();

        // shared caption, restyled per scene with set_text
        s.text("cap", v(CX, 655.0), "")
            .size(20.0)
            .color(FADED)
            .wrap(1000.0)
            .hidden();

        // sticky HUD badge: pinned to screen through every camera move
        s.rect("badge", v(1140.0, 96.0), 220.0, 34.0)
            .color(PAPER_SHADE)
            .stroke(1.5)
            .sticky()
            .z(70)
            .hidden()
            .label("BEAT-SYNCED");

        // scene B: consistent-hash ring
        for (i, p) in ring_pos.iter().enumerate() {
            s.circle(&format!("r{i}"), *p, 30.0)
                .stroke(2.5)
                .tag("ring")
                .hidden()
                .label(&i.to_string());
        }
        for i in 0..8 {
            s.curve(&format!("re{i}"), ring_pos[i], ring_pos[(i + 1) % 8], 34.0)
                .color(FADED)
                .stroke(1.8)
                .tag("redge")
                .z(-1)
                .untraced();
        }
        s.text("key", ring_c, "\"user:42\"")
            .mono_bold()
            .color(ACCENT)
            .tag("ring")
            .hidden();
        s.arrow("hash", ring_c, ring_c)
            .color(ACCENT)
            .stroke(2.5)
            .tag("ring")
            .hidden();

        // scene C: union-find forest
        for (i, p) in uf_pos.iter().enumerate() {
            s.circle(&format!("u{i}"), *p, 28.0)
                .stroke(2.5)
                .tag("uf")
                .hidden()
                .label(&i.to_string());
        }
        // arrows start on the child's rim and will grow to the parent's rim
        for (c, p) in uf_edges {
            let dir = (uf_pos[p] - uf_pos[c]).normalize();
            s.arrow(
                &format!("ue{c}"),
                uf_pos[c] + dir * 28.0,
                uf_pos[c] + dir * 28.0,
            )
            .color(BLUE)
            .stroke(2.2)
            .tag("uf")
            .z(-1)
            .hidden();
        }
        let dir40 = (uf_pos[0] - uf_pos[4]).normalize();
        s.arrow("ue4", uf_pos[4] + dir40 * 28.0, uf_pos[4] + dir40 * 28.0)
            .color(ACCENT)
            .stroke(2.2)
            .tag("uf")
            .z(-1)
            .hidden();

        // scene D: the beat machine
        for (i, p) in eq_pos.iter().enumerate() {
            s.rect(&format!("eq{i}"), *p, 30.0, 90.0)
                .color(PAPER_SHADE)
                .stroke(2.0)
                .tag("eq")
                .hidden();
        }
        s.text("count", v(CX, 260.0), "")
            .serif()
            .size(72.0)
            .hidden();

        // intro cover: a front page over the whole stage
        s.rect("cover", v(CX, CY), W, H).color(PAPER).filled().z(80);
        s.text(
            "kicker",
            v(CX, 200.0),
            "VOL. 1 — TECH DEMO — 60 FPS — DETERMINISTIC",
        )
        .size(18.0)
        .color(FADED)
        .z(85)
        .hidden();
        s.text("title", v(CX, 300.0), "BROADSHEET")
            .serif()
            .size(110.0)
            .z(85)
            .untraced();
        s.line("rule", v(CX - 260.0, 372.0), v(CX + 260.0, 372.0))
            .color(ACCENT)
            .stroke(4.0)
            .z(85)
            .untraced();
        s.text(
            "sub",
            v(CX, 428.0),
            "a newspaper-styled animation engine, written in Rust",
        )
        .size(24.0)
        .z(85)
        .untraced();
        s.text("stamp", v(990.0, 520.0), "OPEN SOURCE")
            .mono_bold()
            .size(26.0)
            .color(ACCENT)
            .rot(-8.0)
            .z(85)
            .hidden();

        // outro
        s.text("install", v(CX, 330.0), "$ cargo add broadsheet")
            .mono_bold()
            .size(40.0)
            .z(85)
            .untraced();
        s.text(
            "outro.cap",
            v(CX, 410.0),
            "every move on this page was scheduled from beats.json",
        )
        .size(20.0)
        .color(FADED)
        .z(85)
        .hidden();
        s.text(
            "credit",
            v(CX, 620.0),
            "music: \"Funk Game Loop\" — Kevin MacLeod (incompetech.com), CC BY 4.0",
        )
        .size(16.0)
        .color(FADED)
        .z(85)
        .hidden();
    }

    // =====================================================================
    // timeline — beat indices, not seconds
    // =====================================================================
    let b = |k: f32| bm.t(k);

    // ---- intro: beats 0..16 ---------------------------------------------
    m.mark("intro");
    m.at(b(0.0), act().fade_in("kicker").dur(0.4));
    m.at(b(1.0), act().type_in("title").dur(bm.spb() * 3.0));
    m.at(b(4.0), act().trace_in("rule").dur(bm.spb()).ease(OutCubic));
    m.at(b(5.0), act().type_in("sub").dur(bm.spb() * 3.0));
    for k in [4.0, 5.0, 6.0, 7.0] {
        m.at(b(k), act().pulse("title").dur(bm.spb() * 0.9));
    }
    m.at(
        b(9.0),
        par![
            act().fade_in("stamp").dur(0.25),
            act().scale_to("stamp", 1.0).dur(0.0),
        ],
    );
    m.at(b(9.0), act().pulse("stamp").dur(bm.spb()));
    m.at(b(12.0), act().fade_in("badge").dur(0.4));
    // page turn: cover + intro type fades, scene A is revealed underneath
    for id in ["cover", "kicker", "title", "rule", "sub", "stamp"] {
        m.at(b(14.0), act().fade_out(id).dur(bm.spb() * 2.0));
    }

    // ---- section card: arrays -------------------------------------------
    m.wait(b(16.0) - m.now());
    m.section("Arrays, Code & Camera");
    let s1 = bm.next_beat(m.now());
    m.mark("arrays");

    m.at(
        b(s1),
        act().set_text(
            "cap",
            "cells() lays out the array; a code block is one entity per line",
        ),
    );
    m.at(b(s1), act().fade_in("target").dur(0.4));
    m.at(
        b(s1 + 1.0),
        stagger(
            bm.spb() / 8.0,
            (0..n)
                .map(|i| act().pulse(&format!("a{i}")).dur(0.4).into())
                .collect(),
        ),
    );
    m.at(
        b(s1 + 2.0),
        par![
            act().fade_in("lo").dur(0.3),
            act().fade_in("hi").dur(0.3),
            act().fade_in("mid").dur(0.3),
        ],
    );

    // four beats per iteration: mid punch-in -> compare -> pointer move
    for (it, (mid, less, ptr, to)) in iters.iter().enumerate() {
        let t0 = s1 + 3.0 + it as f32 * 4.0;
        let mid_x = cell_x(*mid as f32);
        m.at(
            b(t0),
            par![
                act()
                    .move_to("mid", v(mid_x, cells_y - 62.0))
                    .dur(bm.spb())
                    .ease(OutCubic),
                act()
                    .highlight(&format!("code.line{}", 3), ACCENT)
                    .dur(bm.spb() * 2.0),
                act()
                    .cam_to(v(mid_x, cells_y - 30.0))
                    .dur(bm.spb() * 2.0)
                    .ease(InOutCubic),
                act().cam_zoom(1.45).dur(bm.spb() * 2.0).ease(InOutCubic),
            ],
        );
        m.at(b(t0 + 1.0), flash(&format!("a{mid}")).dur(bm.spb() * 1.5));
        let cmp_line = if *less { 4 } else { 5 };
        m.at(
            b(t0 + 2.0),
            act()
                .highlight(&format!("code.line{cmp_line}"), BLUE)
                .dur(bm.spb() * 1.5),
        );
        if *less {
            // too small: the cell shakes its head
            m.at(b(t0 + 2.0), act().shake(&format!("a{mid}")).dur(bm.spb()));
        }
        m.at(
            b(t0 + 3.0),
            par![
                act()
                    .move_to(ptr, v(cell_x(*to), cells_y + 78.0))
                    .dur(bm.spb())
                    .ease(OutBack),
                act().pulse(ptr).dur(bm.spb()),
            ],
        );
    }

    // found: lo == hi == 7
    let fb = s1 + 3.0 + 16.0;
    m.at(
        b(fb),
        par![
            act().color_to("a7", ACCENT).dur(bm.spb()),
            act().color_to("a7.label", PAPER).dur(bm.spb()),
            act().pulse("a7").dur(bm.spb() * 2.0),
            act()
                .cam_to(v(cell_x(7.0), cells_y))
                .dur(bm.spb() * 2.0)
                .ease(InOutCubic),
            act().cam_zoom(1.8).dur(bm.spb() * 2.0).ease(InOutCubic),
            act().set_text(
                "cap",
                "found at index 7 — the camera is just another animatable entity"
            ),
        ],
    );
    m.at(
        b(fb + 3.0),
        par![
            act().cam_to(v(CX, CY)).dur(bm.spb() * 2.0).ease(InOutCubic),
            act().cam_zoom(1.0).dur(bm.spb() * 2.0).ease(InOutCubic),
        ],
    );

    // clear scene A with tags + all()
    let clear_a = fb + 5.0;
    let mut a_ids = m.tagged("a");
    a_ids.extend(m.tagged("code"));
    a_ids.extend(["lo", "hi", "mid", "target", "cap"].map(String::from));
    m.at(b(clear_a), all(&a_ids, |id| act().fade_out(id).dur(0.4)));

    // ---- section card: ring ----------------------------------------------
    m.wait(b(clear_a + 1.0) - m.now());
    m.section("Rings, Curves & Tags");
    let s2 = bm.next_beat(m.now());
    m.mark("ring");

    m.at(
        b(s2),
        act().set_text("cap", "layout::ring + curved edges tracing in on the beat"),
    );
    m.at(
        b(s2),
        stagger(
            bm.spb() / 2.0,
            (0..8)
                .map(|i| act().fade_in(&format!("r{i}")).dur(0.35).into())
                .collect(),
        ),
    );
    m.at(
        b(s2 + 4.0),
        stagger(
            bm.spb() / 2.0,
            (0..8)
                .map(|i| act().trace_in(&format!("re{i}")).dur(bm.spb()).into())
                .collect(),
        ),
    );

    // hash a key onto the ring: arrow grows from the centre to node 3
    m.at(
        b(s2 + 8.0),
        par![
            act().fade_in("key").dur(0.4),
            act().set_text(
                "cap",
                "hash(\"user:42\") = 3 — the arrow is drawn with grow_to()"
            ),
        ],
    );
    m.at(
        b(s2 + 9.0),
        par![
            act().fade_in("hash").dur(0.2),
            act()
                .grow_to(
                    "hash",
                    ring_pos[3] + (ring_c - ring_pos[3]).normalize() * 34.0
                )
                .dur(bm.spb() * 2.0)
                .ease(OutCubic),
        ],
    );
    m.at(
        b(s2 + 11.0),
        par![
            act().color_to("r3", ACCENT).dur(bm.spb()),
            act().color_to("r3.label", PAPER).dur(bm.spb()),
            act().pulse("r3").dur(bm.spb()),
        ],
    );

    // orbit the ring: camera hops node to node on the beat
    m.at(
        b(s2 + 12.0),
        act().set_text(
            "cap",
            "camera pans orbit the ring — the badge stays pinned (sticky)",
        ),
    );
    for (hop, node) in [3usize, 5, 7, 1].iter().enumerate() {
        m.at(
            b(s2 + 12.0 + hop as f32 * 2.0),
            par![
                act()
                    .cam_to(ring_pos[*node])
                    .dur(bm.spb() * 2.0)
                    .ease(InOutCubic),
                act().cam_zoom(1.6).dur(bm.spb() * 2.0).ease(InOutCubic),
            ],
        );
    }
    m.at(
        b(s2 + 20.0),
        par![
            act().cam_to(v(CX, CY)).dur(bm.spb() * 2.0).ease(InOutCubic),
            act().cam_zoom(1.0).dur(bm.spb() * 2.0).ease(InOutCubic),
        ],
    );

    let clear_b = s2 + 22.0;
    let mut ring_ids = m.tagged("ring");
    ring_ids.push("cap".into());
    m.at(
        b(clear_b),
        all(&m.tagged("redge"), |id| act().trace_out(id).dur(0.4)),
    );
    m.at(b(clear_b), all(&ring_ids, |id| act().fade_out(id).dur(0.4)));

    // ---- section card: union-find ----------------------------------------
    m.wait(b(clear_b + 1.0) - m.now());
    m.section("Trees & Retargets");
    let s3 = bm.next_beat(m.now());
    m.mark("union");

    m.at(
        b(s3),
        act().set_text(
            "cap",
            "layout::tree positions a forest; arrows grow along parent links",
        ),
    );
    m.at(
        b(s3),
        stagger(
            bm.spb() / 2.0,
            (0..6)
                .map(|i| act().fade_in(&format!("u{i}")).dur(0.35).into())
                .collect(),
        ),
    );
    m.at(
        b(s3 + 3.0),
        stagger(
            bm.spb() / 2.0,
            uf_edges
                .iter()
                .map(|(c, p)| {
                    let dir = (uf_pos[*p] - uf_pos[*c]).normalize();
                    par![
                        act().fade_in(&format!("ue{c}")).dur(0.2),
                        act()
                            .grow_to(&format!("ue{c}"), uf_pos[*p] - dir * 32.0)
                            .dur(bm.spb())
                            .ease(OutCubic),
                    ]
                    .into()
                })
                .collect(),
        ),
    );

    // union(4, 0): a new parent pointer grows, then 3 gets path-compressed
    m.at(
        b(s3 + 6.0),
        act().set_text(
            "cap",
            "union(4, 0) — then retarget() re-points 3 straight at the root",
        ),
    );
    let dir40 = (uf_pos[0] - uf_pos[4]).normalize();
    m.at(
        b(s3 + 7.0),
        par![
            act().fade_in("ue4").dur(0.2),
            act()
                .grow_to("ue4", uf_pos[0] - dir40 * 32.0)
                .dur(bm.spb() * 2.0)
                .ease(InOutCubic),
            act().pulse("u0").dur(bm.spb() * 2.0),
        ],
    );
    let dir30 = (uf_pos[0] - uf_pos[3]).normalize();
    m.at(
        b(s3 + 10.0),
        par![
            act()
                .retarget("ue3", uf_pos[0] - dir30 * 32.0)
                .dur(bm.spb() * 2.0)
                .ease(InOutCubic),
            act().color_to("ue3", ACCENT).dur(bm.spb() * 2.0),
            act().pulse("u3").dur(bm.spb()),
        ],
    );
    m.at(b(s3 + 12.0), flash("u0").dur(bm.spb() * 2.0));

    let clear_c = s3 + 14.0;
    let mut uf_ids = m.tagged("uf");
    uf_ids.push("cap".into());
    m.at(b(clear_c), all(&uf_ids, |id| act().fade_out(id).dur(0.4)));

    // ---- section card: beat machine ---------------------------------------
    m.wait(b(clear_c + 1.0) - m.now());
    m.section("The Beat Machine");
    let s4 = bm.next_beat(m.now());
    m.mark("beats");

    m.at(
        b(s4),
        act().set_text(
            "cap",
            "these hits are read straight from beats.json — data in, choreography out",
        ),
    );
    m.at(
        b(s4),
        stagger(
            bm.spb() / 6.0,
            (0..eq_n)
                .map(|i| act().fade_in(&format!("eq{i}")).dur(0.3).into())
                .collect(),
        ),
    );

    // schedule directly off the detected beat list, not the grid
    let win_a = b(s4 + 2.0);
    let win_b = b(s4 + 18.0);
    let mut k = 0usize;
    for &bt in bm.beats.iter().filter(|&&t| t >= win_a && t < win_b) {
        let hot = [k % eq_n, (k * 5 + 2) % eq_n, (k * 7 + 5) % eq_n];
        for (j, i) in hot.iter().enumerate() {
            let up = 1.35 + 0.25 * j as f32;
            m.at(
                bt,
                seq![
                    act()
                        .scale_to(&format!("eq{i}"), up)
                        .dur(bm.spb() * 0.3)
                        .ease(OutQuad),
                    act()
                        .scale_to(&format!("eq{i}"), 1.0)
                        .dur(bm.spb() * 0.6)
                        .ease(OutCubic),
                ],
            );
        }
        m.at(bt, flash(&format!("eq{}", k % eq_n)).dur(bm.spb() * 0.8));
        if k % 4 == 0 {
            m.at(
                bt,
                act()
                    .set_text("count", &format!("{}", k / 4 + 1))
                    .dur(bm.spb() * 0.8),
            );
        }
        k += 1;
    }

    let clear_d = s4 + 19.0;
    let mut eq_ids = m.tagged("eq");
    eq_ids.extend(["count", "cap"].map(String::from));
    m.at(b(clear_d), all(&eq_ids, |id| act().fade_out(id).dur(0.5)));

    // ---- outro -------------------------------------------------------------
    let s5 = clear_d + 2.0;
    m.wait(b(s5) - m.now());
    m.mark("outro");
    m.at(b(s5), act().type_in("install").dur(bm.spb() * 3.0));
    m.at(b(s5 + 3.0), act().fade_in("outro.cap").dur(0.5));
    m.at(b(s5 + 4.0), act().fade_in("credit").dur(0.5));
    m.at(b(s5 + 5.0), act().pulse("install").dur(bm.spb()));
    for id in ["install", "outro.cap", "credit", "badge"] {
        m.at(b(s5 + 8.0), act().fade_out(id).dur(bm.spb() * 2.0));
    }
    m.wait(b(s5 + 10.0) - m.now());

    broadsheet::run(m);
}
