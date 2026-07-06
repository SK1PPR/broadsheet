# broadsheet

A 2D animation engine for algorithm & data-structure explainer videos, in
Rust on [macroquad]. Newspaper-styled, deterministic, code-driven.

Every render looks like a page from the same broadsheet: off-white paper,
ink strokes, one spot color, serif headlines, mono data. That's the channel's
visual identity — defined once in `src/style.rs`.

[macroquad]: https://github.com/not-fl3/macroquad

## Run the examples

```sh
cargo run --example bloom_filter    # live preview window
cargo run --example union_find
```

Live transport controls (for lining narration up with beats):

| key | action |
|---|---|
| `Space` | pause / play |
| `←` `→` | step one frame |
| `,` `.` | jump ±1 s |
| `1`–`9` | jump to section markers |
| `F` | toggle fullscreen (fit-to-screen, letterboxed) |
| `R` | restart |
| drag bottom bar | scrub |

The HUD shows exact `t` and frame number; it is never present in recordings.

## Record a video

```sh
cargo run --example bloom_filter -- --record out/bloom --fps 60
```

Renders at a fixed timestep (`t = frame / fps`, wall clock ignored → output
is deterministic), writes `out/bloom/frame_00000.png …`, then runs ffmpeg if
installed — otherwise it prints the exact stitch command:

```sh
ffmpeg -framerate 60 -i out/bloom/frame_%05d.png -c:v libx264 -pix_fmt yuv420p bloom.mp4
```

Recording supersamples at `--scale 1.5` by default, so the 1280×720 logical
canvas comes out as true 1920×1080 with fonts rasterized at full resolution
(pass `--scale 2` for 1440p). Everything is drawn with 4× MSAA.

Tip: `--fps 2` sparsely samples the whole movie in a few dozen frames —
a fast visual proof-read of a full video. `--frames N` caps the frame count.

## Writing a movie

```rust
use broadsheet::prelude::*;

fn main() {
    let mut m = Movie::new("Skip Lists", 1280, 720);

    // 1. declare the cast (state at t = 0)
    m.scene()
        .circle("A", v(300., 400.), 40.).label("A")
        .circle("B", v(900., 400.), 40.).label("B").hidden()
        .arrow("e", v(340., 400.), v(340., 400.)).hidden()
        .text("cap", v(640., 620.), "").size(22.).color(FADED).hidden();

    // 2. script the beats (cursor advances with each play)
    m.play(act().set_text("cap", "two nodes, one pointer"));
    m.play(act().fade_in("B").dur(0.4));
    m.play(par![                       // simultaneous
        act().fade_in("e").dur(0.15),
        act().grow_to("e", v(860., 400.)).dur(0.5).ease(InOutCubic),
    ]);
    m.play(seq![                       // sequential
        act().highlight("B", ACCENT),
        act().pulse("B"),
        wait(0.5),                     // narration beat
    ]);
    m.section("Level 2");              // newspaper section card + jump marker
    m.play(act().move_to("A", v(300., 250.)).dur(0.6).ease(OutBack));

    broadsheet::run(m);
}
```

Verbs: `move_to` `move_by` `fade_in` `fade_out` `color_to` `highlight`
(auto-reverts) `scale_to` `pulse` `shake` `grow_to`/`retarget` (line & arrow
endpoints — draws edges, rewires pointers) `set_text` (crossfade).
Tune any act with `.dur(secs)` and `.ease(...)`.

Scene niceties: `.label("A")` puts a mono label riding on a shape
(addressable as `"A.label"`); `.follow(id, offset)` pins any entity to
another; `.wrap(px)` word-wraps long text (captions) into centred lines;
`.hidden()` starts invisible for a later `fade_in`; `m.wait(s)`
leaves silence; `m.at(t, clip)` places a clip at an absolute timestamp;
`m.now()` tells you the cursor time for narration notes.

Palette: `INK`, `PAPER`, `ACCENT` (newsprint red), `BLUE`, `FADED`,
`PAPER_SHADE`.

## Extending it

See [ARCHITECTURE.md](ARCHITECTURE.md) — module map, the statelessness
invariant, and step-by-step recipes for adding a primitive or a verb
(the two things a weekly video occasionally needs).
