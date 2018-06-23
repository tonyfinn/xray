extern crate opengl_graphics;
extern crate spinning_square;

use opengl_graphics::{ OpenGL };
use spinning_square::{build_app, build_window, run_loop};

fn main() {
    let opengl = OpenGL::V3_2;
    let size = [200, 200];

    let mut window = build_window(size, opengl);
    let mut app = build_app(opengl);
    run_loop(app, window);
}