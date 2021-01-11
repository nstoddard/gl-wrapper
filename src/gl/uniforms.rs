use glow::HasContext;

use super::context::*;
use super::program::*;
use super::texture::*;

type GlUniformLocation = <glow::Context as HasContext>::UniformLocation;

/// Holds uniforms for a given program.
///
/// Example implementation:
/// ```
/// struct ExampleUniforms<'a> {
///     matrix: Matrix4<f32>,
///     tex: &'a Texture2d,
/// }
///
/// struct ExampleUniformsGl {
///     matrix: Matrix4Uniform,
///     tex: TextureUniform,
/// }
///
/// impl<'a> Uniforms for ExampleUniforms<'a> {
///     type GlUniforms = ExampleUniformsGl;
///
///     fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
///         gl_uniforms.matrix.set(context, &self.matrix);
///         gl_uniforms.tex.set(context, self.tex, 0);
///     }
/// }
///
/// impl GlUniforms for ExampleUniformsGl {
///     fn new(context: &GlContext, program: &GlProgramId) -> Self {
///         ExampleUniformsGl {
///             matrix: Matrix4Uniform::new("matrix", context, program),
///             tex: TextureUniform::new("tex", context, program),
///         }
///     }
/// }
/// ```
pub trait Uniforms {
    /// The `GlUniforms` instance corresponding to this `Uniforms`.
    type GlUniforms: GlUniforms;

    /// Updates the given `GlUniforms` from this `Uniforms`. Should call `set` on each uniform
    /// in the associated `GlUniforms`.
    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms);
}

/// A type used to hold the uniform locations, which can be updated from a corresponding instance of the `Uniforms` trait.
///
/// See the `Uniforms` trait for an example implementation.
pub trait GlUniforms {
    fn new(context: &GlContext, program: GlProgramId) -> Self;
}

// TODO: these structs are probably redundant
pub struct Matrix4Uniform {
    loc: GlUniformLocation,
}

impl Matrix4Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, mat: &impl AsRef<[f32; 16]>) {
        unsafe {
            context.inner().uniform_matrix_4_f32_slice(Some(&self.loc), false, mat.as_ref());
        }
    }
}

pub struct TextureUniform {
    loc: GlUniformLocation,
}

impl TextureUniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, texture: &Texture2d, texture_unit: u32) {
        unsafe {
            context.inner().uniform_1_i32(Some(&self.loc), texture_unit as i32);
        }
        texture.bind(texture_unit);
    }
}

pub struct Vector2Uniform {
    loc: GlUniformLocation,
}

impl Vector2Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 2]>) {
        let val = val.as_ref();
        unsafe {
            context.inner().uniform_2_f32(Some(&self.loc), val[0], val[1]);
        }
    }
}

pub struct Vector3Uniform {
    loc: GlUniformLocation,
}

impl Vector3Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 3]>) {
        let val = val.as_ref();
        unsafe {
            context.inner().uniform_3_f32(Some(&self.loc), val[0], val[1], val[2]);
        }
    }
}

pub struct Vector4Uniform {
    loc: GlUniformLocation,
}

impl Vector4Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 4]>) {
        let val = val.as_ref();
        unsafe {
            context.inner().uniform_4_f32(Some(&self.loc), val[0], val[1], val[2], val[3]);
        }
    }
}

pub struct Array2Uniform {
    loc: GlUniformLocation,
}

impl Array2Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 2]) {
        unsafe {
            context.inner().uniform_2_f32(Some(&self.loc), val[0], val[1]);
        }
    }
}

pub struct Array3Uniform {
    loc: GlUniformLocation,
}

impl Array3Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 3]) {
        unsafe {
            context.inner().uniform_3_f32(Some(&self.loc), val[0], val[1], val[2]);
        }
    }
}

pub struct Array4Uniform {
    loc: GlUniformLocation,
}

impl Array4Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 4]) {
        unsafe {
            context.inner().uniform_4_f32(Some(&self.loc), val[0], val[1], val[2], val[3]);
        }
    }
}

pub struct F32Uniform {
    loc: GlUniformLocation,
}

impl F32Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { loc: unsafe { context.inner().get_uniform_location(program, name).unwrap() } }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: f32) {
        unsafe {
            context.inner().uniform_1_f32(Some(&self.loc), val);
        }
    }
}


/// An instance of `Uniforms` that contains no data.
pub struct EmptyUniforms {}

pub struct EmptyUniformsGl {}

impl Uniforms for EmptyUniforms {
    type GlUniforms = EmptyUniformsGl;

    fn update(&self, _context: &GlContext, _gl_uniforms: &Self::GlUniforms) {}
}

impl GlUniforms for EmptyUniformsGl {
    fn new(_context: &GlContext, _program: GlProgramId) -> Self {
        EmptyUniformsGl {}
    }
}
