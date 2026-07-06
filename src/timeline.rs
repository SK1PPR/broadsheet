//! The keyframe timeline: tracks, clips, and stateless evaluation.
//!
//! Design invariant: **evaluation is a pure function of absolute time**.
//! `Timeline::apply(base, t)` computes every animated property at time `t`
//! from resolved keyframes — no accumulated state. That is what makes pause,
//! frame-stepping, scrubbing, and deterministic offline rendering all free.
//!
//! Flow: verbs ([`crate::animate::ActBuilder`]) emit relative [`TrackSpec`]s
//! inside [`Clip`]s → `seq`/`par` compose clips → [`crate::movie::Movie`]
//! places clips at absolute times → [`Timeline::resolve`] does one forward
//! pass to pin down each track's `from` value → playback just interpolates.

use std::collections::HashMap;

use macroquad::prelude::{Color, Vec2};

use crate::easing::Easing;
use crate::primitives::Shape;
use crate::scene::Scene;

/// A dynamically-typed animatable value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    F(f32),
    V(Vec2),
    C(Color),
}

impl Value {
    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::F(a), Value::F(b)) => Value::F(a + b),
            (Value::V(a), Value::V(b)) => Value::V(a + b),
            // color has no meaningful Rel semantics; keep the delta operand
            _ => other,
        }
    }

    fn lerp(a: Value, b: Value, u: f32) -> Value {
        match (a, b) {
            (Value::F(x), Value::F(y)) => Value::F(x + (y - x) * u),
            (Value::V(x), Value::V(y)) => Value::V(x + (y - x) * u),
            (Value::C(x), Value::C(y)) => Value::C(Color::new(
                x.r + (y.r - x.r) * u,
                x.g + (y.g - x.g) * u,
                x.b + (y.b - x.b) * u,
                x.a + (y.a - x.a) * u,
            )),
            _ => b,
        }
    }
}

/// Which property of an entity a track animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prop {
    Pos,
    /// Endpoint of a `Line`/`Arrow` shape.
    To,
    Color,
    Opacity,
    Scale,
}

/// Where a track ends up. `Rel` and `Revert` are resolved to absolute values
/// in [`Timeline::resolve`], once the chronologically-previous value is known.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetValue {
    /// Animate to this absolute value.
    Abs(Value),
    /// Animate to (current value + delta). Used by `move_by`, `pulse`, `shake`.
    Rel(Value),
    /// Animate back to the value the property had *before the previous track
    /// started*. Used by `highlight`/`pulse` to auto-restore.
    Revert,
}

/// One property animation, with `start` relative to its enclosing [`Clip`].
#[derive(Debug, Clone)]
pub struct TrackSpec {
    pub id: String,
    pub prop: Prop,
    pub target: TargetValue,
    pub start: f32,
    pub dur: f32,
    pub easing: Easing,
}

/// An instantaneous text-content swap (used by `set_text` mid-crossfade).
#[derive(Debug, Clone)]
pub struct TextEvent {
    pub id: String,
    pub content: String,
    pub at: f32,
}

/// A composable bundle of tracks/events with a known duration. Clips are
/// what `seq!`/`par!` operate on; times inside are relative to clip start.
#[derive(Debug, Clone, Default)]
pub struct Clip {
    pub tracks: Vec<TrackSpec>,
    pub events: Vec<TextEvent>,
    pub dur: f32,
}

impl Clip {
    /// An empty clip that occupies `d` seconds — a narration beat.
    pub fn wait(d: f32) -> Clip {
        Clip { dur: d, ..Default::default() }
    }

    /// Shift everything in this clip later by `dt`.
    pub fn shift(mut self, dt: f32) -> Clip {
        for t in &mut self.tracks {
            t.start += dt;
        }
        for e in &mut self.events {
            e.at += dt;
        }
        self.dur += dt;
        self
    }

    /// Run clips one after another. Total duration = sum.
    pub fn seq(clips: Vec<Clip>) -> Clip {
        let mut out = Clip::default();
        for c in clips {
            let offset = out.dur;
            for mut t in c.tracks {
                t.start += offset;
                out.tracks.push(t);
            }
            for mut e in c.events {
                e.at += offset;
                out.events.push(e);
            }
            out.dur += c.dur;
        }
        out
    }

    /// Run clips at the same time. Total duration = longest.
    pub fn par(clips: Vec<Clip>) -> Clip {
        let mut out = Clip::default();
        for c in clips {
            out.tracks.extend(c.tracks);
            out.events.extend(c.events);
            out.dur = out.dur.max(c.dur);
        }
        out
    }
}

/// A fully-resolved track: `from` is a concrete value, so evaluation at any
/// `t` is direct interpolation.
#[derive(Debug, Clone)]
struct Track {
    from: Value,
    to: Value,
    start: f32,
    dur: f32,
    easing: Easing,
}

/// The resolved, immutable animation program for a movie.
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    /// Per (entity, property): tracks sorted by start time.
    tracks: HashMap<(String, Prop), Vec<Track>>,
    events: Vec<TextEvent>,
    /// Total duration in seconds.
    pub dur: f32,
}

fn get_prop(scene: &Scene, id: &str, prop: Prop) -> Option<Value> {
    let e = scene.get(id)?;
    Some(match prop {
        Prop::Pos => Value::V(e.pos),
        Prop::Color => Value::C(e.color),
        Prop::Opacity => Value::F(e.opacity),
        Prop::Scale => Value::F(e.scale),
        Prop::To => match &e.shape {
            Shape::Line { to } | Shape::Arrow { to } => Value::V(*to),
            _ => return None,
        },
    })
}

fn set_prop(scene: &mut Scene, id: &str, prop: Prop, v: Value) {
    let Some(e) = scene.get_mut(id) else { return };
    match (prop, v) {
        (Prop::Pos, Value::V(p)) => e.pos = p,
        (Prop::Color, Value::C(c)) => e.color = c,
        (Prop::Opacity, Value::F(o)) => e.opacity = o,
        (Prop::Scale, Value::F(s)) => e.scale = s,
        (Prop::To, Value::V(p)) => {
            if let Shape::Line { to } | Shape::Arrow { to } = &mut e.shape {
                *to = p;
            }
        }
        _ => {}
    }
}

impl Timeline {
    /// Resolve absolute-time track specs against the base scene.
    ///
    /// One forward pass per (entity, property) in chronological order pins
    /// down each track's `from` (= previous track's end, or the base value),
    /// turns `Rel` deltas into absolute targets, and gives `Revert` the value
    /// the property had before the preceding track began.
    ///
    /// Panics if a track references an unknown entity id — better to fail at
    /// build time than to render a movie with a silent no-op animation.
    pub fn resolve(base: &Scene, specs: Vec<TrackSpec>, mut events: Vec<TextEvent>, dur: f32) -> Timeline {
        let mut grouped: HashMap<(String, Prop), Vec<TrackSpec>> = HashMap::new();
        for s in specs {
            assert!(
                base.contains(&s.id),
                "animation references unknown entity id {:?}",
                s.id
            );
            grouped.entry((s.id.clone(), s.prop)).or_default().push(s);
        }

        let mut tracks: HashMap<(String, Prop), Vec<Track>> = HashMap::new();
        for ((id, prop), mut specs) in grouped {
            specs.sort_by(|a, b| a.start.total_cmp(&b.start));
            let base_val = get_prop(base, &id, prop)
                .unwrap_or_else(|| panic!("entity {id:?} has no property {prop:?}"));
            let mut cur = base_val;
            let mut prev_from: Option<Value> = None;
            let mut resolved = Vec::with_capacity(specs.len());
            for s in specs {
                let from = cur;
                let to = match s.target {
                    TargetValue::Abs(v) => v,
                    TargetValue::Rel(v) => from.add(v),
                    TargetValue::Revert => prev_from.unwrap_or(base_val),
                };
                resolved.push(Track { from, to, start: s.start, dur: s.dur, easing: s.easing });
                prev_from = Some(from);
                cur = to;
            }
            tracks.insert((id, prop), resolved);
        }

        events.sort_by(|a, b| a.at.total_cmp(&b.at));
        Timeline { tracks, events, dur }
    }

    /// Evaluate the world at absolute time `t`: returns a fresh copy of the
    /// base scene with every animated property set. Pure — call with any `t`
    /// in any order (this is what scrubbing does).
    pub fn apply(&self, base: &Scene, t: f32) -> Scene {
        let mut scene = base.clone();

        for ((id, prop), tracks) in &self.tracks {
            let mut value: Option<Value> = None;
            for tr in tracks {
                if t < tr.start {
                    break;
                } else if t < tr.start + tr.dur && tr.dur > 0.0 {
                    let u = tr.easing.apply((t - tr.start) / tr.dur);
                    value = Some(Value::lerp(tr.from, tr.to, u));
                    break;
                } else {
                    value = Some(tr.to);
                }
            }
            if let Some(v) = value {
                set_prop(&mut scene, id, *prop, v);
            }
        }

        // latest text event wins (events are sorted by time)
        for ev in &self.events {
            if ev.at > t {
                break;
            }
            if let Some(e) = scene.get_mut(&ev.id) {
                if let Shape::Text { content, .. } = &mut e.shape {
                    *content = ev.content.clone();
                }
            }
        }

        // follow pass: pin followers to their targets. Two passes so a
        // follower-of-a-follower (rare) still lands close enough.
        for _ in 0..2 {
            for i in 0..scene.entities.len() {
                if let Some((target, offset)) = scene.entities[i].follow.clone() {
                    if let Some(te) = scene.get(&target) {
                        let (p, o) = (te.pos + offset, te.opacity);
                        scene.entities[i].pos = p;
                        scene.entities[i].opacity =
                            base.entities[i].opacity_animated(self, &base.entities[i].id, t) * o;
                    }
                }
            }
        }

        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::easing::Easing;
    use crate::primitives::Entity;
    use macroquad::prelude::Vec2;

    fn scene_with_dot() -> Scene {
        let mut s = Scene::new();
        s.add(Entity::new(
            "dot",
            Shape::Circle { r: 10.0 },
            Vec2::new(0.0, 0.0),
            Color::new(0.0, 0.0, 0.0, 1.0),
        ));
        s
    }

    fn spec(prop: Prop, target: TargetValue, start: f32, dur: f32) -> TrackSpec {
        TrackSpec { id: "dot".into(), prop, target, start, dur, easing: Easing::Linear }
    }

    #[test]
    fn abs_track_interpolates_and_holds() {
        let base = scene_with_dot();
        let tl = Timeline::resolve(
            &base,
            vec![spec(Prop::Pos, TargetValue::Abs(Value::V(Vec2::new(100.0, 0.0))), 1.0, 2.0)],
            vec![],
            5.0,
        );
        assert_eq!(tl.apply(&base, 0.0).get("dot").unwrap().pos.x, 0.0); // before
        assert_eq!(tl.apply(&base, 2.0).get("dot").unwrap().pos.x, 50.0); // midpoint
        assert_eq!(tl.apply(&base, 4.0).get("dot").unwrap().pos.x, 100.0); // held after
    }

    #[test]
    fn rel_chains_from_previous_end_and_revert_restores() {
        let base = scene_with_dot();
        // move +100, then revert (pulse/highlight pattern)
        let tl = Timeline::resolve(
            &base,
            vec![
                spec(Prop::Pos, TargetValue::Rel(Value::V(Vec2::new(100.0, 0.0))), 0.0, 1.0),
                spec(Prop::Pos, TargetValue::Revert, 2.0, 1.0),
            ],
            vec![],
            5.0,
        );
        assert_eq!(tl.apply(&base, 1.5).get("dot").unwrap().pos.x, 100.0);
        assert_eq!(tl.apply(&base, 4.0).get("dot").unwrap().pos.x, 0.0);
    }

    #[test]
    fn evaluation_is_order_independent() {
        // scrubbing backwards must give identical results to playing forward
        let base = scene_with_dot();
        let tl = Timeline::resolve(
            &base,
            vec![spec(Prop::Opacity, TargetValue::Abs(Value::F(0.0)), 0.5, 1.0)],
            vec![],
            3.0,
        );
        let forward: Vec<f32> = (0..30)
            .map(|i| tl.apply(&base, i as f32 * 0.1).get("dot").unwrap().opacity)
            .collect();
        let backward: Vec<f32> = (0..30)
            .rev()
            .map(|i| tl.apply(&base, i as f32 * 0.1).get("dot").unwrap().opacity)
            .collect();
        let mut backward = backward;
        backward.reverse();
        assert_eq!(forward, backward);
    }

    #[test]
    fn easing_endpoints_are_exact() {
        use crate::easing::Easing::*;
        for e in [Linear, InQuad, OutQuad, InOutQuad, InCubic, OutCubic, InOutCubic, OutBack, OutElastic, OutBounce] {
            assert!((e.apply(0.0)).abs() < 1e-4, "{e:?} at 0");
            assert!((e.apply(1.0) - 1.0).abs() < 1e-4, "{e:?} at 1");
        }
    }

    #[test]
    #[should_panic(expected = "unknown entity")]
    fn unknown_id_fails_at_resolve_not_playback() {
        let base = scene_with_dot();
        Timeline::resolve(
            &base,
            vec![TrackSpec {
                id: "typo".into(),
                prop: Prop::Opacity,
                target: TargetValue::Abs(Value::F(0.0)),
                start: 0.0,
                dur: 1.0,
                easing: Easing::Linear,
            }],
            vec![],
            1.0,
        );
    }
}

/// Small helper so the follow pass can combine "the follower's own animated
/// opacity" with the followed entity's opacity, instead of overwriting it.
trait OpacityAt {
    fn opacity_animated(&self, tl: &Timeline, id: &str, t: f32) -> f32;
}

impl OpacityAt for crate::primitives::Entity {
    fn opacity_animated(&self, tl: &Timeline, id: &str, t: f32) -> f32 {
        let key = (id.to_string(), Prop::Opacity);
        let Some(tracks) = tl.tracks.get(&key) else { return self.opacity };
        let mut value = self.opacity;
        for tr in tracks {
            if t < tr.start {
                break;
            } else if t < tr.start + tr.dur && tr.dur > 0.0 {
                let u = tr.easing.apply((t - tr.start) / tr.dur);
                if let Value::F(v) = Value::lerp(tr.from, tr.to, u) {
                    value = v;
                }
                break;
            } else if let Value::F(v) = tr.to {
                value = v;
            }
        }
        value
    }
}
