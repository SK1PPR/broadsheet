//! Deterministic offline rendering: fixed-timestep PNG frame dump + ffmpeg.
//!
//! Determinism comes from ignoring the wall clock entirely: frame `f` is
//! rendered at `t = f / fps`, captured, and written before the next frame
//! begins. The output is bit-identical across runs on the same machine.

use std::path::PathBuf;
use std::process::Command;

use macroquad::prelude::get_screen_data;

/// Writes numbered PNG frames to a directory and knows how to stitch them.
pub struct Recorder {
    pub dir: PathBuf,
    pub fps: u32,
    frame: u32,
}

impl Recorder {
    /// Create the output directory (and parents) if needed.
    pub fn new(dir: impl Into<PathBuf>, fps: u32) -> std::io::Result<Recorder> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Recorder { dir, fps, frame: 0 })
    }

    /// Capture the current backbuffer as the next numbered frame.
    /// Call after all drawing for the frame, before `next_frame().await`.
    pub fn capture(&mut self) {
        let img = get_screen_data();
        let path = self.dir.join(format!("frame_{:05}.png", self.frame));
        img.export_png(path.to_str().expect("non-utf8 record path"));
        self.frame += 1;
    }

    /// Number of frames captured so far.
    pub fn frames(&self) -> u32 {
        self.frame
    }

    /// Try to stitch the frames with ffmpeg (if it is on PATH); either way,
    /// print the exact command so it can be run manually.
    pub fn finish(&self, out_name: &str) {
        let pattern = self.dir.join("frame_%05d.png");
        let out = self.dir.join(out_name);
        let args = [
            "-y",
            "-framerate",
            &self.fps.to_string(),
            "-i",
            pattern.to_str().unwrap(),
            "-c:v",
            "libx264",
            "-crf",
            "18",
            "-preset",
            "slow",
            "-pix_fmt",
            "yuv420p",
            out.to_str().unwrap(),
        ];
        println!("\n{} frames written to {}/", self.frame, self.dir.display());
        println!("stitch with:\n  ffmpeg {}\n", args.join(" "));
        match Command::new("ffmpeg").args(args).status() {
            Ok(s) if s.success() => println!("ffmpeg: wrote {}", out.display()),
            Ok(s) => eprintln!("ffmpeg exited with {s}"),
            Err(_) => println!("(ffmpeg not found on PATH — run the command above once installed)"),
        }
    }
}
