# Changelog

All notable changes to `broadsheet`. Versioning follows semver; pre-1.0,
minor bumps may include breaking changes (they are listed).

## 0.3.0 — 2026-07-09

### Added

- **Themes** — `style::Theme` token set (palette, masthead strings, role
  colors) with `broadsheet()` (default), `midnight()`, and `plain()`
  presets. Swap with `Movie::set_theme(...)` before declaring entities.
- **Semantic roles** — `style::Role` (`Active`, `Visited`, `Skipped`,
  `Found`, `Stale`, `Deleted`, `Maybe`, `Absent`), mapped to colors by the
  active theme. `.role(...)` on the scene builder, `Movie::role(...)` for
  animation verbs.
- **Slideshow mode** — `Movie::slide("name")` drops boundaries; the
  `--slideshow` player flag pauses at each one, `Space`/`→` animates to the
  next, `←` snaps back. Slides are exported to `markers.json`.
- **Widgets** — `widgets::{array, bit_array, hash_table, linked_list, tree,
  graph, hash_ring, lsm_levels, skip_list}`: one call declares the whole
  structure and returns a handle exposing its entity ids.
- **Layouts** — `layout::levels` (skip lists / LSM / hierarchies),
  `layout::blocks` (pages/chunks), `layout::graph` (deterministic seeded
  spring layout), `layout::rng` (splitmix64 for repeatable jitter).
- `examples/slideshow_demo.rs` (seven-slide widget tour, midnight theme)
  and `examples/hello.rs` (minimal starter).

### Breaking

- `render::draw_page_chrome` takes a `&Theme` parameter.
- `record::Recorder::finish` takes a third `slides` slice.
- `markers.json` gains a `slides` array.

## 0.2.3 — 2026-07-09

- Export the beat-mark table to wasm hosts (`bs_mark_*`).

## 0.2.2 — 2026-07-08

- Opaque paper web background (`bare` mode) for seamless embedding.

## 0.2.1 — 2026-07-08

- Seamless web embedding: paper letterbox, page-driven playback.

## 0.2.0 — 2026-07-08

- WebAssembly support: WebGL2 config, JS interop (`bs_seek`, `bs_time`,
  `bs_duration`, `bs_set_paused`) for scroll-scrubbed playback.

## 0.1.x — 2026-07-06 … 07-08

- Initial release: entity scene, stateless keyframe timeline, animation verb
  DSL (`seq!`/`par!`/`stagger!`), newspaper page style, live preview player,
  deterministic recording through ffmpeg, beat-marker export, camera
  pan/zoom, trace/typewriter reveals, word wrap, supersampling.
