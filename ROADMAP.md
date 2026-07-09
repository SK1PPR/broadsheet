# Roadmap

This project has caught up with most of the original roadmap. The checklist
below records what is already shipped and what is intentionally left for later.
Every future item must preserve the core invariant: timeline evaluation stays a
pure function of `t`.

## Implemented

- [x] **Stagger combinator** — `stagger(delay, clips)` and `stagger![delay; …]`
      sit beside `seq!`/`par!` for cascaded reveals.
- [x] **Groups / tags** — entities can be tagged with `.tag("name")`, queried
      with `m.tagged("name")`, and animated as a group with `all(...)`.
- [x] **Curved edges** — `curve` and `curve_arrow` support quadratic Bezier
      paths with the same trace/grow rendering path as straight edges.
- [x] **Draw-on effect for outlines** — `.untraced()` plus
      `act().trace_in(id)` / `trace_out(id)` reveal or erase strokes.
- [x] **Typewriter / per-glyph text** — `act().type_in(id)` reveals text by
      character using the trace progress track.
- [x] **Beat-marker export** — recordings write `markers.json` containing
      sections and `m.mark("name")` timestamps.
- [x] **Auto-layout helpers** — `layout::row`, `grid`, `tree`, and `ring`
      generate common positions for examples.
- [x] **Array/table primitive** — `SceneBuilder::cells(...)` declares labeled
      cell arrays in one call.
- [x] **Code-block primitive** — `SceneBuilder::code_block(...)` creates
      addressable per-line text entities for highlighting.
- [x] **Camera moves** — camera pan/zoom are normal tracks on `__cam` via
      `act().cam_to(...)` and `act().cam_zoom(...)`.
- [x] **Pipe frames straight into ffmpeg** — recording defaults to raw RGBA
      frames over stdin when ffmpeg is available; `--png` keeps the old PNG
      sequence path.
- [x] **Transparent-background export** — `--alpha` renders transparent PNG
      frames without page chrome.
- [x] **Still-frame export** — `--still S` renders one PNG at timestamp `S`.
- [x] **GIF/clip export** — `--from S --to S` records a time range for short
      clips, and `--gif` pipes the result straight into `out.gif`.
- [x] **Paper-grain shader pass** — `--grain` applies a subtle grain/vignette
      post-process in live preview and recordings.

Shipped in 0.3.0:

- [x] **Custom themes** — `Theme` token set (palette, masthead, role colors)
      with `broadsheet`/`midnight`/`plain` presets, set via `m.set_theme(...)`.
- [x] **Semantic roles** — `Role::{Active, Visited, Skipped, Found, Stale,
      Deleted, Maybe, Absent}` mapped to colors per theme; `.role(...)` on the
      scene builder, `m.role(...)` for verbs.
- [x] **Slideshow mode** — `m.slide("name")` boundaries + `--slideshow`:
      pause at each boundary, `Space`/`→` animates to the next, `←` goes back.
      Slides also land in `markers.json`.
- [x] **Data-structure widgets** — `widgets::{array, bit_array, hash_table,
      linked_list, tree, graph, hash_ring, lsm_levels, skip_list}` declare a
      whole structure and return id handles.
- [x] **More layouts** — `layout::levels` (skip lists / LSM / hierarchies),
      `layout::blocks` (pages/chunks), `layout::graph` (deterministic seeded
      spring layout), `layout::rng` (splitmix64, repeatable jitter).

See `examples/features_demo.rs` for a compact tour of the core toolkit and
`examples/slideshow_demo.rs` for themes, roles, widgets, and slides.

## Smaller polish still worth doing

- [ ] **Named output path for stills** — `--still S --out path.png` instead of
      always writing `still_S.png`.
- [ ] **Golden-frame tests for implemented features** — render a few fixed
      frames from `features_demo` and compare hashes to guard the renderer.

- [ ] **State diff transitions** — snapshot declared entity state, mutate,
      emit move/fade/retarget clips automatically (deferred from 0.3.0).
- [ ] **Ghost/previous-state overlays** — faded copy of the pre-transition
      state for before/after comparisons.

## Bigger swings (later)

- [ ] **3D** — `Vec3` positions + perspective camera behind
      `render::project`; prerequisite for the quaternion/BVH/frustum videos.
- [ ] **Particle system** — a `Shape` variant whose draw arm evaluates a
      particle distribution as a pure function of `t` (keeps determinism).
- [ ] **Shape morphing** — polygon to polygon interpolation for convex hull
      steps and Voronoi cell changes.
- [ ] **Math typesetting** — prerender LaTeX to SVG/mesh at build time and draw
      as a primitive; formulas are unavoidable eventually.

## Explicitly not planned

- GUI/editor for building animations — code-driven is the point.
- Audio playback/sync inside the engine — post-production's job; marker export
  is the hand-off.
