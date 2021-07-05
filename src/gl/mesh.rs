use glow::HasContext;
use std::marker::PhantomData;

use super::context::*;
use super::program::*;
use super::surface::*;
use super::uniforms::*;

pub type GlBuffer = <glow::Context as HasContext>::Buffer;
pub type GlVertexArrayObject = <glow::Context as HasContext>::VertexArray;

/// An OpenGL primitive.
#[doc(hidden)]
pub trait Primitive {
    const AS_GL: u32;
}

#[derive(Copy, Clone, Debug)]
pub enum MeshUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
}

impl MeshUsage {
    fn as_gl(self) -> u32 {
        match self {
            MeshUsage::StaticDraw => glow::STATIC_DRAW,
            MeshUsage::DynamicDraw => glow::DYNAMIC_DRAW,
            MeshUsage::StreamDraw => glow::STREAM_DRAW,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DrawMode {
    Draw2D,
    Draw3D { depth: bool },
}

impl DrawMode {
    fn bind(self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.draw_mode != Some(self) {
            cache.draw_mode = Some(self);

            match self {
                DrawMode::Draw2D => {
                    context.disable(GlFlag::CullFace);
                    context.disable(GlFlag::DepthTest);
                }
                DrawMode::Draw3D { depth } => {
                    context.enable(GlFlag::CullFace);
                    if depth {
                        context.enable(GlFlag::DepthTest);
                    } else {
                        context.disable(GlFlag::DepthTest);
                    }
                }
            }
        }
    }
}

/// An index into a mesh.
pub type MeshIndex = u16;

/// A struct that builds a mesh from a collection of primitives.
///
/// This struct only stores the mesh data and indices; to use it in OpenGL, it must be used to
/// build a `Mesh`.
pub struct MeshBuilder<V: Vertex, P: Primitive> {
    vertex_data: Vec<f32>,
    indices: Vec<MeshIndex>,
    next_index: MeshIndex,
    phantom: PhantomData<(V, P)>,
}

impl<V: Vertex> MeshBuilder<V, Triangles> {
    // TODO: consider entirely replacing MeshBuilder with lyon
    #[cfg(feature = "lyon_tessellation")]
    pub fn from_lyon_vertex_buffers(
        vertex_buffers: lyon_tessellation::VertexBuffers<V, MeshIndex>,
    ) -> Self {
        let mut vertex_data = vec![];
        for vert in &vertex_buffers.vertices {
            vert.add_to_mesh(&mut |data| vertex_data.push(data));
        }
        Self {
            vertex_data,
            indices: vertex_buffers.indices,
            next_index: vertex_buffers.vertices.len() as u16,
            phantom: PhantomData,
        }
    }
}

impl<V: Vertex, P: Primitive> MeshBuilder<V, P> {
    pub fn new() -> Self {
        MeshBuilder { vertex_data: vec![], indices: vec![], next_index: 0, phantom: PhantomData }
    }

    /// Adds a vertex to the mesh. The vertex won't be rendered unless it's used in a primitive
    /// (currently either `Triangles`, `Lines`, or `Points`, each of which adds a method to this
    /// struct to add the corresponding primitive).
    pub fn vert(&mut self, vert: V) -> MeshIndex {
        assert!(self.next_index < MeshIndex::max_value());
        let index = self.next_index;
        self.next_index += 1;
        vert.add_to_mesh(&mut |data| self.vertex_data.push(data));
        index
    }

    pub fn verts(&mut self, verts: Vec<V>) -> Vec<MeshIndex> {
        let mut res = Vec::with_capacity(verts.len());
        for vert in verts {
            res.push(self.vert(vert));
        }
        res
    }

    /// Builds a `Mesh` from this `MeshBuilder`.
    pub fn build<U: GlUniforms>(
        &self,
        context: &GlContext,
        program: &GlProgram<V, U>,
        usage: MeshUsage,
        draw_mode: DrawMode,
    ) -> Mesh<V, U, P> {
        let mut mesh = Mesh::new(context, program, draw_mode);
        mesh.build_from(self, usage);
        mesh
    }

    /// Clears all data from the `MeshBuilder`. Does *not* reclaim the memory that had been used,
    /// so reusing the `MeshBuilder` won't have to reallocate unless the new mesh is larger than
    /// the old one.
    pub fn clear(&mut self) {
        self.vertex_data.clear();
        self.indices.clear();
        self.next_index = 0;
    }

    /// Adds all vertices and primitives from the other mesh to this mesh.
    pub fn extend(&mut self, other: MeshBuilder<V, P>) {
        let start_index = self.next_index;
        let num_verts = other.next_index;
        self.next_index += num_verts;
        self.vertex_data.extend(other.vertex_data);
        self.indices.extend(other.indices.iter().map(|x| x + start_index));
    }

    pub fn next_index(&self) -> MeshIndex {
        self.next_index
    }
}

impl<V: Vertex, P: Primitive> Default for MeshBuilder<V, P> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone)]
pub struct Triangles;

impl Primitive for Triangles {
    const AS_GL: u32 = glow::TRIANGLES;
}

impl<V: Vertex> MeshBuilder<V, Triangles> {
    /// Adds a triangle to the mesh.
    pub fn triangle(&mut self, a: MeshIndex, b: MeshIndex, c: MeshIndex) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }
}

#[derive(Copy, Clone)]
pub struct TriangleStrip;

impl Primitive for TriangleStrip {
    const AS_GL: u32 = glow::TRIANGLE_STRIP;
}

impl<V: Vertex> MeshBuilder<V, TriangleStrip> {
    /// Adds an index to the mesh.
    pub fn triangle_strip_index(&mut self, a: MeshIndex) {
        self.indices.push(a);
    }
}

#[derive(Copy, Clone)]
pub struct TriangleFan;

impl Primitive for TriangleFan {
    const AS_GL: u32 = glow::TRIANGLE_FAN;
}

impl<V: Vertex> MeshBuilder<V, TriangleFan> {
    /// Adds an index to the mesh.
    pub fn triangle_fan_index(&mut self, a: MeshIndex) {
        self.indices.push(a);
    }
}

#[derive(Copy, Clone)]
pub struct Lines;

impl Primitive for Lines {
    const AS_GL: u32 = glow::LINES;
}

impl<V: Vertex> MeshBuilder<V, Lines> {
    /// Adds a line to the mesh.
    pub fn line(&mut self, a: MeshIndex, b: MeshIndex) {
        self.indices.push(a);
        self.indices.push(b);
    }
}

#[derive(Copy, Clone)]
pub struct LineStrip;

impl Primitive for LineStrip {
    const AS_GL: u32 = glow::LINE_STRIP;
}

impl<V: Vertex> MeshBuilder<V, LineStrip> {
    /// Adds an index to the mesh.
    pub fn line_strip_index(&mut self, a: MeshIndex) {
        self.indices.push(a);
    }
}

#[derive(Copy, Clone)]
pub struct LineLoop;

impl Primitive for LineLoop {
    const AS_GL: u32 = glow::LINE_LOOP;
}

impl<V: Vertex> MeshBuilder<V, LineLoop> {
    /// Adds an index to the mesh.
    pub fn line_loop_index(&mut self, a: MeshIndex) {
        self.indices.push(a);
    }
}

#[derive(Copy, Clone)]
pub struct Points;

impl Primitive for Points {
    const AS_GL: u32 = glow::POINTS;
}

impl<V: Vertex> MeshBuilder<V, Points> {
    /// Adds a point to the mesh.
    pub fn point(&mut self, a: MeshIndex) {
        self.indices.push(a);
    }
}

/// A mesh; built using a `MeshBuilder`.
pub struct Mesh<V: Vertex, U: GlUniforms, P: Primitive> {
    vao: GlVertexArrayObject,
    vbo: GlBuffer,
    ibo: GlBuffer,
    context: GlContext,
    program: GlProgram<V, U>,
    num_indices: i32,
    phantom: PhantomData<P>,
    // TODO: can this be inferred from the vertex/uniforms types?
    draw_mode: DrawMode,
}

impl<V: Vertex, U: GlUniforms, P: Primitive> Drop for Mesh<V, U, P> {
    fn drop(&mut self) {
        unsafe {
            self.context.inner().delete_vertex_array(self.vao);
            self.context.inner().delete_buffer(self.vbo);
            self.context.inner().delete_buffer(self.ibo);
        }
    }
}

impl<V: Vertex, U: GlUniforms, P: Primitive> Mesh<V, U, P> {
    /// Creates an empty `Mesh`. It must have data written via `build_from` before it's usable.
    pub fn new(context: &GlContext, program: &GlProgram<V, U>, draw_mode: DrawMode) -> Self {
        unsafe {
            let vao = context.inner().create_vertex_array().unwrap();
            context.inner().bind_vertex_array(Some(vao));

            let vbo = context.inner().create_buffer().unwrap();
            let ibo = context.inner().create_buffer().unwrap();
            context.inner().bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            context.inner().bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));

            Mesh {
                vao,
                vbo,
                ibo,
                context: context.clone(),
                program: program.clone(),
                num_indices: 0,
                phantom: PhantomData,
                draw_mode,
            }
        }
    }

    /// Clears the mesh's current contents and updates it with the contents of the `MeshBuilder`.
    pub fn build_from(&mut self, builder: &MeshBuilder<V, P>, usage: MeshUsage) {
        self.num_indices = builder.indices.len() as i32;
        if self.num_indices == 0 {
            return;
        }

        self.bind();

        setup_vertex_attribs::<V, _, _>(&self.program, false);

        unsafe {
            self.context.inner().buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                // TODO: find a better way to convert a &[f32] to a &[u8], or modify `glow` so the conversion isn't needed
                std::slice::from_raw_parts(
                    (&builder.vertex_data).as_ptr() as *const u8,
                    builder.vertex_data.len() * std::mem::size_of::<f32>(),
                ),
                usage.as_gl(),
            );

            self.context.inner().buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    (&builder.indices).as_ptr() as *const u8,
                    builder.indices.len() * std::mem::size_of::<u16>(),
                ),
                usage.as_gl(),
            );
        }
    }

    fn bind(&self) {
        unsafe {
            self.context.inner().bind_vertex_array(Some(self.vao));
            // The ELEMENT_ARRAY_BUFFER doesn't need to be bound here, but the ARRAY_BUFFER does (https://stackoverflow.com/a/21652930)
            self.context.inner().bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        }
    }

    /// Draws the mesh.
    pub fn draw(
        &self,
        surface: &(impl Surface + ?Sized),
        uniforms: &impl Uniforms<GlUniforms = U>,
    ) {
        if self.num_indices == 0 {
            return;
        }

        self.bind();
        self.program.bind(&self.context);
        uniforms.update(&self.context, &self.program.inner.gl_uniforms);
        surface.bind(&self.context);
        self.draw_mode.bind(&self.context);

        unsafe {
            self.context.inner().draw_elements(P::AS_GL, self.num_indices, glow::UNSIGNED_SHORT, 0);
        }
    }

    /// Draws the mesh using instanced rendering. Like `draw()`, but several instances
    /// can be passed in the `instances` parameter and the mesh will be drawn once for each
    /// instance. The instance data's fields must be in the same order as its `VertexData` impl
    /// specifies, and it must use `#[repr(C)]`.
    pub fn draw_instanced<I: VertexData>(
        &self,
        surface: &(impl Surface + ?Sized),
        uniforms: &impl Uniforms<GlUniforms = U>,
        instances: &[I],
    ) {
        if self.num_indices == 0 || instances.is_empty() {
            return;
        }

        self.bind();
        self.program.bind(&self.context);
        uniforms.update(&self.context, &self.program.inner.gl_uniforms);
        surface.bind(&self.context);
        self.draw_mode.bind(&self.context);

        unsafe {
            self.context.inner().bind_buffer(glow::ARRAY_BUFFER, Some(self.context.instanced_vbo));

            setup_vertex_attribs::<I, _, _>(&self.program, true);

            self.context.inner().buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    instances.as_ptr() as *const u8,
                    instances.len() * std::mem::size_of::<I>(),
                ),
                // TODO: make this configurable
                MeshUsage::StreamDraw.as_gl(),
            );

            self.context.inner().draw_elements_instanced(
                P::AS_GL,
                self.num_indices,
                glow::UNSIGNED_SHORT,
                0,
                instances.len() as i32,
            );
        }
    }
}

fn setup_vertex_attribs<D: VertexData, V: Vertex, U: GlUniforms>(
    program: &GlProgram<V, U>,
    instanced: bool,
) {
    let context = &program.inner.context;
    let stride = D::stride();
    let mut offset = 0;
    for (attr, size) in D::ATTRIBUTES.iter() {
        let loc = unsafe {
            context.inner().get_attrib_location(program.inner.program, attr).unwrap() as u32
        };

        // Matrices take up 4 attributes and each row has to be specified separately.
        if *size == 16 {
            setup_vertex_attrib(context, loc, 4, stride, offset, instanced);
            setup_vertex_attrib(context, loc + 1, 4, stride, offset + 4, instanced);
            setup_vertex_attrib(context, loc + 2, 4, stride, offset + 8, instanced);
            setup_vertex_attrib(context, loc + 3, 4, stride, offset + 12, instanced);
        } else if *size <= 4 {
            setup_vertex_attrib(context, loc, *size, stride, offset, instanced);
        } else {
            panic!("Unsupported vertex data size");
        }

        offset += size;
    }
}

fn setup_vertex_attrib(
    context: &GlContext,
    loc: u32,
    size: i32,
    stride: i32,
    offset: i32,
    instanced: bool,
) {
    unsafe {
        context.inner().enable_vertex_attrib_array(loc);
        context.inner().vertex_attrib_pointer_f32(
            loc,
            size,
            glow::FLOAT,
            false,
            stride * 4,
            offset * 4,
        );
        if instanced {
            context.inner().vertex_attrib_divisor(loc, 1);
        }
    }
}
