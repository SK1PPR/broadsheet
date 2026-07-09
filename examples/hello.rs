//! The smallest useful broadsheet movie — copy this file to start yours.
//!
//! ```sh
//! cargo run --example hello                      # live preview
//! cargo run --example hello -- --record out/hi   # render out/hi/out.mp4
//! ```

use broadsheet::prelude::*;
use broadsheet::widgets;

fn main() {
    let mut m = Movie::new("Hello, Broadsheet", 1280, 720);
    m.set_theme(Theme::plain()); // or broadsheet() / midnight() / your own

    // declare the cast (the world at t = 0)
    let arr = widgets::array(&mut m, "arr", &["2", "7", "1", "9", "4"], v(640., 340.));
    m.scene()
        .text("cap", v(640., 560.), "find the max: keep the best so far")
        .size(24.)
        .hidden();

    // script the beats (the cursor advances with each play)
    m.play(act().fade_in("cap"));
    let mut best = 0;
    for i in 0..arr.n {
        m.play(act().highlight(&arr.cell(i), m.role(Role::Active)).dur(0.5));
        if ["2", "7", "1", "9", "4"][i] > ["2", "7", "1", "9", "4"][best] {
            best = i;
        }
    }
    m.play(act().color_to(&arr.cell(best), m.role(Role::Found)));
    m.play(act().pulse(&arr.cell(best)));
    m.wait(1.0);

    broadsheet::run(m);
}
