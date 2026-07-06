//! # broadsheet — a 2D animation engine for algorithm explainer videos
//!
//! Newspaper-styled, deterministic, code-driven animations on macroquad.
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
//! Run live (`cargo run --example …`), or record deterministic frames with
//! `cargo run --example … -- --record out/ --fps 60`.
//!
//! See `ARCHITECTURE.md` for the module map and extension recipes.

pub mod animate;
pub mod easing;
pub mod movie;
pub mod player;
pub mod primitives;
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
    let conf = macroquad::window::Conf {
        window_title: movie.title.clone(),
        window_width: (movie.width as f32 * opts.scale) as i32,
        window_height: (movie.height as f32 * opts.scale) as i32,
        high_dpi: false,
        window_resizable: false,
        // 4x MSAA: smooth circle/line/diagonal edges
        sample_count: 4,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, player::run_loop(movie));
}

/// Everything a movie script needs: `use broadsheet::prelude::*;`
pub mod prelude {
    pub use crate::animate::{act, flash, wait, ActBuilder};
    pub use crate::easing::Easing::{self, *};
    pub use crate::movie::Movie;
    pub use crate::style::{ACCENT, BLUE, FADED, INK, PAPER, PAPER_SHADE};
    pub use crate::timeline::Clip;
    pub use crate::v;
    pub use crate::{par, seq};
    pub use macroquad::prelude::{Color, Vec2};
}
