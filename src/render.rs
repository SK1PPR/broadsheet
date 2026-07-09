//! The macroquad draw pass: scene → pixels, plus the newspaper page chrome.
//!
//! New primitive = match arm in [`draw_entity`]. All world coordinates flow
//! through [`View::xform`]: supersampling scale + 2D camera today, a 3D
//! projection later.

use macroquad::prelude::*;

use crate::primitives::{Align, Entity, FontKind, Shape};
use crate::scene::Scene;
use crate::style::{self, Fonts};

/// World (logical) → output (physical) transform: supersampling factor `ss`
/// plus the animatable 2D camera (`cam` centre, `zoom` factor).
#[derive(Debug, Clone, Copy)]
pub struct View {
    pub ss: f32,
    pub cam: Vec2,
    pub zoom: f32,
    /// Logical canvas centre; the camera zooms about this after recentering.
    pub center: Vec2,
}

impl View {
    /// Identity camera at supersampling factor `ss` for a `w`×`h` canvas.
    pub fn neutral(w: f32, h: f32, ss: f32) -> View {
        let center = Vec2::new(w / 2.0, h / 2.0);
        View {
            ss,
            cam: center,
            zoom: 1.0,
            center,
        }
    }

    /// Read the camera pose from the scene's `"__cam"` entity, if present.
    pub fn from_scene(scene: &Scene, w: f32, h: f32, ss: f32) -> View {
        let mut v = View::neutral(w, h, ss);
        if let Some(cam) = scene.get(crate::movie::CAMERA_ID) {
            v.cam = cam.pos;
            v.zoom = cam.scale;
        }
        v
    }

    #[inline]
    pub fn xform(&self, p: Vec2) -> Vec2 {
        ((p - self.cam) * self.zoom + self.center) * self.ss
    }

    /// Size multiplier (camera zoom × supersampling).
    #[inline]
    pub fn k(&self) -> f32 {
        self.zoom * self.ss
    }
}

fn font_of(fonts: &Fonts, kind: FontKind) -> Option<&Font> {
    match kind {
        FontKind::Serif => fonts.serif.as_ref(),
        FontKind::Mono => fonts.mono.as_ref(),
        FontKind::MonoBold => fonts.mono_bold.as_ref(),
    }
}

// ---- paths & tracing ------------------------------------------------------

/// Draw the first `frac` (by arc length) of a polyline.
fn draw_path(pts: &[Vec2], frac: f32, width: f32, color: Color) {
    if pts.len() < 2 || frac <= 0.0 {
        return;
    }
    let total: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    let mut budget = total * frac.min(1.0);
    for w in pts.windows(2) {
        let seg = (w[1] - w[0]).length();
        if seg <= 0.0 {
            continue;
        }
        if budget >= seg {
            draw_line(w[0].x, w[0].y, w[1].x, w[1].y, width, color);
            budget -= seg;
        } else {
            let end = w[0] + (w[1] - w[0]) * (budget / seg);
            draw_line(w[0].x, w[0].y, end.x, end.y, width, color);
            return;
        }
    }
}

/// Point and unit tangent at `frac` of a polyline's arc length.
fn path_point(pts: &[Vec2], frac: f32) -> (Vec2, Vec2) {
    let total: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    let mut budget = total * frac.clamp(0.0, 1.0);
    for w in pts.windows(2) {
        let seg = (w[1] - w[0]).length();
        if seg <= 0.0 {
            continue;
        }
        if budget <= seg {
            let dir = (w[1] - w[0]) / seg;
            return (w[0] + dir * budget, dir);
        }
        budget -= seg;
    }
    let n = pts.len();
    let dir = (pts[n - 1] - pts[n - 2]).normalize_or_zero();
    (pts[n - 1], dir)
}

fn bezier_pts(from: Vec2, ctrl: Vec2, to: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let a = from.lerp(ctrl, t);
            let b = ctrl.lerp(to, t);
            a.lerp(b, t)
        })
        .collect()
}

fn circle_pts(c: Vec2, r: f32, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect()
}

/// Arrowhead sized from stroke width, at `tip`, pointing along `dir`.
fn draw_head(tip: Vec2, dir: Vec2, width: f32, color: Color) {
    if dir == Vec2::ZERO {
        return;
    }
    let head_len = 10.0 + width * 2.5;
    let head_w = head_len * 0.5;
    let base = tip - dir * head_len;
    let perp = Vec2::new(-dir.y, dir.x);
    draw_triangle(tip, base + perp * head_w, base - perp * head_w, color);
}

/// Path with an optional arrowhead riding its traced tip. The stroke stops
/// short of the tip so the head doesn't overlap it.
fn draw_stroke_path(pts: &[Vec2], frac: f32, width: f32, color: Color, arrow: bool) {
    if !arrow {
        draw_path(pts, frac, width, color);
        return;
    }
    let total: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    let drawn = total * frac;
    if drawn < 1.0 {
        return;
    }
    let (tip, dir) = path_point(pts, frac);
    let head_len = (10.0 + width * 2.5).min(drawn);
    let body_frac = frac * (1.0 - head_len / drawn.max(1e-3)).max(0.0);
    draw_path(pts, body_frac, width, color);
    draw_head(tip, dir, width, color);
}

// ---- text -------------------------------------------------------------------

fn wrap_lines(
    text: &str,
    font: Option<&Font>,
    font_size: u16,
    font_scale: f32,
    max_w: f32,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let cand = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if !cur.is_empty() && measure_text(&cand, font, font_size, font_scale).width > max_w {
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

/// Draw text at `pos` (physical pixels, physical `size`). Handles wrapping,
/// alignment, rotation and typewriter `trace`.
///
/// `raster` is the size the glyphs are rasterized at; the remaining factor up
/// to `size` is applied as a smooth vertex scale. Keeping `raster` constant
/// while the camera zooms (pass logical size × supersampling) avoids the
/// per-frame re-rasterization that makes text jitter during zooms.
#[allow(clippy::too_many_arguments)]
pub fn draw_text_block(
    text: &str,
    pos: Vec2,
    size: f32,
    raster: f32,
    color: Color,
    font: Option<&Font>,
    rot_deg: f32,
    wrap: Option<f32>,
    align: Align,
    trace: f32,
) {
    let font_size = raster.max(1.0).round() as u16;
    let font_scale = size.max(0.01) / font_size as f32;
    let lines = match wrap {
        Some(w) => wrap_lines(text, font, font_size, font_scale, w),
        None => vec![text.to_string()],
    };
    let total_chars: usize = lines.iter().map(|l| l.chars().count()).sum();
    let mut char_budget = if trace >= 1.0 {
        usize::MAX
    } else {
        (total_chars as f32 * trace.max(0.0)) as usize
    };

    let line_h = size * 1.4;
    let y0 = pos.y - line_h * (lines.len() as f32 - 1.0) / 2.0;
    for (i, line) in lines.iter().enumerate() {
        if char_budget == 0 {
            break;
        }
        let n_chars = line.chars().count();
        let shown: String = if char_budget >= n_chars {
            char_budget -= n_chars;
            line.clone()
        } else {
            let s: String = line.chars().take(char_budget).collect();
            char_budget = 0;
            s
        };
        let x = match align {
            Align::Center => {
                // anchor on the full line so typing doesn't shift the block
                let full = measure_text(line, font, font_size, font_scale);
                pos.x - full.width / 2.0
            }
            Align::Left => pos.x,
        };
        let dims = measure_text(&shown, font, font_size, font_scale);
        draw_text_ex(
            &shown,
            x,
            y0 + line_h * i as f32 + dims.offset_y / 2.0,
            TextParams {
                font,
                font_size,
                font_scale,
                font_scale_aspect: 1.0,
                rotation: rot_deg.to_radians(),
                color,
            },
        );
    }
}

// ---- entities -----------------------------------------------------------------

/// Draw one entity through `view`.
pub fn draw_entity(e: &Entity, fonts: &Fonts, view: &View) {
    if e.opacity <= 0.001 || e.id == crate::movie::CAMERA_ID {
        return;
    }
    let trace = e.trace.clamp(0.0, 1.0);
    // fills fade in as their outline is traced
    let fill = style::with_opacity(e.color, e.opacity * trace);
    let stroke_c = style::with_opacity(e.color, e.opacity);
    let outline = style::with_opacity(e.stroke.outline_color.unwrap_or(e.color), e.opacity);
    let k = view.k();
    let p = view.xform(e.pos);
    let width = e.stroke.width * k;

    match &e.shape {
        Shape::Circle { r } => {
            let r = r * e.scale * k;
            if e.stroke.fill {
                draw_circle(p.x, p.y, r, fill);
            }
            if e.stroke.outline {
                if trace >= 1.0 {
                    draw_circle_lines(p.x, p.y, r, width, outline);
                } else {
                    draw_path(&circle_pts(p, r, 64), trace, width, outline);
                }
            }
        }
        Shape::Rect { w, h } => {
            let (w, h) = (w * e.scale * k, h * e.scale * k);
            let (x, y) = (p.x - w / 2.0, p.y - h / 2.0);
            if e.stroke.fill {
                draw_rectangle(x, y, w, h, fill);
            }
            if e.stroke.outline {
                if trace >= 1.0 {
                    draw_rectangle_lines(x, y, w, h, width * 2.0, outline);
                } else {
                    let c = [
                        Vec2::new(x, y),
                        Vec2::new(x + w, y),
                        Vec2::new(x + w, y + h),
                        Vec2::new(x, y + h),
                        Vec2::new(x, y),
                    ];
                    draw_path(&c, trace, width, outline);
                }
            }
        }
        Shape::Line { to } => {
            draw_path(&[p, view.xform(*to)], trace, width * e.scale, stroke_c);
        }
        Shape::Arrow { to } => {
            draw_stroke_path(
                &[p, view.xform(*to)],
                trace,
                width * e.scale,
                stroke_c,
                true,
            );
        }
        Shape::Curve { ctrl, to, arrow } => {
            let pts = bezier_pts(p, view.xform(*ctrl), view.xform(*to), 32);
            draw_stroke_path(&pts, trace, width * e.scale, stroke_c, *arrow);
        }
        Shape::Polygon { pts } => {
            if pts.len() < 3 {
                return;
            }
            let phys: Vec<Vec2> = pts.iter().map(|&q| view.xform(q + e.pos)).collect();
            if e.stroke.fill {
                for i in 1..phys.len() - 1 {
                    draw_triangle(phys[0], phys[i], phys[i + 1], fill);
                }
            }
            if e.stroke.outline {
                let mut closed = phys.clone();
                closed.push(phys[0]);
                draw_path(&closed, trace, width, outline);
            }
        }
        Shape::Text { content, size } => {
            // rasterize at the zoom-independent size so camera zooms and
            // pulses scale glyphs smoothly instead of re-rasterizing
            draw_text_block(
                content,
                p,
                size * e.scale * k,
                size * view.ss,
                stroke_c,
                font_of(fonts, e.font),
                e.rot,
                e.wrap.map(|w| w * k),
                e.align,
                trace,
            );
        }
    }
}

/// Draw a whole scene in z-order (stable within equal z).
pub fn draw_scene(scene: &Scene, fonts: &Fonts, view: &View) {
    let mut order: Vec<usize> = (0..scene.entities.len()).collect();
    order.sort_by_key(|&i| scene.entities[i].z);
    let sticky_view = View {
        cam: view.center,
        zoom: 1.0,
        ..*view
    };
    for i in order {
        let entity = &scene.entities[i];
        draw_entity(
            entity,
            fonts,
            if entity.sticky { &sticky_view } else { view },
        );
    }
}

/// The newspaper page chrome drawn under every frame: double border,
/// masthead title, dateline rules. It lives in world coordinates, so camera
/// moves treat it as part of the page rather than as sticky UI.
pub fn draw_page_chrome(
    title: &str,
    w: f32,
    h: f32,
    fonts: &Fonts,
    view: &View,
    theme: &style::Theme,
) {
    clear_background(theme.paper);
    let k = view.k();
    let rect = |x: f32, y: f32, rw: f32, rh: f32, width: f32, color: Color| {
        let p = view.xform(Vec2::new(x, y));
        draw_rectangle_lines(p.x, p.y, rw * k, rh * k, width * k, color);
    };
    let line = |a: Vec2, b: Vec2, width: f32, color: Color| {
        let a = view.xform(a);
        let b = view.xform(b);
        draw_line(a.x, a.y, b.x, b.y, width * k, color);
    };

    rect(16.0, 16.0, w - 32.0, h - 32.0, 3.0, theme.ink);
    rect(24.0, 24.0, w - 48.0, h - 48.0, 1.0, theme.faded);

    let title_upper = title.to_uppercase();
    draw_text_block(
        &title_upper,
        view.xform(Vec2::new(w / 2.0, 58.0)),
        40.0 * k,
        40.0 * view.ss,
        theme.ink,
        fonts.serif.as_ref(),
        0.0,
        None,
        Align::Center,
        1.0,
    );
    if let Some(mono) = fonts.mono.as_ref() {
        let fs = (14.0 * view.ss).round() as u16;
        let fscale = 14.0 * k / fs as f32;
        let left = view.xform(Vec2::new(44.0, 62.0));
        draw_text_ex(
            &theme.masthead_left,
            left.x,
            left.y,
            TextParams {
                font: Some(mono),
                font_size: fs,
                font_scale: fscale,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: theme.faded,
            },
        );
        let rdims = measure_text(&theme.masthead_right, Some(mono), fs, fscale);
        let right = view.xform(Vec2::new(w - 44.0, 62.0));
        draw_text_ex(
            &theme.masthead_right,
            right.x - rdims.width,
            right.y,
            TextParams {
                font: Some(mono),
                font_size: fs,
                font_scale: fscale,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: theme.faded,
            },
        );
    }

    line(
        Vec2::new(40.0, 88.0),
        Vec2::new(w - 40.0, 88.0),
        2.5,
        theme.ink,
    );
    line(
        Vec2::new(40.0, 93.0),
        Vec2::new(w - 40.0, 93.0),
        1.0,
        theme.ink,
    );
}
