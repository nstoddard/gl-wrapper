use glow::HasContext;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext, WebGlContextAttributes};

use super::framebuffer::*;
use super::mesh::*;
use super::program::*;
use super::rect::*;
use super::surface::*;
use super::texture::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::glfw::*;

/// An OpenGL context.
#[derive(Clone)]
pub struct GlContext {
    inner: Rc<RefCell<glow::Context>>,
    pub(crate) cache: Rc<RefCell<GlContextCache>>,
    // A VBO that is currently used for all instanced rendering
    // TODO: this isn't suitable for all cases of instanced rendering; some apps will want to
    // use static data for the instances rather than recreating them each frame.
    pub(crate) instanced_vbo: GlBuffer,
}

pub(crate) struct GlContextCache {
    pub draw_mode: Option<DrawMode>,
    pub bound_program: Option<ProgramId>,
    pub bound_framebuffer: Option<FramebufferId>,
    pub bound_read_framebuffer: Option<FramebufferId>,
    pub bound_textures: [Option<(u32, TextureId)>; 32],
}

impl GlContextCache {
    fn new() -> Self {
        Self {
            draw_mode: None,
            bound_program: None,
            bound_framebuffer: None,
            bound_read_framebuffer: None,
            bound_textures: [None; 32],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum GlFlag {
    DepthTest,
    CullFace,
}

impl GlFlag {
    fn as_gl(self) -> u32 {
        match self {
            GlFlag::DepthTest => glow::DEPTH_TEST,
            GlFlag::CullFace => glow::CULL_FACE,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type EventReceiver = std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>;

impl GlContext {
    /// Creates a `GlContext` and associated surface.
    ///
    /// Returns an error if the context couldn't be created.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        window_mode: WindowMode,
        grab_cursor: bool,
        debug_context: bool,
    ) -> Result<(Self, ScreenSurface, EventReceiver), &'static str> {
        let mut glfw = get_glfw();
        let (mut window, event_receiver) =
            create_window_inner(&mut glfw, &window_mode, grab_cursor, debug_context);

        let context =
            unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s)) };

        let screen_surface = ScreenSurface::new(window, window_mode, grab_cursor);

        Ok((Self::new_inner(context, debug_context), screen_surface, event_receiver))
    }

    /// Creates a `GlContext` and associated surface.
    ///
    /// Returns an error if the context couldn't be created.
    #[cfg(target_arch = "wasm32")]
    pub fn new(canvas_id: &str) -> Result<(Self, ScreenSurface), &'static str> {
        let document = window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .expect("Unable to find canvas element")
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let context = glow::Context::from_webgl2_context(
            canvas
                .get_context_with_context_options(
                    "webgl2",
                    WebGlContextAttributes::new().antialias(true).as_ref(),
                )
                .expect("Unable to create canvas")
                .ok_or("Unable to create canvas")?
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap(),
        );
        Ok((Self::new_inner(context, false), ScreenSurface::new(canvas)))
    }

    fn new_inner(context: glow::Context, debug_context: bool) -> Self {
        unsafe {
            context.enable(glow::BLEND);
            context.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            context.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            let instanced_vbo = context.create_buffer().unwrap();
            context.bind_buffer(glow::ARRAY_BUFFER, Some(instanced_vbo));

            if debug_context {
                context.enable(glow::DEBUG_OUTPUT);
                context.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
                context.debug_message_control(
                    glow::DONT_CARE,
                    glow::DONT_CARE,
                    glow::DONT_CARE,
                    &[],
                    true,
                );
                context.debug_message_callback(debug_callback);
            }

            GlContext {
                inner: Rc::new(RefCell::new(context)),
                cache: Rc::new(RefCell::new(GlContextCache::new())),
                instanced_vbo,
            }
        }
    }

    // TODO: sometimes this function is called multiple times in a row; avoid that when possible
    pub(crate) fn inner(&self) -> std::cell::RefMut<glow::Context> {
        self.inner.borrow_mut()
    }

    /// Sets the viewport. This is primarily intended to be used by the `Surface` trait.
    pub fn viewport(&self, viewport: &Rect<i32>) {
        unsafe {
            self.inner().viewport(
                viewport.start.x,
                viewport.start.y,
                viewport.end.x - viewport.start.x,
                viewport.end.y - viewport.start.y,
            );
        }
    }

    pub(crate) fn enable(&self, flag: GlFlag) {
        unsafe {
            self.inner().enable(flag.as_gl());
        }
    }

    pub(crate) fn disable(&self, flag: GlFlag) {
        unsafe {
            self.inner().disable(flag.as_gl());
        }
    }

    pub fn check_for_errors(&self) {
        let err = unsafe { self.inner().get_error() };
        if err != 0 {
            panic!("OpenGL error: {}", err);
        }
    }
}

fn debug_callback(source: u32, typ: u32, _: u32, severity: u32, message: &str) {
    let source = match source {
        glow::DEBUG_SOURCE_API => "API",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window system",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "Shader compiler",
        glow::DEBUG_SOURCE_THIRD_PARTY => "Third party",
        glow::DEBUG_SOURCE_APPLICATION => "Application",
        glow::DEBUG_SOURCE_OTHER => "Other",
        _ => "(Unknown)",
    };
    let typ = match typ {
        glow::DEBUG_TYPE_ERROR => "Error",
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated behavior",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined behavior",
        glow::DEBUG_TYPE_PORTABILITY => "Portability",
        glow::DEBUG_TYPE_PERFORMANCE => "Performance",
        glow::DEBUG_TYPE_OTHER => "Other",
        _ => "(Unknown)",
    };
    let severity_str = match severity {
        glow::DEBUG_SEVERITY_HIGH => "High",
        glow::DEBUG_SEVERITY_MEDIUM => "Medium",
        glow::DEBUG_SEVERITY_LOW => "Low",
        _ => return, // Very low priority message; disregard it
    };
    let formatted = format!("{} {} {} {}", source, typ, severity_str, message);

    match severity {
        glow::DEBUG_SEVERITY_HIGH => log::error!("{}", formatted),
        glow::DEBUG_SEVERITY_MEDIUM => log::warn!("{}", formatted),
        glow::DEBUG_SEVERITY_LOW => log::info!("{}", formatted),
        _ => log::info!("{}", formatted),
    }
}
