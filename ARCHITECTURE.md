# broadsheet — Architecture

A 2D animation engine for algorithm explainer videos. Newspaper-styled,
deterministic, code-driven, built on macroquad.

## The one invariant that matters

**The timeline is stateless.** `Timeline::apply(base_scene, t)` is a pure
function: it computes every animated property at absolute time `t` directly
from resolved keyframes. Nothing accumulates frame to frame.

Everything else falls out of this:

- **Pause / step / scrub** — evaluate any `t`, in any order.
- **Deterministic recording** — frame `f` is rendered at `t = f / fps`,
  wall clock ignored. Output is bit-identical across runs.
- **No animation bugs from ordering** — a property's value at `t` depends
  only on its own track list, never on what happened to be drawn last frame.

Keep this invariant when extending. If a new feature tempts you to mutate the
scene persistently mid-playback, express it as tracks/events instead.

## Module map

```
src/
├── lib.rs         re-exports, prelude, run() entry point
├── primitives.rs  Entity + Shape enum (circle, rect, line, arrow, polygon, text)
├── scene.rs       Scene (id-addressed entity store) + SceneBuilder (declaration DSL)
├── easing.rs      Easing enum, pure f32 → f32 curves
├── timeline.rs    TrackSpec/Clip (unresolved) → Timeline (resolved), apply()
├── animate.rs     the verb DSL: act(), ActBuilder, seq!/par!, wait()
├── movie.rs       Movie: base scene + cursor-based clip placement + sections
├── style.rs       the house style: palette constants, embedded fonts
├── render.rs      scene → pixels (one match on Shape), page chrome, project()
├── player.rs      live window w/ transport controls, or offline record loop
└── record.rs      PNG frame dump + ffmpeg stitch
```

Data flows one way:

```
SceneBuilder ─→ Scene (base, t = 0 state)
act()/seq!/par! ─→ Clip (relative times) ─→ Movie::play (absolute times)
                                              │
Movie::finalize ─→ Timeline::resolve (pins every track's `from` value)
                                              │
per frame:  Timeline::apply(base, t) ─→ Scene copy ─→ render::draw_scene
```

### How `resolve` works (the only clever part)

Verbs emit tracks with three kinds of target: `Abs` (go to this value),
`Rel` (go to current + delta), and `Revert` (go back to the value before the
previous track — used by `highlight`/`pulse` auto-restore). At finalize time,
one forward pass per (entity, property) in chronological order turns all of
these into concrete `from → to` pairs. After that, playback is dumb
interpolation.

Corollary: two tracks animating the **same property of the same entity at
overlapping times** resolve in start order and the later one wins visually.
Don't overlap them deliberately; combine different properties instead
(color + scale + opacity all coexist fine).

## How to add a new primitive

1. Add a variant to `Shape` in `primitives.rs`.
2. Add a match arm in `render::draw_entity`.
3. (Optional) Add a `SceneBuilder` method in `scene.rs` for nice declaration.
4. If it has its own animatable geometry (like `Line.to`), add a `Prop`
   variant and wire it in `timeline.rs::{get_prop, set_prop}`.

That's the complete list — nothing else in the engine knows about shapes.

## How to add a new animation verb

1. Add a variant to `Verb` in `animate.rs`.
2. Add a builder method on `ActBuilder` (this is the public API — doc it).
3. Add a match arm in `build_clip` emitting one or more `TrackSpec`s
   (and/or `TextEvent`s) over `[0, d]`.

Compound gestures are just multiple tracks: `pulse` is scale-up + `Revert`,
`shake` is six `Rel` position segments summing to zero, `set_text` is
fade-out + swap event + fade-in. Study those three before writing a new one.

## Extension seams (deliberate non-goals, kept cheap)

- **3D**: every world coordinate passes through `render::project()`, today
  the identity. A camera matrix goes there; `Entity.pos` grows a z. Nothing
  in scene/timeline/animate cares.
- **Shaders / post-processing**: `player.rs` draws chrome + scene in one
  block. Wrap that block in a render-target + material pass; the rest of the
  engine is untouched (macroquad supports custom GLSL materials already).
- **Particles**: a new `Shape` variant whose draw arm evaluates a particle
  distribution *as a pure function of `t`* (keep the invariant!).

## The house style

`style.rs` owns the identity: paper/ink/accent palette, embedded fonts
(Playfair Display for headlines, IBM Plex Mono for data — both OFL, compiled
into the binary so renders are portable), and the masthead strings.
`render::draw_page_chrome` draws the page border + masthead on every frame.
Change the look once there; every past and future video follows.

## Recording pipeline

`--record dir/` renders at fixed timestep and dumps `frame_%05d.png`, then
runs (or prints) the ffmpeg stitch command. See README for the exact flags.
`--fps 2` is a useful trick: it samples the *entire* movie sparsely — cheap
visual smoke test of a whole video without rendering thousands of frames.
