use cgmath::*;
use glow::HasContext;

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(not(target_arch = "wasm32"))]
use crate::glfw::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::*;

use super::context::*;
use super::framebuffer::*;
use super::rect::*;

/// A trait for things that can be rendered to.
pub trait Surface {
    /// Binds the `Surface` and sets the appropriate viewport.
    #[doc(hidden)]
    fn bind(&self, context: &GlContext);

    /// Binds the `Surface` for reading. Doesn't modify the viewport.
    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext);

    /// Clears one or more buffers.
    ///
    /// Example usage:
    /// ```
    /// surface.clear(&context, &[ClearBuffer::Color([0.0, 0.0, 0.0, 0.0])]);
    /// ```
    fn clear(&self, context: &GlContext, buffers: &[ClearBuffer]) {
        assert!(!buffers.is_empty());
        self.bind(context);

        let mut bits = 0;
        for buffer in buffers {
            bits |= buffer.as_gl();

            if let Some(color) = buffer.color() {
                unsafe {
                    context.inner().clear_color(color[0], color[1], color[2], color[3]);
                }
            }
        }

        unsafe {
            context.inner().clear(bits);
        }
    }

    /// Returns the size of the surface.
    fn size(&self) -> Vector2<u32>;

    #[inline]
    fn width(&self) -> u32 {
        self.size().x
    }

    #[inline]
    fn height(&self) -> u32 {
        self.size().y
    }
}

pub trait ClearColor {
    #[doc(hidden)]
    fn color(self) -> [f32; 4];
}

impl ClearColor for [f32; 4] {
    fn color(self) -> [f32; 4] {
        self
    }
}

#[derive(Copy, Clone)]
pub enum ClearBuffer {
    Color([f32; 4]),
    Depth,
}

impl ClearBuffer {
    fn as_gl(&self) -> u32 {
        match self {
            ClearBuffer::Color(_) => glow::COLOR_BUFFER_BIT,
            ClearBuffer::Depth => glow::DEPTH_BUFFER_BIT,
        }
    }

    fn color(&self) -> Option<[f32; 4]> {
        match self {
            ClearBuffer::Color(color) => Some(*color),
            _ => None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// A surface that represents the screen/default framebuffer.
pub struct ScreenSurface {
    viewport: Rect<i32>,
    size: Vector2<u32>,
    canvas: HtmlCanvasElement,
    id: FramebufferId,
}

#[cfg(target_arch = "wasm32")]
impl ScreenSurface {
    pub(crate) fn new(canvas: HtmlCanvasElement) -> Self {
        let viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(canvas.width() as i32, canvas.height() as i32)),
        );
        let size = vec2(canvas.width(), canvas.height());
        ScreenSurface { viewport, size, canvas, id: FramebufferId::new() }
    }

    /// Resizes the canvas.
    pub fn set_size(&mut self, context: &GlContext, new_size: Vector2<u32>) {
        self.canvas.set_width(new_size.x);
        self.canvas.set_height(new_size.y);
        self.viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(new_size.x as i32, new_size.y as i32)),
        );
        self.size = new_size;
        // Resizing requires that we also change the viewport to match
        let cache = context.cache.borrow();
        if cache.bound_framebuffer == Some(self.id) {
            context.viewport(&self.viewport);
        }
    }

    /// Returns the canvas corresponding to this surface.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
}

#[cfg(target_arch = "wasm32")]
impl Surface for ScreenSurface {
    #[doc(hidden)]
    fn bind(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_framebuffer != Some(self.id) {
            cache.bound_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            }
            context.viewport(&self.viewport);
        }
    }

    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_read_framebuffer != Some(self.id) {
            cache.bound_read_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            }
        }
    }

    fn size(&self) -> Vector2<u32> {
        self.size
    }
}

#[derive(Clone)]
pub enum WindowMode {
    Fullscreen,
    Windowed(Vector2<u32>, String),
}

impl WindowMode {
    pub fn is_windowed(&self) -> bool {
        match self {
            WindowMode::Windowed(_, _) => true,
            WindowMode::Fullscreen => false,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// A surface that represents the screen/default framebuffer.
pub struct ScreenSurface {
    pub(crate) inner: glfw::Window,
    viewport: Rect<i32>,
    window_mode: WindowMode,
    pub(crate) grab_cursor: bool,
    size: Vector2<u32>,
    id: FramebufferId,
}

#[cfg(not(target_arch = "wasm32"))]
impl ScreenSurface {
    pub(crate) fn new(window: glfw::Window, window_mode: WindowMode, grab_cursor: bool) -> Self {
        let (window_width, window_height) = window.get_framebuffer_size();
        Self {
            inner: window,
            viewport: Rect::new(Point2::origin(), point2(window_width, window_height)),
            window_mode,
            grab_cursor,
            size: vec2(window_width as u32, window_height as u32),
            id: FramebufferId::new(),
        }
    }

    /// Resizes the surface.
    pub fn set_size(&mut self, context: &GlContext, new_size: Vector2<u32>) {
        self.viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(new_size.x as i32, new_size.y as i32)),
        );
        self.size = new_size;
        // Resizing requires that we also change the viewport to match
        let cache = context.cache.borrow();
        if cache.bound_framebuffer == Some(self.id) {
            context.viewport(&self.viewport);
        }
    }

    pub fn close_window(&mut self) {
        self.inner.set_should_close(true);
    }

    pub fn window_mode(&self) -> &WindowMode {
        &self.window_mode
    }

    pub fn set_window_mode(&mut self, window_mode: WindowMode) {
        update_window_mode(&mut self.inner, &window_mode);
        self.window_mode = window_mode;
    }

    pub fn get_grab_cursor(&self) -> bool {
        self.grab_cursor
    }

    pub fn set_grab_cursor(&mut self, grab_cursor: bool) {
        self.grab_cursor = grab_cursor;
        self.inner.set_cursor_mode(if grab_cursor {
            glfw::CursorMode::Disabled
        } else {
            glfw::CursorMode::Normal
        });
    }

    /// Takes a screenshot and saves it to the given path, or
    /// screenshots/screenshot-<date and time>.png if None.
    pub fn take_screenshot(&self, context: &GlContext, path: Option<PathBuf>, include_alpha: bool) {
        crate::screenshot::take_screenshot(context, self, path, include_alpha);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Surface for ScreenSurface {
    #[doc(hidden)]
    fn bind(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_framebuffer != Some(self.id) {
            cache.bound_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            }
            context.viewport(&self.viewport);
        }
    }

    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_read_framebuffer != Some(self.id) {
            cache.bound_read_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            }
        }
    }

    fn size(&self) -> Vector2<u32> {
        self.size
    }
}
