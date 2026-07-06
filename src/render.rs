//! The macroquad draw pass: scene → pixels, plus the newspaper page chrome.
//!
//! New primitive = match arm in [`draw_entity`]. All coordinates flow
//! through [`project`]: today the supersampling scale, later a 3D camera.

use macroquad::prelude::*;

use crate::primitives::{Entity, FontKind, Shape};
use crate::scene::Scene;
use crate::style::{self, Fonts};

/// World (logical) → screen (physical). The seam for supersampling now and
/// a camera/3D projection later.
#[inline]
pub fn project(p: Vec2, s: f32) -> Vec2 {
    p * s
}

fn font_of(fonts: &Fonts, kind: FontKind) -> Option<&Font> {
    match kind {
        FontKind::Serif => fonts.serif.as_ref(),
        FontKind::Mono => fonts.mono.as_ref(),
        FontKind::MonoBold => fonts.mono_bold.as_ref(),
    }
}

/// Greedy word-wrap: pack words into lines no wider than `max_w` px.
fn wrap_lines(text: &str, font: Option<&Font>, font_size: u16, max_w: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let cand = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if !cur.is_empty() && measure_text(&cand, font, font_size, 1.0).width > max_w {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = cand;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Draw `text` centred on `pos` (physical pixels, physical `size`).
/// With `wrap`, text breaks into lines and the whole block centres on `pos`.
pub fn draw_centered_text(
    text: &str,
    pos: Vec2,
    size: f32,
    color: Color,
    font: Option<&Font>,
    rot_deg: f32,
    wrap: Option<f32>,
) {
    let font_size = size.max(1.0).round() as u16;
    let lines = match wrap {
        Some(w) => wrap_lines(text, font, font_size, w),
        None => vec![text.to_string()],
    };
    let line_h = size * 1.4;
    let y0 = pos.y - line_h * (lines.len() as f32 - 1.0) / 2.0;
    for (i, line) in lines.iter().enumerate() {
        let dims = measure_text(line, font, font_size, 1.0);
        draw_text_ex(
            line,
            pos.x - dims.width / 2.0,
            y0 + line_h * i as f32 + dims.offset_y / 2.0,
            TextParams {
                font,
                font_size,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: rot_deg.to_radians(),
                color,
            },
        );
    }
}

fn draw_arrow_shape(from: Vec2, to: Vec2, width: f32, color: Color) {
    let d = to - from;
    let len = d.length();
    if len < 0.5 {
        return; // zero-length, e.g. before a grow_to starts
    }
    let dir = d / len;
    let head_len = (10.0 + width * 2.5).min(len);
    let head_w = head_len * 0.5;
    let base = to - dir * head_len;
    let perp = Vec2::new(-dir.y, dir.x);
    draw_line(from.x, from.y, base.x, base.y, width, color);
    draw_triangle(to, base + perp * head_w, base - perp * head_w, color);
}

fn draw_polygon_shape(pts: &[Vec2], offset: Vec2, e: &Entity, fill: Color, outline: Color, s: f32) {
    if pts.len() < 3 {
        return;
    }
    let p: Vec<Vec2> = pts.iter().map(|&q| project(q + offset, s)).collect();
    if e.stroke.fill {
        // triangle fan; assumes convex
        for i in 1..p.len() - 1 {
            draw_triangle(p[0], p[i], p[i + 1], fill);
        }
    }
    if e.stroke.outline {
        for i in 0..p.len() {
            let a = p[i];
            let b = p[(i + 1) % p.len()];
            draw_line(a.x, a.y, b.x, b.y, e.stroke.width * s, outline);
        }
    }
}

/// Draw one entity at supersampling factor `s`.
pub fn draw_entity(e: &Entity, fonts: &Fonts, s: f32) {
    if e.opacity <= 0.001 {
        return;
    }
    let fill = style::with_opacity(e.color, e.opacity);
    let outline = style::with_opacity(e.stroke.outline_color.unwrap_or(e.color), e.opacity);
    let p = project(e.pos, s);

    match &e.shape {
        Shape::Circle { r } => {
            let r = r * e.scale * s;
            if e.stroke.fill {
                draw_circle(p.x, p.y, r, fill);
            }
            if e.stroke.outline {
                draw_circle_lines(p.x, p.y, r, e.stroke.width * s, outline);
            }
        }
        Shape::Rect { w, h } => {
            let (w, h) = (w * e.scale * s, h * e.scale * s);
            let (x, y) = (p.x - w / 2.0, p.y - h / 2.0);
            if e.stroke.fill {
                draw_rectangle(x, y, w, h, fill);
            }
            if e.stroke.outline {
                draw_rectangle_lines(x, y, w, h, e.stroke.width * 2.0 * s, outline);
            }
        }
        Shape::Line { to } => {
            let q = project(*to, s);
            draw_line(p.x, p.y, q.x, q.y, e.stroke.width * e.scale * s, fill);
        }
        Shape::Arrow { to } => {
            draw_arrow_shape(p, project(*to, s), e.stroke.width * e.scale * s, fill);
        }
        Shape::Polygon { pts } => {
            draw_polygon_shape(pts, e.pos, e, fill, outline, s);
        }
        Shape::Text { content, size } => {
            draw_centered_text(
                content,
                p,
                size * e.scale * s,
                fill,
                font_of(fonts, e.font),
                e.rot,
                e.wrap.map(|w| w * s),
            );
        }
    }
}

/// Draw a whole scene in z-order (stable within equal z).
pub fn draw_scene(scene: &Scene, fonts: &Fonts, s: f32) {
    let mut order: Vec<usize> = (0..scene.entities.len()).collect();
    order.sort_by_key(|&i| scene.entities[i].z);
    for i in order {
        draw_entity(&scene.entities[i], fonts, s);
    }
}

/// The newspaper page chrome drawn under every frame: double border,
/// masthead title, dateline rules. `w`/`h` are logical; `s` is the
/// supersampling factor.
pub fn draw_page_chrome(title: &str, w: f32, h: f32, fonts: &Fonts, s: f32) {
    clear_background(style::PAPER);
    let (pw, ph) = (w * s, h * s);

    // double page border
    draw_rectangle_lines(
        16.0 * s,
        16.0 * s,
        pw - 32.0 * s,
        ph - 32.0 * s,
        3.0 * s,
        style::INK,
    );
    draw_rectangle_lines(
        24.0 * s,
        24.0 * s,
        pw - 48.0 * s,
        ph - 48.0 * s,
        1.0 * s,
        style::FADED,
    );

    // masthead
    let title_upper = title.to_uppercase();
    draw_centered_text(
        &title_upper,
        Vec2::new(pw / 2.0, 58.0 * s),
        40.0 * s,
        style::INK,
        fonts.serif.as_ref(),
        0.0,
        None,
    );
    if let Some(mono) = fonts.mono.as_ref() {
        let fs = (14.0 * s).round() as u16;
        draw_text_ex(
            style::MASTHEAD_LEFT,
            44.0 * s,
            62.0 * s,
            TextParams {
                font: Some(mono),
                font_size: fs,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: style::FADED,
            },
        );
        let rdims = measure_text(style::MASTHEAD_RIGHT, Some(mono), fs, 1.0);
        draw_text_ex(
            style::MASTHEAD_RIGHT,
            pw - 44.0 * s - rdims.width,
            62.0 * s,
            TextParams {
                font: Some(mono),
                font_size: fs,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: style::FADED,
            },
        );
    }

    // dateline: thick + thin rule under the masthead
    draw_line(
        40.0 * s,
        88.0 * s,
        pw - 40.0 * s,
        88.0 * s,
        2.5 * s,
        style::INK,
    );
    draw_line(
        40.0 * s,
        93.0 * s,
        pw - 40.0 * s,
        93.0 * s,
        1.0 * s,
        style::INK,
    );
}
