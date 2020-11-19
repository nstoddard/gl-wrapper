use cgmath::*;
use glow::HasContext;
use log::*;
use uid::*;

use super::context::*;
use super::rect::*;
use super::surface::*;
use super::texture::*;

type GlFramebuffer = <glow::Context as HasContext>::Framebuffer;
type GlRenderbuffer = <glow::Context as HasContext>::Renderbuffer;

#[doc(hidden)]
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub(crate) struct FramebufferId_(());

pub(crate) type FramebufferId = Id<FramebufferId_>;

/// A renderbuffer.
pub struct Renderbuffer {
    renderbuffer: GlRenderbuffer,
    size: Vector2<u32>,
    context: GlContext,
}

impl Drop for Renderbuffer {
    fn drop(&mut self) {
        unsafe {
            self.context.inner().delete_renderbuffer(self.renderbuffer);
        }
    }
}

impl Renderbuffer {
    pub fn new(context: &GlContext, size: Vector2<u32>, format: TextureFormat) -> Self {
        unsafe {
            let renderbuffer = context.inner().create_renderbuffer().unwrap();
            context.inner().bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
            let max_samples = context.inner().get_parameter_i32(glow::MAX_SAMPLES);
            let samples = max_samples; // TODO: make this configurable
            context.inner().renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                samples,
                format.to_gl_internal_format(),
                size.x as i32,
                size.y as i32,
            );
            Renderbuffer { renderbuffer, size, context: context.clone() }
        }
    }
}

/// A framebuffer attachment; either a texture or a renderbuffer.
pub trait FramebufferAttachment {
    fn size(&self) -> Vector2<u32>;

    #[doc(hidden)]
    fn attach_to_framebuffer(&self);

    #[doc(hidden)]
    fn context(&self) -> &GlContext;
}

impl FramebufferAttachment for Texture2d {
    fn size(&self) -> Vector2<u32> {
        self.size
    }

    #[doc(hidden)]
    fn attach_to_framebuffer(&self) {
        unsafe {
            self.context.inner().framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.texture),
                0,
            );
        }
    }

    #[doc(hidden)]
    fn context(&self) -> &GlContext {
        &self.context
    }
}

impl FramebufferAttachment for Renderbuffer {
    fn size(&self) -> Vector2<u32> {
        self.size
    }

    #[doc(hidden)]
    fn attach_to_framebuffer(&self) {
        unsafe {
            self.context.inner().framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(self.renderbuffer),
            );
        }
    }

    #[doc(hidden)]
    fn context(&self) -> &GlContext {
        &self.context
    }
}

/// A framebuffer.
///
/// Framebuffers currently have only one attachment, either a texture or a renderbuffer.
pub struct Framebuffer<A: FramebufferAttachment> {
    framebuffer: GlFramebuffer,
    // TODO: this shouldn't be public
    pub attachment: A,
    viewport: Rect<i32>,
    id: FramebufferId,
}

impl<A: FramebufferAttachment> Drop for Framebuffer<A> {
    fn drop(&mut self) {
        unsafe {
            self.attachment.context().inner().delete_framebuffer(self.framebuffer);
        }
    }
}

impl Framebuffer<Texture2d> {
    pub fn new_with_texture(
        context: &GlContext,
        size: Vector2<u32>,
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = Texture2d::empty(context, size, format, min_filter, mag_filter, wrap_mode);
        Self::new(context, texture)
    }
}

impl Framebuffer<Renderbuffer> {
    pub fn new_with_renderbuffer(
        context: &GlContext,
        size: Vector2<u32>,
        format: TextureFormat,
    ) -> Self {
        let renderbuffer = Renderbuffer::new(context, size, format);
        Self::new(context, renderbuffer)
    }
}

impl<A: FramebufferAttachment> Framebuffer<A> {
    pub fn new(context: &GlContext, attachment: A) -> Self {
        unsafe {
            let framebuffer = context.inner().create_framebuffer().unwrap();
            context.inner().bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            attachment.attach_to_framebuffer();

            let framebuffer_status = context.inner().check_framebuffer_status(glow::FRAMEBUFFER);
            if framebuffer_status != glow::FRAMEBUFFER_COMPLETE {
                let reason = match framebuffer_status {
                    glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "incomplete attachment",
                    glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                        "incomplete missing attachment"
                    }
                    glow::FRAMEBUFFER_UNSUPPORTED => "unsupported",
                    _ => "unknown reason",
                };
                error!("Framebuffer not complete: {}", reason);
                panic!()
            }

            let viewport =
                Rect::new(Point2::origin(), Point2::from_vec(attachment.size().cast().unwrap()));

            Framebuffer { framebuffer, attachment, viewport, id: FramebufferId::new() }
        }
    }

    /// Note: this only works if the destination framebuffer isn't multisampled.
    // TODO: add parameters to set src/dest rects
    pub fn blit_to(&self, context: &GlContext, surface: &impl Surface) {
        self.bind_read(context);
        surface.bind(context);
        let size = self.attachment.size().cast().unwrap();
        unsafe {
            context.inner().blit_framebuffer(
                0,
                0,
                size.x,
                size.y,
                0,
                0,
                size.x,
                size.y,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
        }
    }
}

impl<A: FramebufferAttachment> Surface for Framebuffer<A> {
    #[doc(hidden)]
    fn bind(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_framebuffer != Some(self.id) {
            cache.bound_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.framebuffer));
                context.viewport(&self.viewport);
            }
        }
    }

    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_read_framebuffer != Some(self.id) {
            cache.bound_read_framebuffer = Some(self.id);
            unsafe {
                context.inner().bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            }
        }
    }

    fn size(&self) -> Vector2<u32> {
        self.attachment.size()
    }
}
