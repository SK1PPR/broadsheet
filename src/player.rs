//! The runtime: live preview window with transport controls, or offline
//! recording, both driving the same pure `Timeline::apply(base, t)`.
//!
//! Live controls:
//! - `Space`      pause / play
//! - `←` / `→`    step one frame (pauses)
//! - `,` / `.`    jump ±1 s
//! - `R`          restart
//! - `1`–`9`      jump to section markers
//! - drag the bottom bar to scrub
//!
//! The HUD (time / frame readout, scrub bar) is live-only; recorded frames
//! never contain it.

use macroquad::prelude::*;

use crate::movie::Movie;
use crate::record::Recorder;
use crate::render;
use crate::style::{self, Fonts};

/// Options parsed from CLI args
/// (`-- --record out/ --fps 60 --scale 1.5 --frames 120`).
pub(crate) struct Opts {
    pub record: Option<String>,
    pub fps: u32,
    /// Optional hard cap on frames (smoke tests).
    pub max_frames: Option<u32>,
    /// Supersampling factor: logical 1280×720 × 1.5 = 1920×1080 output.
    /// Defaults to 1.0 live, 1.5 when recording.
    pub scale: f32,
}

pub(crate) fn parse_opts() -> Opts {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = Opts {
        record: None,
        fps: 60,
        max_frames: None,
        scale: 0.0,
    };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scale" => {
                if i + 1 < args.len() {
                    opts.scale = args[i + 1].parse().expect("--scale expects a number");
                    i += 1;
                }
            }
            "--record" => {
                // optional value; defaults to "frames"
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    opts.record = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    opts.record = Some("frames".into());
                }
            }
            "--fps" => {
                if i + 1 < args.len() {
                    opts.fps = args[i + 1].parse().expect("--fps expects a number");
                    i += 1;
                }
            }
            "--frames" => {
                if i + 1 < args.len() {
                    opts.max_frames = Some(args[i + 1].parse().expect("--frames expects a number"));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if opts.scale <= 0.0 {
        opts.scale = if opts.record.is_some() { 1.5 } else { 1.0 };
    }
    opts
}

/// Run a movie: live window by default, `--record dir/` for offline frames.
/// This is what [`crate::run`] calls inside the macroquad window.
pub async fn run_loop(movie: Movie) {
    let fonts = Fonts::load();
    let (base, timeline) = movie.finalize();
    let opts = parse_opts();
    let (w, h) = (movie.width as f32, movie.height as f32);
    let s = opts.scale;

    if let Some(dir) = opts.record {
        // ---- offline: fixed timestep, wall clock ignored ----
        let mut rec = Recorder::new(&dir, opts.fps).expect("cannot create record dir");
        let total = ((timeline.dur * opts.fps as f32).ceil() as u32)
            .min(opts.max_frames.unwrap_or(u32::MAX));
        for f in 0..total {
            let t = f as f32 / opts.fps as f32;
            let scene = timeline.apply(&base, t);
            render::draw_page_chrome(&movie.title, w, h, &fonts, s);
            render::draw_scene(&scene, &fonts, s);
            rec.capture();
            next_frame().await;
        }
        rec.finish("out.mp4");
        std::process::exit(0);
    }

    // ---- live preview ----
    let mut t: f32 = 0.0;
    let mut paused = false;
    let mut fullscreen = false;
    let frame_dt = 1.0 / opts.fps as f32;

    loop {
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::Right) {
            paused = true;
            t += frame_dt;
        }
        if is_key_pressed(KeyCode::Left) {
            paused = true;
            t -= frame_dt;
        }
        if is_key_pressed(KeyCode::Period) {
            t += 1.0;
        }
        if is_key_pressed(KeyCode::Comma) {
            t -= 1.0;
        }
        if is_key_pressed(KeyCode::R) {
            t = 0.0;
        }
        let digits = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for (i, k) in digits.iter().enumerate() {
            if is_key_pressed(*k) {
                if let Some((st, _)) = movie.sections.get(i) {
                    t = *st;
                }
            }
        }

        // fit the (w*s, h*s) canvas into whatever the window/screen is:
        // scale to fit, centred, letterboxed. fit == 1.0 in a normal window.
        let (pw, ph) = (w * s, h * s);
        let (sw, sh) = (screen_width(), screen_height());
        let fit = (sw / pw).min(sh / ph);
        let cam = Camera2D {
            target: vec2(pw / 2.0, ph / 2.0),
            zoom: vec2(2.0 * fit / sw, -2.0 * fit / sh),
            ..Default::default()
        };
        set_camera(&cam);

        // scrub bar (canvas coordinates; mouse mapped through the camera)
        let bar_y = ph - 26.0;
        let m_logical = cam.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let (mx, my) = (m_logical.x, m_logical.y);
        if is_mouse_button_down(MouseButton::Left) && my >= bar_y {
            paused = true;
            t = (mx / pw).clamp(0.0, 1.0) * timeline.dur;
        }

        if !paused {
            t += get_frame_time();
        }
        t = t.clamp(0.0, timeline.dur);

        let scene = timeline.apply(&base, t);
        render::draw_page_chrome(&movie.title, w, h, &fonts, s);
        render::draw_scene(&scene, &fonts, s);

        // ---- HUD (never recorded) ----
        draw_rectangle(0.0, bar_y, pw, 26.0, style::with_opacity(style::INK, 0.85));
        draw_rectangle(0.0, bar_y, pw * (t / timeline.dur), 3.0, style::ACCENT);
        for (st, _) in &movie.sections {
            draw_rectangle(
                pw * (st / timeline.dur) - 1.0,
                bar_y,
                2.0,
                8.0,
                style::PAPER,
            );
        }
        let frame_no = (t * opts.fps as f32).round() as u32;
        let hud = format!(
            "{}  t={:6.2}s  frame={:5}  [space] play/pause  [</>] step  [,/.] +/-1s  [1-9] sections  [R] restart",
            if paused { "PAUSED " } else { "PLAYING" },
            t,
            frame_no
        );
        draw_text_ex(
            &hud,
            10.0,
            bar_y + 18.0,
            TextParams {
                font: fonts.mono.as_ref(),
                font_size: 13,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: style::PAPER,
            },
        );

        next_frame().await;
    }
}
