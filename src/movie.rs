//! The [`Movie`]: top-level container tying a scene to a timeline, with a
//! cursor-based sequencing model.
//!
//! `play(clip)` appends at the current cursor and advances it; `at(t, clip)`
//! places a clip at an absolute time without moving the cursor (for lining
//! up with narration beats); `wait(s)` leaves silence.

use macroquad::prelude::Vec2;

use macroquad::prelude::Color;

use crate::animate::{act, ActBuilder};
use crate::primitives::{Entity, Shape};
use crate::scene::{Scene, SceneBuilder};
use crate::style::{self, Role, Theme};
use crate::timeline::{Clip, TextEvent, Timeline, TrackSpec};

/// Reserved id of the animatable camera entity every movie carries.
/// Animate it with `act().cam_to(pos)` / `act().cam_zoom(z)`.
pub const CAMERA_ID: &str = "__cam";

/// A complete animation: base scene + placed clips + metadata.
pub struct Movie {
    /// Shown in the masthead of every frame and as the window title.
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub(crate) scene: Scene,
    placed: Vec<(f32, Clip)>,
    cursor: f32,
    /// (time, name) markers — the player jumps to these with keys 1–9.
    pub sections: Vec<(f32, String)>,
    /// Named beat markers from [`Movie::mark`], exported to `markers.json`
    /// alongside recordings for narration alignment.
    pub marks: Vec<(f32, String)>,
    /// Slide boundaries from [`Movie::slide`]. In `--slideshow` mode the
    /// player pauses at each one and `Space`/`→` plays to the next.
    pub slides: Vec<(f32, String)>,
    /// Visual token set every frame is drawn from. Change it with
    /// [`Movie::set_theme`] before declaring entities.
    pub theme: Theme,
    section_n: usize,
}

impl Movie {
    /// New movie with a canvas size. 1280×720 keeps live preview snappy;
    /// recordings supersample to 1080p regardless.
    pub fn new(title: &str, width: u32, height: u32) -> Movie {
        let mut scene = Scene::new();
        let mut cam = Entity::new(
            CAMERA_ID,
            Shape::Circle { r: 0.0 },
            Vec2::new(width as f32 / 2.0, height as f32 / 2.0),
            style::PAPER,
        );
        cam.opacity = 0.0;
        scene.add(cam);
        Movie {
            title: title.into(),
            width,
            height,
            scene,
            placed: Vec::new(),
            cursor: 0.0,
            sections: Vec::new(),
            marks: Vec::new(),
            slides: Vec::new(),
            theme: Theme::default(),
            section_n: 0,
        }
    }

    /// Swap the visual theme (palette, masthead, role colors). Call this
    /// **before** declaring entities: shape defaults (fills, outlines, index
    /// digits) are baked in at declaration time.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Color the current theme assigns to a semantic [`Role`]:
    /// `act().highlight("n3", m.role(Role::Found))`.
    pub fn role(&self, r: Role) -> Color {
        self.theme.role(r)
    }

    /// Declare entities (the world at t = 0). Call as many times as you like.
    pub fn scene(&mut self) -> SceneBuilder<'_> {
        SceneBuilder::new(&mut self.scene, self.theme.clone())
    }

    /// Start describing an animation act (same as the free [`act()`]).
    pub fn act(&self) -> ActBuilder {
        act()
    }

    /// Append a clip at the cursor; the cursor advances past it.
    pub fn play(&mut self, clip: impl Into<Clip>) {
        let clip = clip.into();
        self.cursor = self.cursor.max(0.0);
        let end = self.cursor + clip.dur;
        self.placed.push((self.cursor, clip));
        self.cursor = end;
    }

    /// Place a clip at an absolute time. Does not move the cursor.
    pub fn at(&mut self, t: f32, clip: impl Into<Clip>) {
        self.placed.push((t, clip.into()));
    }

    /// Advance the cursor by `s` seconds of nothing — a narration beat.
    pub fn wait(&mut self, s: f32) {
        self.cursor += s;
    }

    /// Current cursor time (useful for noting narration timestamps).
    pub fn now(&self) -> f32 {
        self.cursor
    }

    /// Drop a named beat marker at the cursor. Markers (plus sections) are
    /// written to `markers.json` next to recorded frames.
    pub fn mark(&mut self, name: &str) {
        self.marks.push((self.cursor, name.to_string()));
    }

    /// Drop a slide boundary at the cursor. Run the player with
    /// `--slideshow` to present: playback pauses at every boundary and
    /// `Space`/`→` animates forward to the next one — states in between
    /// become the "transitions" of the deck. Boundaries also land in
    /// `markers.json`.
    pub fn slide(&mut self, name: &str) {
        self.slides.push((self.cursor, name.to_string()));
    }

    /// Ids of all entities carrying `tag`. Pair with [`crate::animate::all`]:
    /// `m.play(all(&m.tagged("bits"), |id| act().fade_out(id)))`.
    pub fn tagged(&self, tag: &str) -> Vec<String> {
        self.scene
            .entities
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .map(|e| e.id.clone())
            .collect()
    }

    /// Section break: fades in a serif headline with an accent rule
    /// (newspaper section header), holds, fades out. Also records a marker
    /// the player can jump to with number keys.
    pub fn section(&mut self, title: &str) {
        self.section_n += 1;
        let n = self.section_n;
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;

        let head_id = format!("__section{n}");
        let rule_id = format!("__section{n}.rule");
        let kicker_id = format!("__section{n}.kicker");
        let bg_id = format!("__section{n}.bg");
        let (paper, ink, accent, faded) = (
            self.theme.paper,
            self.theme.ink,
            self.theme.accent,
            self.theme.faded,
        );
        {
            let mut s = self.scene();
            // backdrop keeps the card legible over a busy stage
            s.rect(&bg_id, Vec2::new(cx, cy - 10.0), 800.0, 230.0)
                .color(paper)
                .outline_color(ink)
                .stroke(1.5)
                .z(88)
                .hidden();
            s.text(&head_id, Vec2::new(cx, cy - 10.0), title)
                .serif()
                .size(58.0)
                .color(ink)
                .z(90)
                .hidden();
            s.line(
                &rule_id,
                Vec2::new(cx - 130.0, cy + 34.0),
                Vec2::new(cx + 130.0, cy + 34.0),
            )
            .color(accent)
            .stroke(3.0)
            .z(90)
            .hidden();
            s.text(&kicker_id, Vec2::new(cx, cy - 62.0), &format!("§ {n}"))
                .size(20.0)
                .color(faded)
                .z(90)
                .hidden();
        }

        self.sections.push((self.cursor, title.to_string()));
        let clip = crate::seq![
            crate::par![
                act().fade_in(&bg_id).dur(0.4),
                act().fade_in(&head_id).dur(0.4),
                act().fade_in(&rule_id).dur(0.4),
                act().fade_in(&kicker_id).dur(0.4),
            ],
            crate::timeline::Clip::wait(1.4),
            crate::par![
                act().fade_out(&bg_id).dur(0.4),
                act().fade_out(&head_id).dur(0.4),
                act().fade_out(&rule_id).dur(0.4),
                act().fade_out(&kicker_id).dur(0.4),
            ],
        ];
        self.play(clip);
    }

    /// Fade out every entity currently declared (a "clear the stage" scene
    /// change). Entities declared *after* this call are unaffected.
    /// Followers are skipped: their opacity already rides the followed
    /// entity, so fading them here would leave them stuck invisible after
    /// the parent fades back in.
    pub fn clear_all(&mut self, dur: f32) {
        let ids: Vec<String> = self
            .scene
            .entities
            .iter()
            .filter(|e| !e.id.starts_with("__") && e.follow.is_none())
            .map(|e| e.id.clone())
            .collect();
        let clips: Vec<Clip> = ids
            .iter()
            .map(|id| act().fade_out(id).dur(dur).into())
            .collect();
        self.play(Clip::par(clips));
    }

    /// Flatten placed clips into absolute-time specs and resolve keyframes.
    /// Called by the player; you rarely need it directly.
    pub fn finalize(&self) -> (Scene, Timeline) {
        let mut specs: Vec<TrackSpec> = Vec::new();
        let mut events: Vec<TextEvent> = Vec::new();
        let mut end = self.cursor;
        for (start, clip) in &self.placed {
            end = end.max(start + clip.dur);
            for t in &clip.tracks {
                let mut t = t.clone();
                t.start += start;
                specs.push(t);
            }
            for e in &clip.events {
                let mut e = e.clone();
                e.at += start;
                events.push(e);
            }
        }
        let tl = Timeline::resolve(&self.scene, specs, events, end + 1.0);
        (self.scene.clone(), tl)
    }
}
