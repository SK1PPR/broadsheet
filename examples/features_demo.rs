//! Tour of the engine's toolkit: cells, code blocks, typewriter text,
//! curved arrows with draw-on, ring layout, tags, stagger, camera moves.
//!
//! Run live:    cargo run --example features_demo
//! Record:      cargo run --example features_demo -- --record out/demo

use broadsheet::prelude::*;

fn main() {
    let mut m = Movie::new("The Toolkit", 1280, 720);

    let ring_pos = layout::ring(6, v(880.0, 400.0), 150.0);

    {
        let mut s = m.scene();

        s.text(
            "type",
            v(120.0, 150.0),
            "every tool in one demo — typed in, of course",
        )
        .left()
        .size(24.0)
        .untraced();

        s.code_block(
            "code",
            v(120.0, 220.0),
            &[
                "fn find(x) {",
                "    while p[x] != x {",
                "        x = p[x];",
                "    }",
                "}",
            ],
            20.0,
        );

        s.cells("bit", 8, v(320.0, 480.0), v(52.0, 52.0), 8.0, None);

        // consistent-hash ring: nodes + curved arrows between neighbours
        for (i, p) in ring_pos.iter().enumerate() {
            s.circle(&format!("r{i}"), *p, 26.0)
                .stroke(2.5)
                .tag("ring")
                .hidden()
                .label(&i.to_string());
        }
        for i in 0..6 {
            let a = ring_pos[i];
            let b = ring_pos[(i + 1) % 6];
            s.curve_arrow(&format!("c{i}"), a, b, 40.0)
                .color(BLUE)
                .stroke(2.0)
                .tag("ringedges")
                .untraced();
        }

        s.text("caption", v(640.0, 660.0), "")
            .size(20.0)
            .color(FADED)
            .wrap(1060.0)
            .hidden();
    }

    // typewriter + code walkthrough
    m.play(act().type_in("type").dur(2.0));
    m.mark("code-walk");
    m.play(act().set_text(
        "caption",
        "code blocks: one entity per line, highlight any of them",
    ));
    m.play(seq![
        act().highlight("code.line1", ACCENT).dur(0.8),
        act().highlight("code.line2", ACCENT).dur(0.8),
    ]);
    m.wait(0.5);

    // cells + stagger
    m.play(act().set_text(
        "caption",
        "cells() builds bit arrays in one call; stagger! cascades",
    ));
    m.play(stagger![
        0.08;
        act().color_to("bit2", ACCENT).dur(0.3),
        act().color_to("bit3", ACCENT).dur(0.3),
        act().color_to("bit5", ACCENT).dur(0.3),
    ]);
    m.wait(0.8);
    let mut intro_ids = vec!["type".to_string()];
    intro_ids.extend((0..5).map(|i| format!("code.line{i}")));
    intro_ids.extend((0..8).map(|i| format!("bit{i}")));
    m.play(all(&intro_ids, |id| act().fade_out(id).dur(0.35)));

    // ring reveal: staggered nodes, then curved arrows trace in
    m.mark("ring");
    m.play(act().set_text("caption", "ring layout + curved arrows tracing on"));
    let nodes: Vec<Clip> = (0..6)
        .map(|i| act().fade_in(&format!("r{i}")).dur(0.3).into())
        .collect();
    m.play(stagger(0.07, nodes));
    let edges: Vec<Clip> = (0..6)
        .map(|i| act().trace_in(&format!("c{i}")).dur(0.5).into())
        .collect();
    m.play(stagger(0.15, edges));
    m.wait(1.0);

    // camera: punch in on the ring, then back out
    m.mark("camera");
    m.play(act().set_text(
        "caption",
        "camera pan + zoom are just tracks on the __cam entity",
    ));
    m.play(par![
        act().cam_to(v(880.0, 350.0)).dur(1.2).ease(InOutCubic),
        act().cam_zoom(1.55).dur(1.2).ease(InOutCubic),
    ]);
    m.wait(1.2);
    m.play(par![
        act().cam_to(v(640.0, 360.0)).dur(1.0).ease(InOutCubic),
        act().cam_zoom(1.0).dur(1.0).ease(InOutCubic),
    ]);

    // tags: clear the ring in one line
    m.play(act().set_text(
        "caption",
        "tags: fade a whole group without naming its members",
    ));
    m.play(all(&m.tagged("ringedges"), |id| {
        act().trace_out(id).dur(0.4)
    }));
    m.play(all(&m.tagged("ring"), |id| act().fade_out(id).dur(0.4)));
    m.wait(2.0);

    broadsheet::run(m);
}
