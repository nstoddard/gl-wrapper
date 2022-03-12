use super::context::*;
use cgmath::*;
use glow::HasContext;
#[cfg(not(target_arch = "wasm32"))]
use image::DynamicImage;
#[cfg(not(target_arch = "wasm32"))]
use image::GenericImageView;
use uid::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlImageElement;

#[doc(hidden)]
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct TextureId_(());

pub type TextureId = Id<TextureId_>;

type GlTexture = <glow::Context as HasContext>::Texture;

// TODO: TextureFormat should support other formats such as U8U8U8
#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    Red,
    RGB,
    RGBA,
    SRGB,
    SRGBA,
}

impl TextureFormat {
    pub fn to_gl_internal_format(self) -> u32 {
        match self {
            TextureFormat::Red => glow::R8,
            TextureFormat::RGB => glow::RGB8,
            TextureFormat::RGBA => glow::RGBA8,
            TextureFormat::SRGB => glow::SRGB8,
            TextureFormat::SRGBA => glow::SRGB8_ALPHA8,
        }
    }

    pub fn to_gl_format(self) -> u32 {
        match self {
            TextureFormat::Red => glow::RED,
            TextureFormat::RGB => glow::RGB,
            TextureFormat::RGBA => glow::RGBA,
            TextureFormat::SRGB => glow::RGB,
            TextureFormat::SRGBA => glow::RGBA,
        }
    }

    pub fn is_srgb(self) -> bool {
        matches!(self, TextureFormat::SRGB | TextureFormat::SRGBA)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear,
}

impl MinFilter {
    fn as_gl(self) -> u32 {
        match self {
            MinFilter::Nearest => glow::NEAREST,
            MinFilter::Linear => glow::LINEAR,
            MinFilter::NearestMipmapNearest => glow::NEAREST_MIPMAP_NEAREST,
            MinFilter::NearestMipmapLinear => glow::NEAREST_MIPMAP_LINEAR,
            MinFilter::LinearMipmapNearest => glow::LINEAR_MIPMAP_NEAREST,
            MinFilter::LinearMipmapLinear => glow::LINEAR_MIPMAP_LINEAR,
        }
    }

    pub fn has_mipmap(self) -> bool {
        match self {
            MinFilter::Nearest => false,
            MinFilter::Linear => false,
            MinFilter::NearestMipmapNearest => true,
            MinFilter::NearestMipmapLinear => true,
            MinFilter::LinearMipmapNearest => true,
            MinFilter::LinearMipmapLinear => true,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MagFilter {
    Nearest,
    Linear,
}

impl MagFilter {
    fn as_gl(self) -> u32 {
        match self {
            MagFilter::Nearest => glow::NEAREST,
            MagFilter::Linear => glow::LINEAR,
        }
    }
}
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum WrapMode {
    ClampToEdge,
    Repeat,
}

impl WrapMode {
    fn as_gl(self) -> u32 {
        match self {
            WrapMode::ClampToEdge => glow::CLAMP_TO_EDGE,
            WrapMode::Repeat => glow::REPEAT,
        }
    }
}

/// A 2D texture.
pub struct Texture2d {
    pub texture: GlTexture,
    pub size: Vector2<u32>,
    id: TextureId,
    pub context: GlContext,
    is_srgb: bool,
}

impl Drop for Texture2d {
    fn drop(&mut self) {
        unsafe {
            self.context.inner().delete_texture(self.texture);
        }
    }
}

impl Texture2d {
    /// Creates an empty `Texture2d`. Should typically be rendered to with a `Framebuffer`.
    pub fn empty(
        context: &GlContext,
        size: Vector2<u32>,
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        // TODO: add a method to generate mipmaps after data has been written to the texture
        assert!(!min_filter.has_mipmap());

        let texture = unsafe {
            let texture = context.inner().create_texture().unwrap();
            context.inner().bind_texture(glow::TEXTURE_2D, Some(texture));
            context.cache.borrow_mut().clear_bound_textures();
            context.inner().tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                size.x as i32,
                size.y as i32,
                0,
                format.to_gl_format(),
                glow::UNSIGNED_BYTE,
                None,
            );
            texture
        };
        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size,
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    /// Creates a `Texture2d` from an `HtmlImageElement`.
    #[cfg(target_arch = "wasm32")]
    pub fn from_image(
        context: &GlContext,
        image: &HtmlImageElement,
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = unsafe {
            let texture = context.inner().create_texture().unwrap();
            context.inner().bind_texture(glow::TEXTURE_2D, Some(texture));
            context.cache.borrow_mut().clear_bound_textures();
            context.inner().tex_image_2d_with_html_image(
                glow::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                format.to_gl_format(),
                glow::UNSIGNED_BYTE,
                image,
            );
            texture
        };

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size: vec2(image.width(), image.height()),
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    /// Creates a `Texture2d` from a `DynamicImage`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_image(
        context: &GlContext,
        image: &DynamicImage,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let (width, height) = image.dimensions();
        let size = vec2(width, height);

        let format = match image {
            DynamicImage::ImageRgb8(_) => TextureFormat::SRGB,
            DynamicImage::ImageRgba8(_) => TextureFormat::SRGBA,
            _ => todo!("Only RGB and RGBA images are currently supported"),
        };

        Self::from_data(context, size, &image.to_bytes(), format, min_filter, mag_filter, wrap_mode)
    }

    /// Creates a `Texture2d` from data.
    pub fn from_data(
        context: &GlContext,
        size: Vector2<u32>,
        data: &[u8],
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = unsafe {
            let texture = context.inner().create_texture().unwrap();
            context.inner().bind_texture(glow::TEXTURE_2D, Some(texture));
            context.cache.borrow_mut().clear_bound_textures();
            context.inner().tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                size.x as i32,
                size.y as i32,
                0,
                format.to_gl_format(),
                glow::UNSIGNED_BYTE,
                Some(data),
            );
            texture
        };

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size,
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    pub fn set_contents(&self, format: TextureFormat, data: &[u8]) {
        // TODO: remove texture unit parameter
        self.bind(0);
        unsafe {
            self.context.inner().tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                self.size.x as i32,
                self.size.y as i32,
                format.to_gl_format(),
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );
        }
    }

    pub fn set_partial_contents(
        &self,
        format: TextureFormat,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        data: &[u8],
    ) {
        self.bind(0);
        unsafe {
            self.context.inner().tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                xoffset,
                yoffset,
                width,
                height,
                format.to_gl_format(),
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );
        }
    }

    fn set_tex_parameters(
        context: &GlContext,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) {
        unsafe {
            context.inner().tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                min_filter.as_gl() as i32,
            );
            context.inner().tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                mag_filter.as_gl() as i32,
            );
            context.inner().tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                wrap_mode.as_gl() as i32,
            );
            context.inner().tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                wrap_mode.as_gl() as i32,
            );

            if min_filter.has_mipmap() {
                context.inner().generate_mipmap(glow::TEXTURE_2D);
            }
        }
    }

    pub fn bind(&self, texture_unit: u32) {
        let mut cache = self.context.cache.borrow_mut();
        if cache.bound_textures[texture_unit as usize] != Some((glow::TEXTURE_2D, self.id)) {
            cache.bound_textures[texture_unit as usize] = Some((glow::TEXTURE_2D, self.id));
            unsafe {
                self.context.inner().active_texture(glow::TEXTURE0 + texture_unit);
                self.context.inner().bind_texture(glow::TEXTURE_2D, Some(self.texture));
            }
        }
    }

    /// True if the image uses an sRGB format.
    pub fn is_srgb(&self) -> bool {
        self.is_srgb
    }
}
