use crate::gl::uniforms::*;
use crate::gl::*;
use serde::*;
use std::ops::*;

/// An RGBA color, stored in a linear color space.
///
/// When converting to/from sRGB, this currently approximates sRGB by using a gamma of 2.2.
#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub const BLACK: Color4 = Color4 { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color4 = Color4 { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color4 = Color4 { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color4 = Color4 { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color4 = Color4 { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const CYAN: Color4 = Color4 { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: Color4 = Color4 { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: Color4 = Color4 { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color4 = Color4 { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    /// Creates a `Color4` from an sRGB color plus an alpha component.
    ///
    /// Note that the alpha component is *not* transformed by this function.
    pub fn from_srgba(r: f32, g: f32, b: f32, a: f32) -> Color4 {
        Color4 { r: r.powf(2.2), g: g.powf(2.2), b: b.powf(2.2), a }
    }

    /// Creates a `Color4` from an sRGB color.
    pub fn from_srgb(r: f32, g: f32, b: f32) -> Color4 {
        Color4 { r: r.powf(2.2), g: g.powf(2.2), b: b.powf(2.2), a: 1.0 }
    }

    /// Creates a `Color4` from an sRGB grayscale value.
    pub fn from_grayscale_srgb(x: f32) -> Color4 {
        let x = x.powf(2.2);
        Color4 { r: x, g: x, b: x, a: 1.0 }
    }

    /// Creates a `Color4` from an HSV color.
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Color4 {
        let s2 = s.abs();
        let c = v * s2;
        let h2 = ((h + (if s < 0.0 { 0.5 } else { 0.0 })) % 1.0) * 6.0;
        let x = c * (1.0 - ((h2 % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h2 as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => panic!("Invalid color in hsv"),
        };
        Color4::from_srgb(r + m, g + m, b + m)
    }

    /// Converts the `Color4` to sRGB and returns the result in an array.
    pub fn to_srgb(self) -> [f32; 4] {
        [self.r.powf(1.0 / 2.2), self.g.powf(1.0 / 2.2), self.b.powf(1.0 / 2.2), self.a]
    }

    /// Converts the `Color4` to an array, without converting to sRGB.
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Blends the `Color4` with another color, according to the other color's alpha.
    pub fn blend(&self, other: Color4) -> Color4 {
        let mut self2 = *self * (1.0 - other.a);
        // This line is necessary because multiplication doesn't multiply the alpha.
        self2.a *= 1.0 - other.a;
        self2 + other * other.a
    }

    /// Interpolates between two colors.
    pub fn lerp(self, other: Color4, other_amount: f32) -> Color4 {
        self * (1.0 - other_amount) + other * other_amount
    }

    /// Converts this `Color4` to sRGB, multiplies by the given constant, and converts back.
    ///
    /// Because the multiplication is done in sRGB, this will typically result in a more
    /// realistic-looking result than when multiplying in a linear color space.
    pub fn mul_srgb(&self, rhs: f32) -> Self {
        let srgb = self.to_srgb();
        Color4::from_srgba(srgb[0] * rhs, srgb[1] * rhs, srgb[2] * rhs, srgb[3])
    }
}

impl Add<Color4> for Color4 {
    type Output = Color4;
    fn add(self, rhs: Color4) -> Color4 {
        Color4 { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b, a: self.a + rhs.a }
    }
}

impl Mul<f32> for Color4 {
    type Output = Color4;
    /// Multiplication doesn't multiply the alpha component.
    fn mul(self, rhs: f32) -> Color4 {
        Color4 { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs, a: self.a }
    }
}

impl From<Color4> for [f32; 4] {
    /// Converts the `Color4` into an array, converting to sRGB in the process.
    fn from(color: Color4) -> [f32; 4] {
        color.to_srgb()
    }
}

impl VertexComponent for Color4 {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        f(self.r);
        f(self.g);
        f(self.b);
        f(self.a);
    }
}

/// A uniform for a `Color4`.
pub struct Color4Uniform {
    inner: Array4Uniform,
}

impl Color4Uniform {
    pub fn new(name: &str, context: &GlContext, program: GlProgramId) -> Self {
        Self { inner: Array4Uniform::new(name, context, program) }
    }

    // TODO: guarantee that the program is bound when this is called
    /// Sets the uniform. If `convert_to_srgb` is true, the color will be converted to sRGB first.
    /// In most cases, the color should be kept in a linear color space here (so `convert_to_srgb`
    /// would be false), and the fragment shader should convert to sRGB as the final step.
    pub fn set(&self, context: &GlContext, color: &Color4, convert_to_srgb: bool) {
        self.inner.set(context, if convert_to_srgb { color.to_srgb() } else { color.to_array() });
    }
}
