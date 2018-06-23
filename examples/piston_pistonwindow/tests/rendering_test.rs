extern crate xray;
extern crate spinning_square;
extern crate piston;
extern crate opengl_graphics;

use opengl_graphics::{ OpenGL };
use spinning_square::{build_app, build_window, run_loop};
use piston::input::{UpdateArgs,RenderArgs};
use piston::window::{Size,Window};
use xray::gl_screenshot_test;

fn render_args_for_window<W: Window>(window: &W, simulated_dt: f64) -> RenderArgs {
    let Size { width: draw_width, height: draw_height } = window.draw_size();
    let Size { width, height } = window.size();
    RenderArgs {
        ext_dt: simulated_dt,
        width,
        height,
        draw_width,
        draw_height
    }
}

#[test]
fn test_initial_render_should_pass() {
    let opengl = OpenGL::V3_2;
    let size = [200, 200];

    let mut window = build_window(size, opengl);
    let mut app = build_app(opengl);
    let Size { width: draw_width, height: draw_height } = window.draw_size();
    app.render(&render_args_for_window(&window, 0.0));
    gl_screenshot_test("initial_render", 0, 0, draw_width, draw_height);
}

#[test]
fn test_update_should_match() {
    let opengl = OpenGL::V3_2;
    let size = [200, 200];

    let mut window = build_window(size, opengl);
    let mut app = build_app(opengl);
    let Size { width: draw_width, height: draw_height } = window.draw_size();
    app.update(&UpdateArgs { dt: 2.0 });
    app.render(&render_args_for_window(&window, 0.0));
    gl_screenshot_test("update_correct", 0, 0, draw_width, draw_height);
}

#[test]
fn test_update_should_not_match() {
    let opengl = OpenGL::V3_2;
    let size = [200, 200];

    let mut window = build_window(size, opengl);
    let mut app = build_app(opengl);
    let Size { width: draw_width, height: draw_height } = window.draw_size();
    app.update(&UpdateArgs { dt: 2.0 });
    app.render(&render_args_for_window(&window, 0.0));
    gl_screenshot_test("update_incorrect", 0, 0, draw_width, draw_height);
}