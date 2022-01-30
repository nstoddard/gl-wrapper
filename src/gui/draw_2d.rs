use crate::gl::uniforms::*;
use crate::gl::*;
use cgmath::*;
use std::ops::Neg;

use super::color::*;
use super::shader_header::*;

#[repr(C)]
pub struct PlainVert {
    pub pos: Point2<f32>,
    pub color: Color4,
}

impl VertexData for PlainVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("color", 4)];
}

impl VertexComponent for PlainVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.color.add_to_mesh(f);
    }
}

pub struct PlainUniforms {
    pub matrix: Matrix4<f32>,
    pub color: Color4,
}

pub struct PlainUniformsGl {
    matrix: Matrix4Uniform,
    color: Color4Uniform,
}

impl Uniforms for PlainUniforms {
    type GlUniforms = PlainUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.color.set(context, &self.color, false);
    }
}

impl GlUniforms for PlainUniformsGl {
    fn new(context: &GlContext, program: GlProgramId) -> Self {
        let matrix = Matrix4Uniform::new("matrix", context, program);
        let color = Color4Uniform::new("uniColor", context, program);
        PlainUniformsGl { matrix, color }
    }
}

#[repr(C)]
pub struct ImageVert {
    pub pos: Point2<f32>,
    pub uv: Point2<f32>,
    pub color: Color4,
}

impl VertexData for ImageVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("uv", 2), ("color", 4)];
}

impl VertexComponent for ImageVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.uv.add_to_mesh(f);
        self.color.add_to_mesh(f);
    }
}

pub struct ImageUniforms<'a> {
    pub matrix: Matrix4<f32>,
    pub color: Color4,
    pub tex: &'a Texture2d,
}

pub struct ImageUniformsGl {
    matrix: Matrix4Uniform,
    color: Color4Uniform,
    tex: TextureUniform,
}

impl<'a> Uniforms for ImageUniforms<'a> {
    type GlUniforms = ImageUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.color.set(context, &self.color, false);
        gl_uniforms.tex.set(context, self.tex, 0);
    }
}

impl GlUniforms for ImageUniformsGl {
    fn new(context: &GlContext, program: GlProgramId) -> Self {
        let matrix = Matrix4Uniform::new("matrix", context, program);
        let color = Color4Uniform::new("uniColor", context, program);
        let tex = TextureUniform::new("tex", context, program);
        ImageUniformsGl { matrix, color, tex }
    }
}

/// Contains OpenGL programs used by `Draw2d`
///
/// This is expensive to create, so try to only create one of them.
pub struct Draw2dPrograms {
    pub plain_program: GlProgram<PlainVert, PlainUniformsGl>,
    pub image_program_srgb: GlProgram<ImageVert, ImageUniformsGl>,
    pub image_program_linear: GlProgram<ImageVert, ImageUniformsGl>,
}

impl Draw2dPrograms {
    pub fn new(context: &GlContext) -> Self {
        let plain_program: GlProgram<PlainVert, PlainUniformsGl> = GlProgram::new_with_header(
            context,
            include_str!("shaders/plain_vert.glsl"),
            include_str!("shaders/plain_frag.glsl"),
            true,
        );
        let image_program_srgb: GlProgram<ImageVert, ImageUniformsGl> = GlProgram::new_with_header(
            context,
            include_str!("shaders/image_vert.glsl"),
            include_str!("shaders/image_frag.glsl"),
            true,
        );
        let image_program_linear: GlProgram<ImageVert, ImageUniformsGl> =
            GlProgram::new_with_header(
                context,
                include_str!("shaders/image_vert.glsl"),
                include_str!("shaders/image_frag.glsl"),
                false,
            );
        Self { plain_program, image_program_srgb, image_program_linear }
    }
}

/// A struct for drawing simple 2D shapes.
///
/// All distance units are pixels, from the top-left corner of the screen, unless
/// `render_queued_custom_matrix` is used.
///
// TODO: this struct may not be needed; many of the methods here could be in the impl for
// `MeshBuilder<PlainVert, Triangles>`
pub struct Draw2d {
    triangle_mesh_builder: MeshBuilder<PlainVert, Triangles>,
    triangle_mesh: Mesh<PlainVert, PlainUniformsGl, Triangles>,
    image_mesh_builder: MeshBuilder<ImageVert, Triangles>,
    image_mesh_srgb: Mesh<ImageVert, ImageUniformsGl, Triangles>,
    image_mesh_linear: Mesh<ImageVert, ImageUniformsGl, Triangles>,
}

pub fn compute_ortho_matrix(surface: &(impl Surface + ?Sized)) -> Matrix4<f32> {
    let surface_size = surface.size();
    Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
        * ortho(0.0, surface_size.x as f32, 0.0, surface_size.y as f32, 0.0, 1.0)
}

impl Draw2d {
    /// Creates an object that can render a few types of basic geometric shapes.
    pub fn new(context: &GlContext, programs: &Draw2dPrograms) -> Self {
        let triangle_mesh_builder = MeshBuilder::new();
        let triangle_mesh = Mesh::new(context, &programs.plain_program, DrawMode::Draw2D);
        let image_mesh_builder = MeshBuilder::new();
        let image_mesh_srgb = Mesh::new(context, &programs.image_program_srgb, DrawMode::Draw2D);
        let image_mesh_linear =
            Mesh::new(context, &programs.image_program_linear, DrawMode::Draw2D);
        Self {
            triangle_mesh_builder,
            triangle_mesh,
            image_mesh_builder,
            image_mesh_srgb,
            image_mesh_linear,
        }
    }

    /// Render all queued shapes. Until this is called nothing is actually rendered.
    ///
    /// This should typically be called once per frame to minimize the number of draw calls.
    pub fn render_queued(&mut self, surface: &(impl Surface + ?Sized)) {
        self.render_queued_custom_matrix(surface, compute_ortho_matrix(surface));
    }

    /// Render all queued shapes. Until this is called nothing is actually rendered.
    ///
    /// This allows a matrix to be specified which will be used instead of a standard orthographic
    /// projection.
    ///
    /// This should typically be called once per frame to minimize the number of draw calls.
    pub fn render_queued_custom_matrix(
        &mut self,
        surface: &(impl Surface + ?Sized),
        matrix: Matrix4<f32>,
    ) {
        self.triangle_mesh.build_from(&self.triangle_mesh_builder, MeshUsage::StreamDraw);
        self.triangle_mesh.draw(surface, &PlainUniforms { matrix, color: Color4::WHITE });

        self.triangle_mesh_builder.clear();
    }

    /// Draws a filled convex polygon.
    pub fn fill_poly(&mut self, verts: &[Point2<f32>], color: Color4) {
        assert!(verts.len() >= 3);
        let mesh_builder = &mut self.triangle_mesh_builder;
        let a = mesh_builder.vert(PlainVert { pos: verts[0], color });
        let mut b = mesh_builder.vert(PlainVert { pos: verts[1], color });
        for c in verts.iter().skip(2) {
            let c = mesh_builder.vert(PlainVert { pos: *c, color });
            mesh_builder.triangle(a, b, c);
            b = c;
        }
    }

    /// Draws a line strip.
    // TODO: change all coords to i32, and ensure that all verts are aligned to pixels?
    pub fn draw_line_strip(&mut self, verts: &[Point2<f32>], color: Color4, width: f32) {
        assert!(verts.len() >= 2);
        let mesh_builder = &mut self.triangle_mesh_builder;
        let half_width = width * 0.5;
        for (a, b) in verts.iter().zip(verts.iter().skip(1)) {
            let perp = ccw_perp(*b - *a).normalize();
            let vert_a = mesh_builder.vert(PlainVert { pos: *a + perp * half_width, color });
            let vert_b = mesh_builder.vert(PlainVert { pos: *a - perp * half_width, color });
            let vert_c = mesh_builder.vert(PlainVert { pos: *b + perp * half_width, color });
            let vert_d = mesh_builder.vert(PlainVert { pos: *b - perp * half_width, color });
            mesh_builder.triangle(vert_a, vert_b, vert_c);
            mesh_builder.triangle(vert_b, vert_c, vert_d);
        }
    }

    pub fn draw_line(&mut self, a: Point2<f32>, b: Point2<f32>, color: Color4, width: f32) {
        self.draw_line_strip(&[a, b], color, width);
    }

    pub fn fill_rect(&mut self, rect: Rect<i32>, color: Color4) {
        let rect = rect.cast().unwrap();
        self.fill_poly(
            &[
                rect.start,
                point2(rect.end.x, rect.start.y),
                rect.end,
                point2(rect.start.x, rect.end.y),
            ],
            color,
        );
    }

    pub fn outline_rect(&mut self, rect: Rect<i32>, color: Color4, width: f32) {
        let rect = rect.cast().unwrap();
        self.draw_line_strip(
            &[
                rect.start + vec2(0.5, 0.5),
                point2(rect.end.x, rect.start.y) + vec2(0.5, 0.5),
                rect.end + vec2(0.5, 0.5),
                point2(rect.start.x, rect.end.y) + vec2(0.5, 0.5),
                rect.start + vec2(0.5, 0.5),
            ],
            color,
            width,
        );
    }

    // TODO: merge these methods with the ones above if possible
    pub fn fill_rect_f32(&mut self, rect: Rect<f32>, color: Color4) {
        self.fill_poly(
            &[
                rect.start,
                point2(rect.end.x, rect.start.y),
                rect.end,
                point2(rect.start.x, rect.end.y),
            ],
            color,
        );
    }

    pub fn outline_rect_f32(&mut self, rect: Rect<f32>, color: Color4, width: f32) {
        self.draw_line_strip(
            &[
                rect.start + vec2(0.5, 0.5),
                point2(rect.end.x, rect.start.y) + vec2(0.5, 0.5),
                rect.end + vec2(0.5, 0.5),
                point2(rect.start.x, rect.end.y) + vec2(0.5, 0.5),
                rect.start + vec2(0.5, 0.5),
            ],
            color,
            width,
        );
    }

    /// Draws an image. Unlike most other functions on `Draw2d`, this draws the image immediately.
    pub fn draw_image(
        &mut self,
        surface: &(impl Surface + ?Sized),
        tex: &Texture2d,
        pos: Point2<f32>,
        scale: f32,
    ) {
        let matrix =
            compute_ortho_matrix(surface) * Matrix4::from_nonuniform_scale(scale, scale, 1.0);

        let a = self.image_mesh_builder.vert(ImageVert {
            pos,
            uv: point2(0.0, 0.0),
            color: Color4::WHITE,
        });
        let b = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(tex.size().x as f32, 0.0),
            uv: point2(1.0, 0.0),
            color: Color4::WHITE,
        });
        let c = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(0.0, tex.size().y as f32),
            uv: point2(0.0, 1.0),
            color: Color4::WHITE,
        });
        let d = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(tex.size().x as f32, tex.size().y as f32),
            uv: point2(1.0, 1.0),
            color: Color4::WHITE,
        });
        self.image_mesh_builder.triangle(a, b, c);
        self.image_mesh_builder.triangle(b, c, d);

        let image_mesh =
            if tex.is_srgb() { &mut self.image_mesh_srgb } else { &mut self.image_mesh_linear };
        image_mesh.build_from(&self.image_mesh_builder, MeshUsage::StreamDraw);
        image_mesh.draw(surface, &ImageUniforms { matrix, color: Color4::WHITE, tex });

        self.image_mesh_builder.clear();
    }

    /// Draws part of an image. Unlike most other functions on `Draw2d`, this draws the image immediately.
    pub fn draw_part_of_image(
        &mut self,
        surface: &(impl Surface + ?Sized),
        tex: &Texture2d,
        start: Point2<i32>,
        end: Point2<i32>,
        start_pos: Point2<f32>,
        end_pos: Point2<f32>,
        matrix: Matrix4<f32>,
    ) {
        let start: Point2<f32> = start.cast().unwrap();
        let end: Point2<f32> = end.cast().unwrap();
        let start2 = point2(start.x / tex.size().x as f32, start.y / tex.size().y as f32);
        let end2 = point2(end.x / tex.size().x as f32, end.y / tex.size().y as f32);

        let a = self.image_mesh_builder.vert(ImageVert {
            pos: start_pos,
            uv: start2,
            color: Color4::WHITE,
        });
        let b = self.image_mesh_builder.vert(ImageVert {
            pos: point2(end_pos.x, start_pos.y),
            uv: point2(end2.x, start2.y),
            color: Color4::WHITE,
        });
        let c = self.image_mesh_builder.vert(ImageVert {
            pos: point2(start_pos.x, end_pos.y),
            uv: point2(start2.x, end2.y),
            color: Color4::WHITE,
        });
        let d = self.image_mesh_builder.vert(ImageVert {
            pos: end_pos,
            uv: end2,
            color: Color4::WHITE,
        });
        self.image_mesh_builder.triangle(a, b, c);
        self.image_mesh_builder.triangle(b, c, d);

        let image_mesh =
            if tex.is_srgb() { &mut self.image_mesh_srgb } else { &mut self.image_mesh_linear };
        image_mesh.build_from(&self.image_mesh_builder, MeshUsage::StreamDraw);
        image_mesh.draw(surface, &ImageUniforms { matrix, color: Color4::WHITE, tex });

        self.image_mesh_builder.clear();
    }
}

/// Returns the vector 90 degrees counterclockwise from the given vector.
#[inline]
fn ccw_perp<T: Neg<Output = T>>(x: Vector2<T>) -> Vector2<T> {
    vec2(x.y, -x.x)
}
