//! The house visual style: **newsprint / editorial**.
//!
//! Off-white paper, ink-black strokes, one muted red spot color, serif
//! display type + monospace body — every video reads like a page from the
//! same broadsheet. Change the palette here and every movie follows.

use macroquad::prelude::Color;
use macroquad::text::{load_ttf_font_from_bytes, Font};

/// Background: warm off-white newsprint.
pub const PAPER: Color = Color::new(0.957, 0.945, 0.902, 1.0);
/// Primary foreground: near-black ink.
pub const INK: Color = Color::new(0.098, 0.090, 0.078, 1.0);
/// Spot color: newsprint red. Use for highlights, set bits, warnings.
pub const ACCENT: Color = Color::new(0.659, 0.224, 0.180, 1.0);
/// Secondary spot color: editorial slate blue. Use for "the other thing".
pub const BLUE: Color = Color::new(0.180, 0.290, 0.400, 1.0);
/// De-emphasised gray-brown for annotations, indices, rules.
pub const FADED: Color = Color::new(0.541, 0.514, 0.459, 1.0);
/// Slightly darker paper for subtle fills (e.g. unset bits).
pub const PAPER_SHADE: Color = Color::new(0.914, 0.898, 0.847, 1.0);

/// Masthead line printed at the top of every frame.
pub const MASTHEAD_LEFT: &str = "THE BROADSHEET";
pub const MASTHEAD_RIGHT: &str = "SPECIAL EDITION";

/// Loaded font set. `None` fields fall back to macroquad's built-in font.
pub struct Fonts {
    pub serif: Option<Font>,
    pub mono: Option<Font>,
    pub mono_bold: Option<Font>,
}

impl Fonts {
    /// Load the embedded house fonts (Playfair Display + IBM Plex Mono, both
    /// OFL-licensed and compiled into the binary, so movies render
    /// identically on any machine).
    pub fn load() -> Fonts {
        Fonts {
            serif: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/PlayfairDisplay-Bold.ttf"
            ))
            .ok(),
            mono: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/IBMPlexMono-Regular.ttf"
            ))
            .ok(),
            mono_bold: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/IBMPlexMono-Bold.ttf"
            ))
            .ok(),
        }
    }
}

/// `c` with its alpha multiplied by `opacity`.
pub fn with_opacity(c: Color, opacity: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * opacity)
}
