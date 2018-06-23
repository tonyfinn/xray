# XRay

Screenshot testing for Rust games

## Features

* Compares screenshots taken at test time with reference screenshots.
* Outputs the actual screenshot taken and a image containing only those pixels which differ.
* Compatible with OpenGL apps

Example test (for a Piston + OpenGL app):

```rust
#[test]
fn check_basic_screen() {
    let size = [1280, 720];
    let mut app = App::new(size, build_glutin_window(size));
    let Size { width: draw_width, height: draw_height } = app.window.draw_size();
    let Size { width, height } = app.window.size();
    app.render_into_viewport(Viewport {
        rect: [0, 0, draw_width as i32, draw_height as i32],
        window_size: [width, height],
        draw_size: [draw_width, draw_height]
    });
    xray::screenshot_test("basic_rendering/initial_map", 0, 0, draw_width, draw_height);
}
```

## Usage

1. Write your test.
2. Run your test.
3. The first time the test will fail, as there is no reference screenshot. The actual screenshot taken
   during the test will be stored at `test_output/<test_name>/actual.png`.
4. Verify the generated screenshot is correct.
5. Copy the generated screenshot to `references/<test_name>.png`
6. Continue development.

## Known Issues

* Linux/X11: You should run the tests in single threaded mode. Since each test will be creating X11
  windows to render into, and creating multiple windows too quickly will result in some of them
  failing to obtain an input method. This may be possible to fix by wrapping the new window creation in
  a mutex, but I have't had time to investigate.