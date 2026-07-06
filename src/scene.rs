//! The [`Scene`]: an id-addressed store of entities, plus the chainable
//! [`SceneBuilder`] used to declare the time-zero state of a movie.

use std::collections::HashMap;

use macroquad::prelude::{Color, Vec2};

use crate::primitives::{Entity, FontKind, Shape, StrokeStyle};
use crate::style;

/// An id-addressed collection of entities. This is the *base* state of the
/// world at t = 0; the timeline produces per-frame copies of it.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub entities: Vec<Entity>,
    index: HashMap<String, usize>,
}

impl Scene {
    pub fn new() -> Self {
        Scene::default()
    }

    /// Add an entity. Panics on duplicate id — ids are how animations address
    /// things, so silently shadowing one is always a bug in the movie script.
    pub fn add(&mut self, e: Entity) -> usize {
        assert!(
            !self.index.contains_key(&e.id),
            "duplicate entity id {:?}",
            e.id
        );
        let i = self.entities.len();
        self.index.insert(e.id.clone(), i);
        self.entities.push(e);
        i
    }

    pub fn get(&self, id: &str) -> Option<&Entity> {
        self.index.get(id).map(|&i| &self.entities[i])
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Entity> {
        self.index.get(id).copied().map(move |i| &mut self.entities[i])
    }

    pub fn contains(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }
}

/// Chainable builder for declaring entities. Obtained from
/// [`crate::movie::Movie::scene`]. Shape methods (`circle`, `rect`, …) add an
/// entity; modifier methods (`color`, `outlined`, `z`, …) apply to the most
/// recently added one, so declarations read top-to-bottom:
///
/// ```ignore
/// m.scene()
///     .circle("A", v(300., 400.), 40.).outlined().label("A")
///     .text("cap", v(640., 650.), "hello").size(30.).hidden();
/// ```
pub struct SceneBuilder<'a> {
    scene: &'a mut Scene,
    last: Option<usize>,
}

impl<'a> SceneBuilder<'a> {
    pub(crate) fn new(scene: &'a mut Scene) -> Self {
        SceneBuilder { scene, last: None }
    }

    fn push(&mut self, e: Entity) -> &mut Self {
        self.last = Some(self.scene.add(e));
        self
    }

    fn last_mut(&mut self) -> &mut Entity {
        let i = self.last.expect("modifier called before any shape was added");
        &mut self.scene.entities[i]
    }

    // ---- shapes -------------------------------------------------------

    /// Circle centred at `pos` with radius `r`. Ink-outlined, paper-filled by
    /// default (the house style for nodes).
    pub fn circle(&mut self, id: &str, pos: Vec2, r: f32) -> &mut Self {
        let mut e = Entity::new(id, Shape::Circle { r }, pos, style::PAPER);
        e.stroke = StrokeStyle { fill: true, outline: true, outline_color: Some(style::INK), ..Default::default() };
        self.push(e)
    }

    /// Rectangle centred at `pos`. Same default styling as `circle`.
    pub fn rect(&mut self, id: &str, pos: Vec2, w: f32, h: f32) -> &mut Self {
        let mut e = Entity::new(id, Shape::Rect { w, h }, pos, style::PAPER);
        e.stroke = StrokeStyle { fill: true, outline: true, outline_color: Some(style::INK), ..Default::default() };
        self.push(e)
    }

    /// Line from `from` to `to` (absolute coordinates).
    pub fn line(&mut self, id: &str, from: Vec2, to: Vec2) -> &mut Self {
        self.push(Entity::new(id, Shape::Line { to }, from, style::INK))
    }

    /// Arrow from `from` to `to`, head at `to`.
    pub fn arrow(&mut self, id: &str, from: Vec2, to: Vec2) -> &mut Self {
        self.push(Entity::new(id, Shape::Arrow { to }, from, style::INK))
    }

    /// Polygon with absolute points. Animate its `pos` to move it as a unit.
    pub fn polygon(&mut self, id: &str, pts: Vec<Vec2>) -> &mut Self {
        let mut e = Entity::new(id, Shape::Polygon { pts }, Vec2::ZERO, style::PAPER);
        e.stroke = StrokeStyle { fill: true, outline: true, outline_color: Some(style::INK), ..Default::default() };
        self.push(e)
    }

    /// Text centred at `pos`. Mono font, size 28, ink by default.
    pub fn text(&mut self, id: &str, pos: Vec2, content: &str) -> &mut Self {
        self.push(Entity::new(
            id,
            Shape::Text { content: content.into(), size: 28.0 },
            pos,
            style::INK,
        ))
    }

    // ---- modifiers (apply to the last shape added) ---------------------

    /// Set the primary (fill) color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.last_mut().color = c;
        self
    }

    /// Outline only: no fill, ink-colored stroke unless overridden.
    pub fn outlined(&mut self) -> &mut Self {
        let e = self.last_mut();
        e.stroke.fill = false;
        e.stroke.outline = true;
        self
    }

    /// Fill only, no outline.
    pub fn filled(&mut self) -> &mut Self {
        let e = self.last_mut();
        e.stroke.fill = true;
        e.stroke.outline = false;
        self
    }

    /// Outline thickness in pixels (also line/arrow thickness).
    pub fn stroke(&mut self, w: f32) -> &mut Self {
        self.last_mut().stroke.width = w;
        self
    }

    /// Outline color, independent of the fill color.
    pub fn outline_color(&mut self, c: Color) -> &mut Self {
        self.last_mut().stroke.outline_color = Some(c);
        self
    }

    /// Text size (points). Only meaningful for `text` entities.
    pub fn size(&mut self, s: f32) -> &mut Self {
        if let Shape::Text { size, .. } = &mut self.last_mut().shape {
            *size = s;
        }
        self
    }

    /// Use the serif display font (headlines).
    pub fn serif(&mut self) -> &mut Self {
        self.last_mut().font = FontKind::Serif;
        self
    }

    /// Use the bold mono font.
    pub fn mono_bold(&mut self) -> &mut Self {
        self.last_mut().font = FontKind::MonoBold;
        self
    }

    /// Draw order; higher on top.
    pub fn z(&mut self, z: i32) -> &mut Self {
        self.last_mut().z = z;
        self
    }

    /// Start invisible (opacity 0) — reveal later with `fade_in`.
    pub fn hidden(&mut self) -> &mut Self {
        self.last_mut().opacity = 0.0;
        self
    }

    /// Explicit starting opacity.
    pub fn opacity(&mut self, o: f32) -> &mut Self {
        self.last_mut().opacity = o;
        self
    }

    /// Rotation in degrees (text only — e.g. rubber stamps).
    pub fn rot(&mut self, deg: f32) -> &mut Self {
        self.last_mut().rot = deg;
        self
    }

    /// Word-wrap this text entity at `px` logical pixels; wrapped lines are
    /// centred as a block on the entity's position.
    pub fn wrap(&mut self, px: f32) -> &mut Self {
        self.last_mut().wrap = Some(px);
        self
    }

    /// Pin this entity's position to another entity plus an offset. Its
    /// opacity is also multiplied by the followed entity's opacity.
    pub fn follow(&mut self, id: &str, offset: Vec2) -> &mut Self {
        self.last_mut().follow = Some((id.into(), offset));
        self
    }

    /// Attach a centred text label riding on this entity. The label is its
    /// own entity with id `"{parent}.label"`, so it can be animated too
    /// (e.g. `color_to("bit3.label", PAPER)`).
    pub fn label(&mut self, text: &str) -> &mut Self {
        let (parent_id, parent_z) = {
            let e = self.last_mut();
            (e.id.clone(), e.z)
        };
        let mut lbl = Entity::new(
            format!("{parent_id}.label"),
            Shape::Text { content: text.into(), size: 24.0 },
            Vec2::ZERO,
            style::INK,
        );
        lbl.font = FontKind::MonoBold;
        lbl.z = parent_z + 1;
        lbl.follow = Some((parent_id, Vec2::ZERO));
        self.push(lbl)
    }
}
