#![cfg(not(target_arch = "wasm32"))]

use crate::gl::*;
use glfw::Context as GlfwContext;
use glfw::Glfw;
use std::sync::mpsc::Receiver;

thread_local!(static GLOBAL_GLFW: Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap());

pub fn get_glfw() -> Glfw {
    GLOBAL_GLFW.with(|glfw| glfw.clone())
}

fn set_window_hints(glfw: &mut Glfw, debug_context: bool) {
    glfw.window_hint(glfw::WindowHint::Visible(false));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(debug_context));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4))); // TODO: make this configurable
    glfw.window_hint(glfw::WindowHint::Resizable(true));

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
}

pub fn create_window_inner(
    glfw: &mut Glfw,
    window_mode: &WindowMode,
    grab_cursor: bool,
    debug_context: bool,
) -> (glfw::Window, Receiver<(f64, glfw::WindowEvent)>) {
    set_window_hints(glfw, debug_context);
    glfw.with_primary_monitor(|glfw, m| {
        let monitor = m.expect("Failed to find primary monitor.");
        let mode = monitor.get_video_mode().expect("Failed to get video mode (1).");
        let mut res = match *window_mode {
            WindowMode::Fullscreen => glfw
                .create_window(mode.width, mode.height, "", glfw::WindowMode::FullScreen(monitor))
                .expect("Failed to create GLFW window."),
            WindowMode::Windowed(size, ref title) => {
                let (mut window, events) = glfw
                    .create_window(size.x, size.y, title, glfw::WindowMode::Windowed)
                    .expect("Failed to create GLFW window.");
                let (posx, posy) = ((mode.width - size.x) / 2, (mode.height - size.y) / 2);
                window.set_pos(posx as i32, posy as i32);
                (window, events)
            }
        };

        let window = &mut res.0;
        if !window.is_visible() {
            window.show();
        }
        window.make_current();
        // TODO: see if vsync can be made to work
        glfw.set_swap_interval(glfw::SwapInterval::None);
        window.set_all_polling(true);
        window.set_cursor_mode(if grab_cursor {
            glfw::CursorMode::Disabled
        } else {
            glfw::CursorMode::Normal
        });

        res
    })
}

pub fn update_window_mode(window: &mut glfw::Window, window_mode: &WindowMode) {
    get_glfw().with_primary_monitor_mut(|_glfw, m| {
        let monitor = m.expect("Failed to find primary monitor.");
        let mode = monitor.get_video_mode().expect("Failed to get video mode (2).");
        match *window_mode {
            WindowMode::Fullscreen => window.set_monitor(
                glfw::WindowMode::FullScreen(monitor),
                0,
                0,
                mode.width,
                mode.height,
                None,
            ),
            // TODO: update the window title
            WindowMode::Windowed(size, ref _title) => {
                let (posx, posy) = ((mode.width - size.x) / 2, (mode.height - size.y) / 2);
                window.set_monitor(
                    glfw::WindowMode::Windowed,
                    posx as i32,
                    posy as i32,
                    size.x,
                    size.y,
                    None,
                );
            }
        }
    });
}
