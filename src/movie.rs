//! The [`Movie`]: top-level container tying a scene to a timeline, with a
//! cursor-based sequencing model.
//!
//! `play(clip)` appends at the current cursor and advances it; `at(t, clip)`
//! places a clip at an absolute time without moving the cursor (for lining
//! up with narration beats); `wait(s)` leaves silence.

use macroquad::prelude::Vec2;

use crate::animate::{act, ActBuilder};
use crate::scene::{Scene, SceneBuilder};
use crate::style;
use crate::timeline::{Clip, TextEvent, Timeline, TrackSpec};

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
    section_n: usize,
}

impl Movie {
    /// New movie with a canvas size. 1280×720 keeps live preview snappy;
    /// bump to 1920×1080 for final renders.
    pub fn new(title: &str, width: u32, height: u32) -> Movie {
        Movie {
            title: title.into(),
            width,
            height,
            scene: Scene::new(),
            placed: Vec::new(),
            cursor: 0.0,
            sections: Vec::new(),
            section_n: 0,
        }
    }

    /// Declare entities (the world at t = 0). Call as many times as you like.
    pub fn scene(&mut self) -> SceneBuilder<'_> {
        SceneBuilder::new(&mut self.scene)
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
        {
            let mut s = self.scene();
            // backdrop keeps the card legible over a busy stage
            s.rect(&bg_id, Vec2::new(cx, cy - 10.0), 800.0, 230.0)
                .color(style::PAPER)
                .outline_color(style::INK)
                .stroke(1.5)
                .z(88)
                .hidden();
            s.text(&head_id, Vec2::new(cx, cy - 10.0), title)
                .serif()
                .size(58.0)
                .color(style::INK)
                .z(90)
                .hidden();
            s.line(
                &rule_id,
                Vec2::new(cx - 130.0, cy + 34.0),
                Vec2::new(cx + 130.0, cy + 34.0),
            )
            .color(style::ACCENT)
            .stroke(3.0)
            .z(90)
            .hidden();
            s.text(&kicker_id, Vec2::new(cx, cy - 62.0), &format!("§ {n}"))
                .size(20.0)
                .color(style::FADED)
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
    pub fn clear_all(&mut self, dur: f32) {
        let ids: Vec<String> = self
            .scene
            .entities
            .iter()
            .filter(|e| !e.id.starts_with("__section"))
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
