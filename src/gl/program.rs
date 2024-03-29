use cgmath::*;
use glow::HasContext;
use log::*;
use std::marker::PhantomData;
use std::rc::Rc;
use uid::*;

use super::context::*;
use super::uniforms::*;

#[doc(hidden)]
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ProgramId_(());

pub type ProgramId = Id<ProgramId_>;

type GlShader = <glow::Context as HasContext>::Shader;
/// An identifier representing an OpenGL program, used when the full `GlProgram` can't be used.
pub type GlProgramId = <glow::Context as HasContext>::Program;

#[derive(Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn as_gl(self) -> u32 {
        match self {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        }
    }
}

/// An OpenGL program.
pub struct GlProgram<V: Vertex, U: GlUniforms> {
    pub inner: Rc<GlProgramInner<V, U>>,
}

impl<V: Vertex, U: GlUniforms> Clone for GlProgram<V, U> {
    fn clone(&self) -> GlProgram<V, U> {
        GlProgram { inner: self.inner.clone() }
    }
}

pub struct GlProgramInner<V: Vertex, U: GlUniforms> {
    pub program: GlProgramId,
    pub gl_uniforms: U,
    phantom: PhantomData<V>,
    id: ProgramId,
    pub context: GlContext,
    vert_shader: GlShader,
    frag_shader: GlShader,
}

impl<V: Vertex, U: GlUniforms> Drop for GlProgramInner<V, U> {
    fn drop(&mut self) {
        unsafe {
            self.context.inner().delete_program(self.program);
            self.context.inner().delete_shader(self.vert_shader);
            self.context.inner().delete_shader(self.frag_shader);
        }
    }
}

impl<V: Vertex, U: GlUniforms> GlProgram<V, U> {
    pub fn new(context: &GlContext, vert_shader_source: &str, frag_shader_source: &str) -> Self {
        let vert_shader = Self::load_shader(context, ShaderType::Vertex, vert_shader_source);
        let frag_shader = Self::load_shader(context, ShaderType::Fragment, frag_shader_source);

        let program = unsafe {
            let program = context.inner().create_program().unwrap();
            context.inner().attach_shader(program, vert_shader);
            context.inner().attach_shader(program, frag_shader);
            context.inner().link_program(program);

            let link_status = context.inner().get_program_link_status(program);
            if !link_status {
                error!("Error linking program: {}", context.inner().get_program_info_log(program));
                panic!();
            }
            program
        };

        let gl_uniforms = U::new(context, program);

        GlProgram {
            inner: Rc::new(GlProgramInner {
                program,
                gl_uniforms,
                phantom: PhantomData,
                id: ProgramId::new(),
                context: context.clone(),
                vert_shader,
                frag_shader,
            }),
        }
    }

    fn load_shader(context: &GlContext, shader_type: ShaderType, source: &str) -> GlShader {
        unsafe {
            let shader = context.inner().create_shader(shader_type.as_gl()).unwrap();
            context.inner().shader_source(shader, source);
            context.inner().compile_shader(shader);

            let compile_status = context.inner().get_shader_compile_status(shader);
            if !compile_status {
                error!("Error compiling shader: {}", context.inner().get_shader_info_log(shader));
                panic!();
            }

            shader
        }
    }

    pub fn bind(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_program != Some(self.inner.id) {
            cache.bound_program = Some(self.inner.id);
            unsafe {
                context.inner().use_program(Some(self.inner.program));
            }
        }
    }
}

/// A list of all OpenGL attributes for a given program.
///
/// Each pair is (attribute name, attribute size).
///
/// The size should be the size in *floats*, not bytes.
pub type Attributes = &'static [(&'static str, i32)];

/// A vertex for a given program.
///
/// Example implementation:
/// ```
/// struct ExampleVertex {
///     pos: Vector2<f32>,
///     uv: Vector2<f32>,
/// }
///
/// impl VertexData for ExampleVertex {
///     const ATTRIBUTES: Attributes = &[("pos", 2), ("uv", 2)];
/// }
///
/// impl VertexComponent for ExampleVertex {
///     fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
///         self.pos.add_to_mesh(f);
///         self.uv.add_to_mesh(f);
///     }
/// }
/// ```
pub trait Vertex: VertexData + VertexComponent {}

impl<T: VertexData + VertexComponent> Vertex for T {}

pub trait VertexData {
    /// A list of all OpenGL attributes that each vertex contains.
    const ATTRIBUTES: Attributes;

    // TODO: find a way to cache this
    fn stride() -> i32 {
        Self::ATTRIBUTES.iter().map(|&(_, size)| size).sum()
    }
}

/// A component of a vertex.
///
/// See the `Vertex` trait for an example implementation.
pub trait VertexComponent {
    /// Adds the `VertexComponent` to a mesh by calling the given closure for each
    /// `f32` component, in order. Composite `VertexComponent` instances can call
    /// `add_to_mesh` for each of their components rather than calling the closure directly.
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32));
}

impl VertexComponent for f32 {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(*self);
    }
}

impl VertexComponent for Vector2<f32> {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.x);
        f(self.y);
    }
}

impl VertexComponent for Vector3<f32> {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.x);
        f(self.y);
        f(self.z);
    }
}

impl VertexComponent for Vector4<f32> {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.x);
        f(self.y);
        f(self.z);
        f(self.w);
    }
}

impl VertexComponent for Point2<f32> {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.x);
        f(self.y);
    }
}

impl VertexComponent for Point3<f32> {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.x);
        f(self.y);
        f(self.z);
    }
}

impl VertexComponent for [f32; 2] {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self[0]);
        f(self[1]);
    }
}

impl VertexComponent for [f32; 3] {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self[0]);
        f(self[1]);
        f(self[2]);
    }
}

impl VertexComponent for [f32; 4] {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self[0]);
        f(self[1]);
        f(self[2]);
        f(self[3]);
    }
}
