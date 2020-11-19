use crate::gl::uniforms::*;
use crate::gl::*;

const COMMON_HEADER: &str = "#version 300 es
precision highp float;
precision highp sampler2D;
precision highp samplerCube;
precision highp sampler2DArray;
";

const VERT_HEADER: &str = "
void writeGlPosition2D(vec4 pos) {
    gl_Position = pos;
}
";

const FRAG_HEADER_SRGB: &str = "
out vec4 outColor;

vec4 srgb(vec4 color) {
  return vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
}

void writeColor2D(vec4 color) {
  outColor = srgb(color);
}
";

const FRAG_HEADER_NO_SRGB: &str = "
out vec4 outColor;

void writeColor2D(vec4 color) {
  outColor = color;
}
";

fn get_shader_header(shader_type: ShaderType, convert_to_srgb: bool) -> &'static str {
    match shader_type {
        ShaderType::Vertex => VERT_HEADER,
        ShaderType::Fragment => {
            if convert_to_srgb {
                FRAG_HEADER_SRGB
            } else {
                FRAG_HEADER_NO_SRGB
            }
        }
    }
}

fn add_shader_minimal_header(source: &str) -> String {
    let header = COMMON_HEADER;
    format!("{}{}", header, source)
}

fn add_shader_header(shader_type: ShaderType, source: &str, convert_to_srgb: bool) -> String {
    let header = format!("{}{}", COMMON_HEADER, get_shader_header(shader_type, convert_to_srgb));
    format!("{}{}", header, source)
}

/// Some additional constructors for `GlProgram` to make it easier to create shaders which share
/// a common header.
pub trait GlProgramWithHeader {
    /// Adds a `#version` declaration to each shader and `precision highp` declarations.
    fn new_with_minimal_header(
        context: &GlContext,
        vert_shader_source: &str,
        frag_shader_source: &str,
    ) -> Self;

    /// Adds a header to each shader, which includes everything added in
    /// `new_with_minimal_header` plus sRGB conversion functions.
    fn new_with_header(
        context: &GlContext,
        vert_shader_source: &str,
        frag_shader_source: &str,
        convert_to_srgb: bool,
    ) -> Self;
}

impl<V: Vertex, U: GlUniforms> GlProgramWithHeader for GlProgram<V, U> {
    fn new_with_minimal_header(
        context: &GlContext,
        vert_shader_source: &str,
        frag_shader_source: &str,
    ) -> Self {
        Self::new(
            context,
            &add_shader_minimal_header(vert_shader_source),
            &add_shader_minimal_header(frag_shader_source),
        )
    }

    fn new_with_header(
        context: &GlContext,
        vert_shader_source: &str,
        frag_shader_source: &str,
        convert_to_srgb: bool,
    ) -> Self {
        Self::new(
            context,
            &add_shader_header(ShaderType::Vertex, vert_shader_source, convert_to_srgb),
            &add_shader_header(ShaderType::Fragment, frag_shader_source, convert_to_srgb),
        )
    }
}
