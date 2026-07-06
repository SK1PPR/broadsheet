//! Bloom filter: a 16-bit array, k = 3 hash functions, two inserts and a
//! false-positive lookup.
//!
//! Run live:    cargo run --example bloom_filter
//! Record:      cargo run --example bloom_filter -- --record out/bloom --fps 60

use broadsheet::prelude::*;

const M: usize = 16;
const BIT_Y: f32 = 420.0;
const WORD_POS: Vec2 = Vec2::new(640.0, 200.0);
const ARROW_TAIL: Vec2 = Vec2::new(640.0, 240.0);

fn bit_x(i: usize) -> f32 {
    145.0 + 66.0 * i as f32
}

fn bit_id(i: usize) -> String {
    format!("bit{i}")
}

fn label_id(i: usize) -> String {
    format!("bit{i}.label")
}

/// Animate inserting/looking-up a word: retarget the k arrows to its hash
/// positions one by one. `set_bits` also flips the bits to 1 (insert);
/// otherwise the bits just flash (lookup).
fn probe(m: &mut Movie, bits: [usize; 3], set_bits: bool, flash_color: Color) {
    for (h, &b) in bits.iter().enumerate() {
        let hid = format!("h{h}");
        let target = v(bit_x(b), BIT_Y - 38.0);
        m.play(par![
            act().fade_in(&hid).dur(0.15),
            act().grow_to(&hid, target).dur(0.45).ease(InOutCubic),
        ]);
        if set_bits {
            m.play(par![
                act().color_to(&bit_id(b), ACCENT).dur(0.25),
                act().pulse(&bit_id(b)).dur(0.5),
                act().set_text(&label_id(b), "1").dur(0.3),
                act().color_to(&label_id(b), PAPER).dur(0.25),
            ]);
        } else {
            m.play(par![
                act().highlight(&bit_id(b), flash_color).dur(0.7),
                act().pulse(&bit_id(b)).dur(0.5),
            ]);
        }
        m.wait(0.2);
    }
}

/// Fade the arrows out and silently park them back at the tail position.
fn reset_arrows(m: &mut Movie) {
    m.play(par![
        act().fade_out("h0").dur(0.3),
        act().fade_out("h1").dur(0.3),
        act().fade_out("h2").dur(0.3),
    ]);
    m.play(par![
        act().retarget("h0", ARROW_TAIL).dur(0.01),
        act().retarget("h1", ARROW_TAIL).dur(0.01),
        act().retarget("h2", ARROW_TAIL).dur(0.01),
    ]);
}

fn main() {
    let mut m = Movie::new("The Bloom Filter", 1280, 720);

    // ---- cast ----------------------------------------------------------
    {
        let mut s = m.scene();
        for i in 0..M {
            s.rect(&bit_id(i), v(bit_x(i), BIT_Y), 56.0, 56.0)
                .color(PAPER_SHADE)
                .outline_color(INK)
                .stroke(2.0)
                .hidden()
                .label("0");
            s.text(&format!("idx{i}"), v(0.0, 0.0), &i.to_string())
                .size(14.0)
                .color(FADED)
                .follow(&bit_id(i), v(0.0, 48.0));
        }
        s.text("word", WORD_POS, "\"cat\"")
            .mono_bold()
            .size(44.0)
            .hidden();
        for h in 0..3 {
            s.arrow(&format!("h{h}"), ARROW_TAIL, ARROW_TAIL)
                .color(INK)
                .stroke(2.0)
                .hidden();
        }
        s.text("caption", v(640.0, 620.0), "")
            .size(22.0)
            .color(FADED)
            .wrap(1080.0)
            .hidden();
        s.text("stamp", v(640.0, 330.0), "FALSE POSITIVE")
            .serif()
            .size(64.0)
            .color(ACCENT)
            .rot(-7.0)
            .z(50)
            .hidden();
    }

    // deterministic "hashes" for the demo words
    let cat = [2usize, 7, 11];
    let dog = [4usize, 7, 14];
    let owl = [2usize, 4, 14]; // every bit already set by cat/dog

    // ---- intro: the bit array ------------------------------------------
    let mut cascade = Vec::new();
    for i in 0..M {
        cascade.push(Clip::from(act().fade_in(&bit_id(i)).dur(0.25)).shift(0.04 * i as f32));
    }
    m.play(Clip::par(cascade));
    m.play(act().set_text("caption", "m = 16 bits · k = 3 hash functions"));
    m.wait(1.0);

    // ---- insert "cat" ----------------------------------------------------
    m.section("Insert");
    m.play(act().fade_in("word").dur(0.4));
    m.play(act().set_text("caption", "h1(cat)=2   h2(cat)=7   h3(cat)=11"));
    probe(&mut m, cat, true, ACCENT);
    m.wait(0.6);
    reset_arrows(&mut m);

    // ---- insert "dog" ----------------------------------------------------
    m.play(act().set_text("word", "\"dog\"").dur(0.5));
    m.play(act().set_text("caption", "h1(dog)=4   h2(dog)=7   h3(dog)=14"));
    probe(&mut m, dog, true, ACCENT);
    m.wait(0.6);
    reset_arrows(&mut m);

    // ---- lookup "owl": the false positive --------------------------------
    m.section("Lookup");
    m.play(par![
        act().set_text("word", "\"owl\"").dur(0.5),
        act().color_to("word", BLUE).dur(0.5),
        act().color_to("h0", BLUE).dur(0.3),
        act().color_to("h1", BLUE).dur(0.3),
        act().color_to("h2", BLUE).dur(0.3),
    ]);
    m.play(act().set_text("caption", "\"owl\" was never inserted. check its bits:"));
    m.wait(0.4);
    probe(&mut m, owl, false, BLUE);
    m.play(act().set_text(
        "caption",
        "all k bits are 1 → the filter says \"probably present\"",
    ));
    m.wait(1.0);

    m.play(par![
        act().fade_in("stamp").dur(0.3),
        act().pulse("stamp").dur(0.6),
        act().shake("word").dur(0.5),
    ]);
    m.play(act().set_text(
        "caption",
        "bit 2 ← cat, bits 4 & 14 ← dog. never owl. that is the false-positive trade-off.",
    ));
    m.wait(2.5);

    broadsheet::run(m);
}
