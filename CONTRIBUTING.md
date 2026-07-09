# Contributing to broadsheet

Thanks for looking! Contributions welcome — bug reports, new widgets,
layouts, verbs, themes, or documentation.

## Ground rules

1. **The timeline stays stateless.** `Timeline::apply(base, t)` must remain
   a pure function of absolute time — no state accumulating between frames.
   This is what makes scrubbing, stepping, and deterministic recording work.
   Any change that breaks it will be declined regardless of how nice the
   feature is.
2. **Determinism everywhere.** Same script + seed = same pixels. No wall
   clock, no unseeded randomness in anything that reaches the canvas.
3. **Read [ARCHITECTURE.md](ARCHITECTURE.md) first** — module map plus
   step-by-step recipes for the two most common extensions (new primitive,
   new verb).

## Common extension points

| Want to add | Touch |
|---|---|
| a new shape | `primitives::Shape` variant + match arm in `render::draw_entity` |
| a new animation verb | `animate::Verb` variant + builder method + arm in `build_clip` |
| a data-structure widget | `widgets.rs` — declare entities, return an id handle |
| a layout helper | `layout.rs` — pure `Vec2` math, add a unit test |
| a theme | `style.rs` — a `Theme` constructor |

## Before you open a PR

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --examples
```

CI runs exactly these. For anything visual, include a still or short clip
(`cargo run --example X -- --still 5` / `--gif --from A --to B`) in the PR
description so the change can be seen.
