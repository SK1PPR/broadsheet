# Roadmap

Features worth adding, roughly ordered by (usefulness for weekly videos) ÷
(implementation effort). Every item must preserve the core invariant:
timeline evaluation stays a pure function of `t`.

## Near term — unblock better videos immediately

- [ ] **Stagger combinator** — `stagger(clips, delay)` beside `seq!`/`par!`;
      the fade-in cascades in the examples hand-roll this today.
- [ ] **Groups / tags** — tag entities (`.tag("bits")`), then
      `act().fade_out_tagged("bits")`; kills the fade-everything boilerplate
      in `clear_all` and the examples.
- [ ] **Curved edges** — quadratic-bézier `Arc`/`Curve` shape with the same
      `grow_to` support; straight arrows get cluttered on dense graphs
      (skip lists, consistent-hash rings need this).
- [ ] **Draw-on effect for outlines** — animate a shape's outline being
      traced (0→100% of perimeter); the classic explainer reveal.
- [ ] **Typewriter / per-glyph text** — reveal text character by character;
      also per-char fade for code snippets.
- [ ] **Beat-marker export** — dump section + `m.mark("name")` timestamps to
      JSON alongside a recording, for lining up narration in the editor.
      (Deliberate v1 non-goal, but it's ~30 lines and pays off every week.)

## Layout & data helpers

- [ ] **Auto-layout helpers** — `layout::row/grid/tree/ring(ids, region)`
      returning positions; hand-computing coordinates is most of the work in
      an example today.
- [ ] **Array/table primitive** — declare an n-cell array with labels in one
      call (bit arrays, hash tables, ring buffers recur constantly).
- [ ] **Code-block primitive** — monospace multi-line block with per-line
      highlight verbs, for walking through pseudocode next to the animation.
- [ ] **Camera moves** — pan/zoom as animatable tracks (`cam_to(rect, dur)`);
      needed once scenes outgrow one screen (B-trees, large graphs).

## Rendering & output

- [ ] **Pipe frames straight into ffmpeg** — rawvideo over stdin instead of
      PNG-per-frame; ~10× faster renders, no intermediate gigabytes.
- [ ] **Transparent-background export** — RGBA frames for overlaying renders
      on live-coding footage in the editor.
- [ ] **Still-frame export** — `--still 42.5` renders one frame to PNG;
      thumbnails and blog figures.
- [ ] **GIF/clip export** — `--from/--to` range renders for social posts.
- [ ] **Paper-grain shader pass** — subtle newsprint texture + vignette over
      the final composite; first use of the post-processing seam.

## Bigger swings (later)

- [ ] **3D** — `Vec3` positions + perspective camera behind
      `render::project`; prerequisite for the quaternion/BVH/frustum videos.
- [ ] **Particle system** — a `Shape` variant whose draw arm evaluates a
      particle distribution as a pure function of `t` (keeps determinism).
- [ ] **Shape morphing** — polygon↔polygon interpolation (convex hull steps,
      Voronoi cell changes).
- [ ] **Math typesetting** — prerender LaTeX to SVG/mesh at build time and
      draw as a primitive; formulas are unavoidable eventually.
- [ ] **Golden-frame tests** — render fixed frames of each example in CI and
      diff against checked-in hashes; determinism makes this trivial and it
      guards every refactor.

## Explicitly not planned

- GUI/editor for building animations — code-driven is the point.
- Audio playback/sync inside the engine — post-production's job
  (beat-marker export above is the hand-off).
