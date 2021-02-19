//! A stateless wrapper around OpenGL, to make it easier to use and more type-safe.
mod gl;
#[cfg(not(target_arch = "wasm32"))]
mod glfw;
mod gui;
#[cfg(not(target_arch = "wasm32"))]
mod screenshot;

pub use gl::*;
pub use gui::*;
