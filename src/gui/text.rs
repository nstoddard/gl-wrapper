use crate::gl::uniforms::*;
use crate::gl::*;
use cgmath::*;
use fxhash::*;
use rusttype::{self, Scale};
use std::cell::RefCell;
use std::collections::hash_map::*;
use std::iter;
use std::rc::Rc;

use super::color::*;
use super::shader_header::*;

struct TextCacheVert {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
}

impl VertexComponent for TextCacheVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.uv.add_to_mesh(f);
    }
}

impl VertexData for TextCacheVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("uv", 2)];
}

struct TextRenderVert {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
    color: Color4,
}

impl VertexComponent for TextRenderVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.uv.add_to_mesh(f);
        self.color.add_to_mesh(f);
    }
}

impl VertexData for TextRenderVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("uv", 2), ("color", 4)];
}

struct TextCacheUniforms<'a> {
    matrix: Matrix4<f32>,
    tex: &'a Texture2d,
}

struct TextCacheUniformsGl {
    matrix: Matrix4Uniform,
    tex: TextureUniform,
}

impl<'a> Uniforms for TextCacheUniforms<'a> {
    type GlUniforms = TextCacheUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.tex.set(context, self.tex, 0);
    }
}

impl GlUniforms for TextCacheUniformsGl {
    fn new(context: &GlContext, program: GlProgramId) -> Self {
        TextCacheUniformsGl {
            matrix: Matrix4Uniform::new("matrix", context, program),
            tex: TextureUniform::new("tex", context, program),
        }
    }
}

struct TextRenderUniforms<'a> {
    matrix: Matrix4<f32>,
    tex: &'a Texture2d,
}

struct TextRenderUniformsGl {
    matrix: Matrix4Uniform,
    tex: TextureUniform,
}

impl<'a> Uniforms for TextRenderUniforms<'a> {
    type GlUniforms = TextRenderUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.tex.set(context, self.tex, 0);
    }
}

impl GlUniforms for TextRenderUniformsGl {
    fn new(context: &GlContext, program: GlProgramId) -> Self {
        TextRenderUniformsGl {
            matrix: Matrix4Uniform::new("matrix", context, program),
            tex: TextureUniform::new("tex", context, program),
        }
    }
}

const CACHE_VERT_SHADER: &str = "
in vec2 pos;
in vec2 uv;

uniform mat4 matrix;

out vec2 Uv;

void main() {
    Uv = uv;
    gl_Position = matrix * vec4(pos, 0.0, 1.0);
}
";

const CACHE_FRAG_SHADER: &str = "
in vec2 Uv;

uniform sampler2D tex;

out float FragColor;

void main() {
    vec4 tex_color = texture(tex, Uv);
    FragColor = tex_color.r;
}
";

const RENDER_VERT_SHADER: &str = "
in vec2 pos;
in vec2 uv;
in vec4 color;

uniform mat4 matrix;

out vec2 Uv;
out vec4 Color;

void main() {
  gl_Position = matrix * vec4(pos, 0.0, 1.0);
  Uv = uv;
  Color = color;
}";

const RENDER_FRAG_SHADER: &str = "
in vec2 Uv;
in vec4 Color;

uniform sampler2D tex;

out vec4 FragColor;

void main() {
  vec4 tex_color = texture(tex, Uv);
  FragColor = vec4(Color.rgb, tex_color.r);
  // Premultiplied alpha
  FragColor.rgb *= FragColor.a;
}";

struct FontInner {
    size: u32,
    font: rusttype::Font<'static>,
    advance_y: i32,
    ascent: f32,
    glyphs: FxHashMap<char, CachedGlyph>,
    kerning: FxHashMap<(char, char), f32>,
    framebuffer: Framebuffer<Texture2d>,
    cur_x: u32,
    cur_y: u32,
    cache_mesh_builder: MeshBuilder<TextCacheVert, Triangles>,
    render_mesh_builder: MeshBuilder<TextRenderVert, Triangles>,
    cache_mesh: Mesh<TextCacheVert, TextCacheUniformsGl, Triangles>,
    render_mesh: Mesh<TextRenderVert, TextRenderUniformsGl, Triangles>,
    scale: Scale,
}

/// A glyph that has been generated but not yet added to the cache.
struct PendingGlyph {
    // None for whitespace
    display: Option<PendingGlyphDisplay>,
    advance_x: f32,
}

struct PendingGlyphDisplay {
    texture: Texture2d,
    left: i32,
    top: i32,
}

/// Describes how to access and properly position a glyph from the cache.
#[derive(Debug)]
struct CachedGlyph {
    display: Option<CachedGlyphDisplay>,
    advance_x: f32,
}

#[derive(Debug)]
struct CachedGlyphDisplay {
    loc: Vector2<i32>,
    size: Vector2<i32>,
    left: i32,
    top: i32,
}

impl FontInner {
    pub fn new(context: &GlContext, data: Vec<u8>, size: u32) -> Self {
        let font = rusttype::Font::try_from_vec(data).unwrap();
        let scale = Scale { x: size as f32, y: size as f32 };
        let v_metrics = font.v_metrics(scale);
        let descent = v_metrics.descent;
        let ascent = v_metrics.ascent;
        let advance_y = ascent - descent;

        let framebuffer = Framebuffer::new_with_texture(
            context,
            vec2(1024, 1024),
            TextureFormat::Red,
            MinFilter::Nearest,
            MagFilter::Nearest,
            WrapMode::ClampToEdge,
        );
        framebuffer.clear(context, &[ClearBuffer::Color(Color4::TRANSPARENT.into())]);

        // TODO: find a way to share these programs between all Font instances
        let cache_program =
            GlProgram::new_with_minimal_header(context, CACHE_VERT_SHADER, CACHE_FRAG_SHADER);
        let render_program =
            GlProgram::new_with_minimal_header(context, RENDER_VERT_SHADER, RENDER_FRAG_SHADER);
        let cache_mesh_builder = MeshBuilder::new();
        let render_mesh_builder = MeshBuilder::new();
        let cache_mesh = Mesh::new(context, &cache_program, DrawMode::Draw2D);
        let render_mesh = Mesh::new(context, &render_program, DrawMode::Draw2D);

        Self {
            size,
            font,
            advance_y: advance_y as i32,
            ascent,
            glyphs: FxHashMap::default(),
            kerning: FxHashMap::default(),
            framebuffer,
            cur_x: 0,
            cur_y: 0,
            cache_mesh_builder,
            render_mesh_builder,
            cache_mesh,
            render_mesh,
            scale,
        }
    }

    fn get_kerning(&mut self, a: char, b: char) -> f32 {
        match self.kerning.entry((a, b)) {
            Entry::Vacant(entry) => {
                let kerning = self.font.pair_kerning(self.scale, a, b);
                *entry.insert(kerning)
            }
            Entry::Occupied(entry) => *entry.get(),
        }
    }

    // Renders a glyph and returns a glyph to be added to the cache.
    fn load_glyph(&self, context: &GlContext, c: char) -> PendingGlyph {
        let glyph = self.font.glyph(c).scaled(self.scale);
        let advance_x = glyph.h_metrics().advance_width;
        let positioned = glyph.positioned(rusttype::Point { x: 0.0, y: 0.0 });

        let display = if c.is_whitespace() {
            None
        } else {
            let mut bitmap = vec![];
            positioned.draw(|_x, _y, pixel| {
                bitmap.push((pixel * 255.0) as u8);
            });
            let bounding_box = positioned.pixel_bounding_box().unwrap();
            let left = bounding_box.min.x;
            let top = bounding_box.min.y;

            // TODO: consider using glBufferSubData here
            let texture = Texture2d::from_data(
                context,
                vec2(
                    (bounding_box.max.x - bounding_box.min.x) as u32,
                    (bounding_box.max.y - bounding_box.min.y) as u32,
                ),
                &bitmap,
                TextureFormat::Red,
                MinFilter::Nearest,
                MagFilter::Nearest,
                WrapMode::ClampToEdge,
            );

            Some(PendingGlyphDisplay { texture, left, top })
        };

        PendingGlyph { display, advance_x }
    }

    fn cache_glyph(&mut self, context: &GlContext, c: char) {
        if self.glyphs.contains_key(&c) {
            return;
        }

        let glyph = self.load_glyph(context, c);
        let display = if let Some(display) = glyph.display {
            let framebuffer_size = self.framebuffer.attachment.size();
            let glyph_texture_size = display.texture.size();
            let line_out_of_space = self.cur_x + glyph_texture_size.x >= framebuffer_size.x;
            let (x, y) = if line_out_of_space {
                // Note: 1 was added to Y to try to avoid overlap between chars
                // TODO: see if there's a way to do that without the wasted space
                (0, self.cur_y + self.advance_y as u32 + 1)
            } else {
                (self.cur_x, self.cur_y)
            };
            if y >= framebuffer_size.y {
                panic!("Font cache full"); // TODO: resize the cache when this happens
            }
            // Note: 1 was added to X to try to avoid overlap between chars
            self.cur_x = x + glyph_texture_size.x + 1;
            self.cur_y = y;

            let mesh_builder = &mut self.cache_mesh_builder;
            mesh_builder.clear();
            mesh_builder.vert(TextCacheVert { pos: vec2(x as f32, y as f32), uv: vec2(0.0, 0.0) });
            mesh_builder.vert(TextCacheVert {
                pos: vec2((x + glyph_texture_size.x) as f32, y as f32),
                uv: vec2(1.0, 0.0),
            });
            mesh_builder.vert(TextCacheVert {
                pos: vec2(x as f32, (y + glyph_texture_size.y) as f32),
                uv: vec2(0.0, 1.0),
            });
            mesh_builder.vert(TextCacheVert {
                pos: vec2((x + glyph_texture_size.x) as f32, (y + glyph_texture_size.y) as f32),
                uv: vec2(1.0, 1.0),
            });
            mesh_builder.triangle(0, 1, 2);
            mesh_builder.triangle(1, 2, 3);
            self.cache_mesh.build_from(mesh_builder, MeshUsage::DynamicDraw);
            self.cache_mesh.draw(
                &self.framebuffer,
                &TextCacheUniforms {
                    matrix: ortho(
                        0.0,
                        framebuffer_size.x as f32,
                        0.0,
                        framebuffer_size.y as f32,
                        0.0,
                        1.0,
                    ),
                    tex: &display.texture,
                },
            );

            Some(CachedGlyphDisplay {
                loc: vec2(x as i32, y as i32),
                size: glyph_texture_size.cast().unwrap(),
                left: display.left,
                top: display.top,
            })
        } else {
            None
        };

        self.glyphs.insert(c, CachedGlyph { display, advance_x: glyph.advance_x });
    }

    fn get_cached_glyph(&self, c: char) -> &CachedGlyph {
        &self.glyphs[&c]
    }

    pub fn render_queued_chars(&mut self, surface: &impl Surface) {
        // TODO: merge this code with the equivalent in draw_2d
        let surface_size = surface.size();
        let matrix = Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
            * ortho(0.0, surface_size.x as f32, 0.0, surface_size.y as f32, 0.0, 1.0);

        self.render_mesh.build_from(&self.render_mesh_builder, MeshUsage::DynamicDraw);
        self.render_mesh
            .draw(surface, &TextRenderUniforms { matrix, tex: &self.framebuffer.attachment });

        self.render_mesh_builder.clear();
    }

    pub fn render_queued_chars_custom_matrix(
        &mut self,
        surface: &impl Surface,
        matrix: Matrix4<f32>,
    ) {
        self.render_mesh.build_from(&self.render_mesh_builder, MeshUsage::DynamicDraw);
        self.render_mesh
            .draw(surface, &TextRenderUniforms { matrix, tex: &self.framebuffer.attachment });

        self.render_mesh_builder.clear();
    }

    pub fn draw_string(
        &mut self,
        context: &GlContext,
        str: &str,
        loc: Point2<f32>,
        color: Color4,
        matrix: Matrix4<f32>,
    ) {
        for c in str.chars() {
            self.cache_glyph(context, c);
        }

        let mut x_pos = 0;
        for (a, b) in str.chars().zip(str.chars().skip(1).map(Some).chain(iter::once(None))) {
            self.draw_char(context, a, loc + vec2(x_pos as f32, 0.0), color, matrix);
            if let Some(b) = b {
                // TODO: remove cast, or floor/round
                x_pos += self.horiz_advance_between(a, b) as i32;
            }
        }
    }

    pub fn draw_char(
        &mut self,
        context: &GlContext,
        c: char,
        loc: Point2<f32>,
        color: Color4,
        matrix: Matrix4<f32>,
    ) {
        self.cache_glyph(context, c);
        let glyph = self.get_cached_glyph(c);
        if let Some(display) = &glyph.display {
            let loc = vec2(loc.x as f32, loc.y as f32 + self.ascent as f32);
            let framebuffer_size = self.framebuffer.attachment.size();
            let tex_start = display.loc;
            let tex_end = tex_start + display.size;
            let tex_start_x = (tex_start.x as f32) / framebuffer_size.x as f32;
            let tex_start_y = (tex_start.y as f32) / framebuffer_size.y as f32;
            let tex_end_x = (tex_end.x as f32) / framebuffer_size.x as f32;
            let tex_end_y = (tex_end.y as f32) / framebuffer_size.y as f32;
            let left = display.left as f32;
            let top = display.top as f32;
            let size: Vector2<f32> = display.size.cast().unwrap();

            let mesh_builder = &mut self.render_mesh_builder;

            let vert_a = mesh_builder.vert(TextRenderVert {
                pos: point3_to_vec2(matrix.transform_point(point3(loc.x + left, loc.y + top, 0.0))),
                uv: vec2(tex_start_x, tex_start_y),
                color,
            });
            let vert_b = mesh_builder.vert(TextRenderVert {
                pos: point3_to_vec2(matrix.transform_point(point3(
                    loc.x + left + size.x,
                    loc.y + top,
                    0.0,
                ))),
                uv: vec2(tex_end_x, tex_start_y),
                color,
            });
            let vert_c = mesh_builder.vert(TextRenderVert {
                pos: point3_to_vec2(matrix.transform_point(point3(
                    loc.x + left,
                    loc.y + top + size.y,
                    0.0,
                ))),
                uv: vec2(tex_start_x, tex_end_y),
                color,
            });
            let vert_d = mesh_builder.vert(TextRenderVert {
                pos: point3_to_vec2(matrix.transform_point(point3(
                    loc.x + left + size.x,
                    loc.y + top + size.y,
                    0.0,
                ))),
                uv: vec2(tex_end_x, tex_end_y),
                color,
            });
            mesh_builder.triangle(vert_a, vert_b, vert_c);
            mesh_builder.triangle(vert_b, vert_c, vert_d);
        }
    }

    // Note: this requires the chars to already be cached
    fn horiz_advance_between(&mut self, a: char, b: char) -> f32 {
        let kerning = self.get_kerning(a, b);
        let glyph = self.get_cached_glyph(a);
        glyph.advance_x + kerning
    }

    // Note: this requires the char to already be cached
    fn horiz_advance_after(&self, a: char) -> f32 {
        self.get_cached_glyph(a).advance_x
    }

    // Note: for a single char, this is the same as horiz_advance_after
    pub fn string_width(&mut self, context: &GlContext, str: &str) -> f32 {
        for c in str.chars() {
            self.cache_glyph(context, c);
        }
        if str.is_empty() {
            return 0.0;
        }

        let mut width = 0.0;
        let mut iterator_a = str.chars();
        let mut iterator_b = str.chars().skip(1);
        loop {
            match iterator_b.next() {
                None => {
                    width += self.horiz_advance_after(iterator_a.next().unwrap());
                    break;
                }
                Some(b) => {
                    let a = iterator_a.next().unwrap();
                    width += self.horiz_advance_between(a, b);
                }
            }
        }
        width
    }

    // TODO: change this to return Vec2<f32>, or change string_width to return i32
    pub fn string_size(&mut self, context: &GlContext, str: &str) -> Vector2<i32> {
        vec2(self.string_width(context, str) as i32, self.advance_y)
    }
}

/// A struct to render characters using a TTF font.
///
/// All distance units are pixels, from the top-left corner of the screen, unless
/// `render_queued_custom_matrix` is used.
///
/// This uses a cache to store previously rendered characters.
///
/// This is expensive to create, so try to create only one instance per font/size combination.
#[derive(Clone)]
pub struct Font {
    inner: Rc<RefCell<FontInner>>,
}

impl Font {
    /// Creates a new `Font` from a `Vec` containing the contents of a `ttf` file.
    pub fn new(context: &GlContext, data: Vec<u8>, size: u32) -> Self {
        Self { inner: Rc::new(RefCell::new(FontInner::new(context, data, size))) }
    }

    /// Renders all characters that have been drawn with `draw_string` or `draw_char`.
    ///
    /// This should typically be called once per frame to minimize the number of draw calls.
    pub fn render_queued(&self, surface: &impl Surface) {
        self.inner.borrow_mut().render_queued_chars(surface);
    }

    /// Renders all characters that have been drawn with `draw_string` or `draw_char`.
    ///
    /// This allows a matrix to be specified which will be used instead of a standard orthographic
    /// projection. This can be useful for rendering text in a game world rather than as part of a
    /// GUI.
    ///
    /// This should typically be called once per frame to minimize the number of draw calls.
    pub fn render_queued_custom_matrix(&self, surface: &impl Surface, matrix: Matrix4<f32>) {
        self.inner.borrow_mut().render_queued_chars_custom_matrix(surface, matrix);
    }

    /// Queues a string for drawing. To render all queued characters, call `render_queued_chars`.
    pub fn draw_string(&self, context: &GlContext, str: &str, loc: Point2<i32>, color: Color4) {
        self.draw_string_f32(
            context,
            str,
            point2(loc.x as f32, loc.y as f32),
            color,
            Matrix4::identity(),
        );
    }

    /// Queues a character to be drawn. To render all queued characters, call `render_queued_chars`.
    pub fn draw_char(&self, context: &GlContext, c: char, loc: Point2<i32>, color: Color4) {
        self.draw_char_f32(
            context,
            c,
            point2(loc.x as f32, loc.y as f32),
            color,
            Matrix4::identity(),
        );
    }

    /// Queues a string for drawing. To render all queued characters, call `render_queued_chars`.
    pub fn draw_string_f32(
        &self,
        context: &GlContext,
        str: &str,
        loc: Point2<f32>,
        color: Color4,
        matrix: Matrix4<f32>,
    ) {
        self.inner.borrow_mut().draw_string(context, str, loc, color, matrix);
    }

    /// Queues a character to be drawn. To render all queued characters, call `render_queued_chars`.
    pub fn draw_char_f32(
        &self,
        context: &GlContext,
        c: char,
        loc: Point2<f32>,
        color: Color4,
        matrix: Matrix4<f32>,
    ) {
        self.inner.borrow_mut().draw_char(context, c, loc, color, matrix);
    }

    /// Returns the width of a rendered string in pixels.
    pub fn string_width(&self, context: &GlContext, str: &str) -> f32 {
        self.inner.borrow_mut().string_width(context, str)
    }

    /// Returns the size of a rendered string in pixels.
    pub fn string_size(&self, context: &GlContext, str: &str) -> Vector2<i32> {
        self.inner.borrow_mut().string_size(context, str)
    }

    /// Returns the font size.
    pub fn size(&self) -> u32 {
        self.inner.borrow().size
    }

    pub fn advance_y(&self) -> i32 {
        self.inner.borrow().advance_y
    }
}

// TODO: put this somewhere else
fn point3_to_vec2(vec: Point3<f32>) -> Vector2<f32> {
    vec2(vec.x, vec.y)
}
