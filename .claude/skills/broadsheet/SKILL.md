---
name: broadsheet
description: Author, debug, and render broadsheet animations — the Rust 2D engine for algorithm/data-structure explainer videos (movies, themes, widgets, slideshows, recording). Use when writing or editing a movie script (examples/*.rs here or in ledger/animations), rendering video/stills/GIFs, building a slideshow, or wiring a Ledger chapter animation.
---

# broadsheet — authoring animations

Newspaper-styled 2D animation engine on macroquad. One movie = one Rust
example file. Core invariant (never break it): `Timeline::apply(base, t)` is
a pure function of `t` — no state between frames, no wall clock, no unseeded
randomness.

## Movie skeleton

```rust
use broadsheet::prelude::*;
use broadsheet::widgets; // optional

fn main() {
    let mut m = Movie::new("Title", 1280, 720);
    m.set_theme(Theme::midnight()); // BEFORE declaring entities; default = broadsheet()

    // 1. cast — the world at t = 0
    m.scene()
        .circle("A", v(300., 400.), 40.).label("A")
        .text("cap", v(640., 620.), "").size(22.).color(FADED).hidden();

    // 2. beats — cursor advances with each play()
    m.play(act().fade_in("A"));
    m.play(par![ act().move_to("A", v(900., 400.)).dur(0.6).ease(InOutCubic),
                 act().highlight("A", ACCENT) ]);
    m.wait(1.0);              // narration beat
    m.section("Part Two");    // serif section card + player jump marker
    m.mark("beat-name");      // exported to markers.json (adds no time)
    m.slide("slide-name");    // slideshow boundary (adds no time)

    broadsheet::run(m);
}
```

## API quick reference

Shapes (SceneBuilder, chainable — modifiers apply to last shape):
`circle rect line arrow curve curve_arrow polygon text cells code_block`
Modifiers: `.color .role(Role) .outlined .filled .stroke .outline_color
.size .serif .mono_bold .z .hidden .opacity .rot .wrap(px) .left .untraced
.tag .sticky .follow(id, offset) .label(text)`
— `.label()` creates `"{id}.label"` riding the parent; followers inherit
parent opacity, so hide/show the parent only.

Verbs (`act().<verb>(id, ..).dur(s).ease(E)`):
`move_to move_by fade_in fade_out color_to highlight(auto-reverts)
scale_to pulse shake grow_to/retarget(line/arrow endpoint) set_text
trace_in trace_out type_in cam_to cam_zoom`

Combinators: `seq![..]  par![..]  stagger![delay; ..]  wait(s)
all(&m.tagged("tag"), |id| act()...)` — `m.at(t, clip)` places at absolute
time without moving the cursor.

Themes/roles: `Theme::{broadsheet, midnight, plain}`, `m.set_theme(t)`,
`m.role(Role::Found)` → Color for verbs, `.role(Role::Active)` on builder.
Roles: Active Visited Skipped Found Stale Deleted Maybe Absent.

Widgets (`widgets::`): `array bit_array hash_table linked_list tree graph
hash_ring lsm_levels skip_list` — one call declares the structure, returns a
handle with id accessors (`arr.cell(3)`, `list.next(1)`, `bst.edge(i)`).
Everything is tagged with the widget id: `all(&m.tagged("arr"), ...)` fades
the whole thing. Widgets start VISIBLE.

Layouts (`layout::`): `row grid ring tree levels blocks graph(seeded spring)
rng(seed)` — pure Vec2 helpers.

## Gotchas (each has cost real debugging time)

- Duplicate entity id, or animating an unknown id → **panic at startup**.
  Always `cargo run` after writing.
- `move_to` on a Line/Arrow moves only the tail — it DEFORMS. Rigid pointer
  = small `polygon` triangle (polygons move rigidly), or animate `move_to`
  + `grow_to` together.
- `cells()` and widgets cannot start hidden. Card-by-card reveals: declare
  rects manually in a loop with `.hidden()` (see ledger ring_buffer.rs), or
  hide a widget at t=0 with a zero-duration fade:
  `m.at(0.0, all(&m.tagged("w"), |id| act().fade_out(id).dur(0.0)))`.
- `highlight` reverts automatically; use `color_to` for a persistent state
  change (e.g. Role::Found).
- `clear_all(dur)` fades only entities declared BEFORE the call; skips
  followers and `__`-prefixed ids.
- `--still` output is vertically FLIPPED (known bug; mp4/GIF are correct).
  Fix for publishing: `ffmpeg -vf vflip`.
- Window must stay open during `--record`.
- Text jitter during zooms is already handled (raster at fixed size) — do
  not "fix" it.
- Shell on this machine: prefix commands with `command ` (lean-ctx aliases).

## Run / render

```sh
cargo run --example NAME                                   # live preview
cargo run --example NAME -- --slideshow                    # deck mode (needs m.slide)
cargo run --example NAME -- --record out/NAME --fps 60 --grain   # → out.mp4 + markers.json
cargo run --example NAME -- --still 12.5                   # one PNG (flipped)
cargo run --example NAME -- --record out --gif --scale 0.5 --fps 12  # small GIF
cargo run --example NAME -- --record /tmp/chk --frames 2 --png       # fast markers.json dump
```
Live keys: Space pause · ←/→ step · ,/. ±1s · 1-9 sections · F fullscreen ·
R restart · drag bottom bar scrub. Slideshow mode: Space/→ next slide,
← back.

Verification loop for a new movie: build → 2-frame record → read
markers.json (marks/slides present, right order) → 2–3 stills at key times
→ eyeball (they're flipped) → full record only when stills look right.

## Ledger chapters (the ~/personal/ledger consumer)

Scroll-driven web chapters pair one example with one .astro file. Rules:
every prose card kicker `Foo Bar` needs `m.mark("foo-bar.start")` /
`"foo-bar.end"` bracketing its animation; captions use the wasm-empty
helper so web/video durations match; step order = mark order. Full
conventions + verification: `~/personal/ledger/HANDOFF.md`. Build wasm with
`./build-wasm.sh <example>`.

## House style

Newspaper look: ink outlines, paper fills, ACCENT sparingly (highlights,
warnings), BLUE for "the other thing", FADED for annotations/indices.
Serif = headlines/stamps only; mono = data/captions. Rotated serif "stamps"
(`.serif().rot(-3.0)`) mark payoffs (FOUND., no rotations.). Reveal
structure with `stagger!`, trace edges with `.untraced()` + `trace_in`.
Captions: lowercase, concrete, one line. Keep 7–10s of hold (`m.wait`)
after each beat for narration.

After editing engine source (not examples): run
`cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`,
then rebuild the graphify graph per repo CLAUDE.md.
