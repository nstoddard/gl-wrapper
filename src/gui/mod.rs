//! A work-in-progress GUI library.
//!
//! This library currently also contains asset loading and a main loop, but these might
//! be moved to separate crates at some point.

mod assets;
mod color;
mod draw_2d;
mod event;
mod gui;
mod main_loop;
mod shader_header;
mod text;
pub mod widgets;

pub use self::assets::*;
pub use self::color::*;
pub use self::draw_2d::*;
pub use self::event::*;
pub use self::gui::*;
pub use self::main_loop::*;
pub use self::shader_header::*;
pub use self::text::Font;
