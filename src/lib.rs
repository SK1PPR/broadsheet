#![doc = include_str!("../README.md")]
//!
//! # Crate API
//!
//! `broadsheet` is organized around a small, scriptable pipeline:
//!
//! 1. Build a [`movie::Movie`] with a base scene.
//! 2. Declare visual entities with [`scene::SceneBuilder`].
//! 3. Add animation clips with [`animate::act`], [`seq!`], [`par!`], and
//!    [`stagger!`].
//! 4. Hand the movie to [`run`] for live preview or deterministic recording.
//!
//! The easiest entry point is [`prelude`], which re-exports the types and
//! helpers used by movie scripts.
//!
//! ## Minimal Example
//!
//! ```ignore
//! use broadsheet::prelude::*;
//!
//! fn main() {
//!     let mut m = Movie::new("Hello", 1280, 720);
//!     m.scene()
//!         .circle("A", v(300., 400.), 40.).label("A")
//!         .circle("B", v(900., 400.), 40.).label("B");
//!     m.play(seq![
//!         act().move_to("A", v(900., 500.)).dur(0.6).ease(InOutCubic),
//!         act().highlight("B", ACCENT),
//!         wait(0.5),
//!     ]);
//!     broadsheet::run(m);
//! }
//! ```
//!
//! ## Core Concepts
//!
//! - [`movie::Movie`] stores the base scene, timeline clips, section jumps, and
//!   beat marks.
//! - [`scene`] owns entity declaration: circles, rectangles, lines, arrows,
//!   text, cells, code blocks, labels, tags, and follow relationships.
//! - [`animate`] provides the fluent verb DSL: move, fade, highlight, pulse,
//!   trace, type, retarget, and camera moves.
//! - [`timeline`] resolves clips into absolute tracks. Its evaluation is a pure
//!   function of time, so pause, scrub, frame stepping, and offline recording
//!   are deterministic.
//! - [`render`] turns a scene snapshot into macroquad draw calls with the
//!   built-in broadsheet style from [`style`].
//! - [`layout`] contains small coordinate helpers for rows, grids, trees, and
//!   rings.
//!
//! ## Recording
//!
//! Run live with `cargo run --example NAME`, or record with:
//!
//! ```sh
//! cargo run --example NAME -- --record out/name --fps 60
//! ```
//!
//! Useful recording flags include `--still S`, `--from S --to S`, `--alpha`,
//! `--gif`, `--png`, `--grain`, `--scale F`, and `--frames N`.

pub mod animate;
pub mod easing;
pub mod layout;
pub mod movie;
pub mod player;
pub mod primitives;
#[cfg(not(target_arch = "wasm32"))]
pub mod record;
pub mod render;
pub mod scene;
pub mod style;
pub mod timeline;

use macroquad::prelude::Vec2;

/// Shorthand position constructor: `v(100., 200.)`.
pub fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// Open a window and run the movie (live preview, or `--record` offline).
///
/// Call this from a plain `fn main()` — no macroquad attribute needed.
pub fn run(movie: movie::Movie) {
    let opts = player::parse_opts();
    // `mut` is used only by the wasm-only WebGL2 override below
    #[allow(unused_mut)]
    let mut conf = macroquad::window::Conf {
        window_title: movie.title.clone(),
        window_width: (movie.width as f32 * opts.scale) as i32,
        window_height: (movie.height as f32 * opts.scale) as i32,
        high_dpi: false,
        window_resizable: true,
        // 4x MSAA: smooth circle/line/diagonal edges
        sample_count: 4,
        ..Default::default()
    };
    // MSAA render-target resolve needs glReadBuffer/glBlitFramebuffer, which
    // only exist in WebGL2. miniquad defaults to WebGL1, so request WebGL2 on
    // the web or the offscreen target readback throws at the first frame.
    #[cfg(target_arch = "wasm32")]
    {
        conf.platform.webgl_version = macroquad::miniquad::conf::WebGLVersion::WebGL2;
    }
    macroquad::Window::from_config(conf, player::run_loop(movie));
}

/// Everything a movie script needs: `use broadsheet::prelude::*;`
pub mod prelude {
    pub use crate::animate::{act, all, flash, stagger, wait, ActBuilder};
    pub use crate::easing::Easing::{self, *};
    pub use crate::layout;
    pub use crate::movie::Movie;
    pub use crate::style::{ACCENT, BLUE, FADED, INK, PAPER, PAPER_SHADE};
    pub use crate::timeline::Clip;
    pub use crate::v;
    pub use crate::{par, seq, stagger};
    pub use macroquad::prelude::{Color, Vec2};
}
