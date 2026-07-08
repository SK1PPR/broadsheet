//! The runtime: live preview window with transport controls, or offline
//! recording, both driving the same pure `Timeline::apply(base, t)`.
//!
//! Every frame renders into an offscreen render target at the output
//! resolution; live mode blits it to the window (fit, centred, optional
//! grain), record mode reads the pixels back. Window size never affects
//! recorded output.
//!
//! Live controls: `Space` pause, `←/→` frame step, `,`/`.` ±1 s, `1`–`9`
//! section jump, `F` / `Ctrl+Cmd+F` fullscreen, `R` restart, drag bottom bar to scrub.
//! The HUD is live-only.
//!
//! CLI flags (after `--`):
//! - `--record [dir]`  render offline (default sink: ffmpeg pipe → out.mp4)
//! - `--fps N`         output frame rate (default 60)
//! - `--scale F`       supersampling (default 1.5 recorded → 1080p, 1 live)
//! - `--from S --to S` record a time range (clips for social posts)
//! - `--frames N`      hard frame cap (smoke tests)
//! - `--still S`       export the single frame at time S as PNG and exit
//! - `--alpha`         transparent background, no chrome, PNG sequence
//! - `--png`           force PNG sequence instead of the ffmpeg pipe
//! - `--gif`           pipe frames into out.gif instead of out.mp4
//! - `--grain`         newsprint grain + vignette post-process

use macroquad::prelude::*;

use crate::movie::Movie;
#[cfg(not(target_arch = "wasm32"))]
use crate::record::Recorder;
use crate::render::{self, View};
use crate::style::{self, Fonts};

/// JS interop for web builds. miniquad's `mq_js_bundle.js` exposes these as
/// `wasm_exports.bs_*`, letting the host page drive the stateless timeline
/// (scroll-scrub, play/pause) without any bindgen layer.
#[cfg(target_arch = "wasm32")]
mod web {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // f32s smuggled through AtomicU32 bit patterns; NaN = "no seek pending".
    pub static SEEK_BITS: AtomicU32 = AtomicU32::new(u32::MAX);
    pub static TIME_BITS: AtomicU32 = AtomicU32::new(0);
    pub static DUR_BITS: AtomicU32 = AtomicU32::new(0);
    pub static PAUSED: AtomicBool = AtomicBool::new(true);

    pub fn take_seek() -> Option<f32> {
        let t = f32::from_bits(SEEK_BITS.swap(u32::MAX, Ordering::Relaxed));
        (!t.is_nan()).then_some(t)
    }

    #[no_mangle]
    pub extern "C" fn bs_seek(t: f32) {
        SEEK_BITS.store(t.to_bits(), Ordering::Relaxed);
    }

    #[no_mangle]
    pub extern "C" fn bs_set_paused(p: i32) {
        PAUSED.store(p != 0, Ordering::Relaxed);
    }

    #[no_mangle]
    pub extern "C" fn bs_duration() -> f32 {
        f32::from_bits(DUR_BITS.load(Ordering::Relaxed))
    }

    #[no_mangle]
    pub extern "C" fn bs_time() -> f32 {
        f32::from_bits(TIME_BITS.load(Ordering::Relaxed))
    }
}

pub(crate) struct Opts {
    pub record: Option<String>,
    pub fps: u32,
    pub max_frames: Option<u32>,
    pub scale: f32,
    pub still: Option<f32>,
    pub from: f32,
    pub to: Option<f32>,
    pub alpha: bool,
    pub png: bool,
    pub gif: bool,
    pub grain: bool,
    /// Opaque paper background with no page chrome (web embedding).
    pub bare: bool,
}

pub(crate) fn parse_opts() -> Opts {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = Opts {
        record: None,
        fps: 60,
        max_frames: None,
        scale: 0.0,
        still: None,
        from: 0.0,
        to: None,
        alpha: false,
        png: false,
        gif: false,
        grain: false,
        bare: false,
    };
    let mut i = 1;
    let value = |args: &[String], i: usize, flag: &str| -> String {
        args.get(i + 1)
            .unwrap_or_else(|| panic!("{flag} expects a value"))
            .clone()
    };
    while i < args.len() {
        match args[i].as_str() {
            "--record" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    opts.record = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    opts.record = Some("frames".into());
                }
            }
            "--fps" => {
                opts.fps = value(&args, i, "--fps").parse().expect("--fps: number");
                i += 1;
            }
            "--scale" => {
                opts.scale = value(&args, i, "--scale").parse().expect("--scale: number");
                i += 1;
            }
            "--frames" => {
                opts.max_frames = Some(
                    value(&args, i, "--frames")
                        .parse()
                        .expect("--frames: number"),
                );
                i += 1;
            }
            "--still" => {
                opts.still = Some(
                    value(&args, i, "--still")
                        .parse()
                        .expect("--still: seconds"),
                );
                i += 1;
            }
            "--from" => {
                opts.from = value(&args, i, "--from").parse().expect("--from: seconds");
                i += 1;
            }
            "--to" => {
                opts.to = Some(value(&args, i, "--to").parse().expect("--to: seconds"));
                i += 1;
            }
            "--alpha" => opts.alpha = true,
            "--png" => opts.png = true,
            "--gif" => opts.gif = true,
            "--grain" => opts.grain = true,
            _ => {}
        }
        i += 1;
    }
    if opts.scale <= 0.0 {
        // web renders into a high-DPI canvas: supersample so it stays crisp
        // on retina displays instead of upscaling a 1x target.
        opts.scale = if cfg!(target_arch = "wasm32") {
            2.0
        } else if opts.record.is_some() || opts.still.is_some() {
            1.5
        } else {
            1.0
        };
    }
    opts
}

const GRAIN_VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const GRAIN_FRAG: &str = r#"#version 100
precision lowp float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 c = texture2D(Texture, uv);
    float n = fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453);
    c.rgb += (n - 0.5) * 0.04;
    float d = distance(uv, vec2(0.5, 0.5));
    c.rgb *= 1.0 - 0.15 * smoothstep(0.4, 0.9, d);
    gl_FragColor = c;
}"#;

fn fullscreen_pressed() -> bool {
    let command_down = is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
    let control_down = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
    is_key_pressed(KeyCode::F)
        || is_key_pressed(KeyCode::F11)
        || (command_down && control_down && is_key_pressed(KeyCode::F))
}

pub async fn run_loop(movie: Movie) {
    let fonts = Fonts::load();
    let (base, timeline) = movie.finalize();
    #[allow(unused_mut)]
    let mut opts = parse_opts();
    // On the web the page supplies its own chrome (masthead, rules, framing),
    // so render an opaque paper background with no chrome. Opaque (not
    // transparent) keeps it seamless without depending on canvas alpha
    // compositing, which is inconsistent across browsers.
    #[cfg(target_arch = "wasm32")]
    {
        opts.bare = true;
    }
    let (w, h) = (movie.width as f32, movie.height as f32);
    let s = opts.scale;
    let (pw, ph) = ((w * s).round(), (h * s).round());

    let rt = render_target(pw as u32, ph as u32);
    rt.texture.set_filter(FilterMode::Linear);
    let rt_cam = Camera2D {
        zoom: vec2(2.0 / pw, 2.0 / ph),
        target: vec2(pw / 2.0, ph / 2.0),
        render_target: Some(rt.clone()),
        ..Default::default()
    };
    // second target for baking the grain pass into recorded output
    let rt_post = render_target(pw as u32, ph as u32);
    rt_post.texture.set_filter(FilterMode::Linear);
    let rt_post_cam = Camera2D {
        zoom: vec2(2.0 / pw, 2.0 / ph),
        target: vec2(pw / 2.0, ph / 2.0),
        render_target: Some(rt_post.clone()),
        ..Default::default()
    };

    let grain = if opts.grain {
        load_material(
            ShaderSource::Glsl {
                vertex: GRAIN_VERT,
                fragment: GRAIN_FRAG,
            },
            MaterialParams::default(),
        )
        .map_err(|e| eprintln!("grain shader failed to compile: {e}"))
        .ok()
    } else {
        None
    };

    let render_canvas = |t: f32| {
        set_camera(&rt_cam);
        let scene = timeline.apply(&base, t);
        let view = View::from_scene(&scene, w, h, s);
        if opts.alpha {
            clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        } else if opts.bare {
            // web: paper background only, no masthead/border — the surrounding
            // page supplies the chrome, so the canvas blends into the article
            clear_background(style::PAPER);
        } else {
            render::draw_page_chrome(&movie.title, w, h, &fonts, &view);
        }
        render::draw_scene(&scene, &fonts, &view);
    };

    // grain bake: rt -> rt_post through the material; both passes flip, so
    // orientation matches the plain path
    let capture = |grain: &Option<Material>| -> Image {
        if let Some(g) = grain {
            set_camera(&rt_post_cam);
            gl_use_material(g);
            draw_texture_ex(
                &rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(pw, ph)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            set_default_camera();
            rt_post.texture.get_texture_data()
        } else {
            set_default_camera();
            rt.texture.get_texture_data()
        }
    };

    // capture feeds the still/record paths, which don't exist on web
    #[cfg(target_arch = "wasm32")]
    let _ = &capture;

    // ---- single still frame ----
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(ts) = opts.still {
        render_canvas(ts);
        next_frame().await;
        let img = capture(&grain);
        let path = format!("still_{ts:.2}.png");
        img.export_png(&path);
        println!("wrote {path} ({pw}x{ph})");
        std::process::exit(0);
    }

    // ---- offline record ----
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = opts.record.clone() {
        let mut rec = Recorder::new(
            &dir,
            opts.fps,
            pw as u32,
            ph as u32,
            opts.png || opts.alpha,
            opts.gif,
        )
        .expect("cannot create record dir");
        let end_t = opts.to.unwrap_or(timeline.dur).min(timeline.dur);
        let total = (((end_t - opts.from).max(0.0) * opts.fps as f32).ceil() as u32)
            .min(opts.max_frames.unwrap_or(u32::MAX));
        for f in 0..total {
            let t = opts.from + f as f32 / opts.fps as f32;
            render_canvas(t);
            let img = capture(&grain);
            rec.capture(&img);
            next_frame().await;
        }
        rec.finish(&movie.sections, &movie.marks);
        std::process::exit(0);
    }

    #[cfg(target_arch = "wasm32")]
    web::DUR_BITS.store(timeline.dur.to_bits(), std::sync::atomic::Ordering::Relaxed);

    // ---- live preview ----
    let mut t: f32 = 0.0;
    // web builds start paused; the host page drives playback
    let mut paused = cfg!(target_arch = "wasm32");
    let mut fullscreen = false;
    let frame_dt = 1.0 / opts.fps as f32;

    loop {
        // page-driven controls: the atomics are authoritative at frame start,
        // local key handling below may still flip `paused` for this frame
        #[cfg(target_arch = "wasm32")]
        {
            use std::sync::atomic::Ordering;
            paused = web::PAUSED.load(Ordering::Relaxed);
            if let Some(st) = web::take_seek() {
                t = st;
            }
        }

        if fullscreen_pressed() {
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

        let (sw, sh) = (screen_width(), screen_height());
        let bar_y = sh - 26.0;
        let (mx, my) = mouse_position();
        if is_mouse_button_down(MouseButton::Left) && my >= bar_y {
            paused = true;
            t = (mx / sw).clamp(0.0, 1.0) * timeline.dur;
        }

        #[cfg(target_arch = "wasm32")]
        web::PAUSED.store(paused, std::sync::atomic::Ordering::Relaxed);

        if !paused {
            t += get_frame_time();
        }
        t = t.clamp(0.0, timeline.dur);

        #[cfg(target_arch = "wasm32")]
        web::TIME_BITS.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);

        render_canvas(t);

        // blit to window: fit, centred, letterboxed
        set_default_camera();
        // web: paper letterbox so bars match the page; native: ink bars
        #[cfg(target_arch = "wasm32")]
        clear_background(style::PAPER);
        #[cfg(not(target_arch = "wasm32"))]
        clear_background(style::INK);
        let fit = (sw / pw).min(sh / ph);
        let (dw, dh) = (pw * fit, ph * fit);
        let (dx, dy) = ((sw - dw) / 2.0, (sh - dh) / 2.0);
        if let Some(g) = &grain {
            gl_use_material(g);
        }
        draw_texture_ex(
            &rt.texture,
            dx,
            dy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                ..Default::default()
            },
        );
        if grain.is_some() {
            gl_use_default_material();
        }

        // ---- HUD (never recorded, never on web — the page is the chrome) ----
        #[cfg(not(target_arch = "wasm32"))]
        {
            draw_rectangle(0.0, bar_y, sw, 26.0, style::with_opacity(style::INK, 0.85));
            draw_rectangle(0.0, bar_y, sw * (t / timeline.dur), 3.0, style::ACCENT);
            for (st, _) in &movie.sections {
                draw_rectangle(
                    sw * (st / timeline.dur) - 1.0,
                    bar_y,
                    2.0,
                    8.0,
                    style::PAPER,
                );
            }
            let frame_no = (t * opts.fps as f32).round() as u32;
            let hud = format!(
                "{}  t={:6.2}s  frame={:5}  [space] play/pause  [</>] step  [,/.] +/-1s  [1-9] sections  [F/Ctrl+Cmd+F] fullscreen  [R] restart",
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
        }

        next_frame().await;
    }
}
