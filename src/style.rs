//! The house visual style: **newsprint / editorial** — plus the [`Theme`]
//! system that lets a movie swap the whole palette.
//!
//! Off-white paper, ink-black strokes, one muted red spot color, serif
//! display type + monospace body — every video reads like a page from the
//! same broadsheet. The classic palette lives on as consts (and as
//! [`Theme::broadsheet`], the default); [`Theme::midnight`] and
//! [`Theme::plain`] ship as alternatives, or build your own.

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

/// Semantic color roles for algorithm states. A [`Theme`] maps each role to
/// a concrete color, so scripts say *what an element means* and stay
/// readable across palettes:
///
/// ```ignore
/// m.scene().circle("n3", pos, 28.).role(Role::Visited);
/// m.play(act().highlight("n5", m.role(Role::Found)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// The element the algorithm is looking at right now.
    Active,
    /// Already examined and left behind.
    Visited,
    /// Examined and deliberately passed over.
    Skipped,
    /// The answer / a successful hit.
    Found,
    /// Out of date but still physically present (e.g. old SSTable data).
    Stale,
    /// Logically removed (tombstoned).
    Deleted,
    /// Uncertain / probabilistic ("maybe in the set").
    Maybe,
    /// Definitely not present; empty slot.
    Absent,
}

/// A complete color/typography token set. Every frame is drawn from one of
/// these; [`Theme::broadsheet`] is the default. Set it with
/// [`crate::movie::Movie::set_theme`] **before** declaring scene entities —
/// builder defaults (fills, outlines, index digits) are baked in at
/// declaration time.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Page background.
    pub paper: Color,
    /// Primary foreground: strokes, text.
    pub ink: Color,
    /// Primary spot color: highlights, warnings.
    pub accent: Color,
    /// Secondary spot color.
    pub blue: Color,
    /// De-emphasised color: annotations, indices, rules.
    pub faded: Color,
    /// Slightly-off background for subtle fills.
    pub paper_shade: Color,
    /// Masthead line, left side.
    pub masthead_left: String,
    /// Masthead line, right side.
    pub masthead_right: String,
    /// Colors for the eight semantic [`Role`]s.
    pub roles: RoleColors,
}

/// The [`Role`] → color mapping carried by a [`Theme`].
#[derive(Debug, Clone, Copy)]
pub struct RoleColors {
    pub active: Color,
    pub visited: Color,
    pub skipped: Color,
    pub found: Color,
    pub stale: Color,
    pub deleted: Color,
    pub maybe: Color,
    pub absent: Color,
}

impl Theme {
    /// The classic newsprint palette (same values as the module consts).
    pub fn broadsheet() -> Theme {
        Theme {
            paper: PAPER,
            ink: INK,
            accent: ACCENT,
            blue: BLUE,
            faded: FADED,
            paper_shade: PAPER_SHADE,
            masthead_left: MASTHEAD_LEFT.into(),
            masthead_right: MASTHEAD_RIGHT.into(),
            roles: RoleColors {
                active: ACCENT,
                visited: BLUE,
                skipped: FADED,
                found: Color::new(0.220, 0.420, 0.235, 1.0),
                stale: Color::new(0.600, 0.460, 0.220, 1.0),
                deleted: Color::new(0.420, 0.140, 0.110, 1.0),
                maybe: Color::new(0.700, 0.520, 0.150, 1.0),
                absent: PAPER_SHADE,
            },
        }
    }

    /// Dark chalkboard: near-black paper, chalk-white ink, warmer accents.
    pub fn midnight() -> Theme {
        Theme {
            paper: Color::new(0.086, 0.094, 0.110, 1.0),
            ink: Color::new(0.910, 0.902, 0.870, 1.0),
            accent: Color::new(0.870, 0.380, 0.320, 1.0),
            blue: Color::new(0.450, 0.620, 0.780, 1.0),
            faded: Color::new(0.480, 0.500, 0.530, 1.0),
            paper_shade: Color::new(0.140, 0.150, 0.170, 1.0),
            masthead_left: "THE MIDNIGHT EDITION".into(),
            masthead_right: "LATE PRESS".into(),
            roles: RoleColors {
                active: Color::new(0.870, 0.380, 0.320, 1.0),
                visited: Color::new(0.450, 0.620, 0.780, 1.0),
                skipped: Color::new(0.480, 0.500, 0.530, 1.0),
                found: Color::new(0.420, 0.700, 0.450, 1.0),
                stale: Color::new(0.720, 0.580, 0.320, 1.0),
                deleted: Color::new(0.600, 0.250, 0.220, 1.0),
                maybe: Color::new(0.830, 0.680, 0.300, 1.0),
                absent: Color::new(0.140, 0.150, 0.170, 1.0),
            },
        }
    }

    /// Neutral whiteboard: pure white paper, black ink, primary-blue accent.
    pub fn plain() -> Theme {
        Theme {
            paper: Color::new(1.0, 1.0, 1.0, 1.0),
            ink: Color::new(0.10, 0.10, 0.12, 1.0),
            accent: Color::new(0.180, 0.400, 0.780, 1.0),
            blue: Color::new(0.350, 0.350, 0.400, 1.0),
            faded: Color::new(0.600, 0.600, 0.630, 1.0),
            paper_shade: Color::new(0.945, 0.945, 0.955, 1.0),
            masthead_left: "NOTES".into(),
            masthead_right: "WORKING DRAFT".into(),
            roles: RoleColors {
                active: Color::new(0.180, 0.400, 0.780, 1.0),
                visited: Color::new(0.400, 0.300, 0.650, 1.0),
                skipped: Color::new(0.600, 0.600, 0.630, 1.0),
                found: Color::new(0.150, 0.550, 0.300, 1.0),
                stale: Color::new(0.700, 0.500, 0.150, 1.0),
                deleted: Color::new(0.780, 0.220, 0.200, 1.0),
                maybe: Color::new(0.850, 0.600, 0.100, 1.0),
                absent: Color::new(0.945, 0.945, 0.955, 1.0),
            },
        }
    }

    /// Color for a semantic [`Role`].
    pub fn role(&self, r: Role) -> Color {
        match r {
            Role::Active => self.roles.active,
            Role::Visited => self.roles.visited,
            Role::Skipped => self.roles.skipped,
            Role::Found => self.roles.found,
            Role::Stale => self.roles.stale,
            Role::Deleted => self.roles.deleted,
            Role::Maybe => self.roles.maybe,
            Role::Absent => self.roles.absent,
        }
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::broadsheet()
    }
}

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
