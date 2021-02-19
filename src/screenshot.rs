#![cfg(not(target_arch = "wasm32"))]

use glow::HasContext;

use crate::gl::*;
use image;
use image::DynamicImage;
use std::fs;
use std::path::*;

pub fn take_screenshot(
    context: &GlContext,
    surface: &impl Surface,
    path: Option<PathBuf>,
    include_alpha: bool,
) {
    surface.bind(context);
    let window_size = surface.size();
    let mut pixels =
        vec![0; (window_size.x * window_size.y * (if include_alpha { 4 } else { 3 })) as usize];
    unsafe {
        context.inner().read_buffer(glow::FRONT);
        context.inner().read_pixels(
            0,
            0,
            window_size.x as i32,
            window_size.y as i32,
            if include_alpha { glow::RGBA } else { glow::RGB },
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(&mut pixels),
        );
    }

    let path = match path {
        Some(path) => path,
        None => {
            let time = chrono::offset::Local::now();
            Path::new(&format!("screenshots/screenshot-{}.png", time.to_rfc3339())).to_path_buf()
        }
    };
    match path.parent() {
        None => panic!("Invalid screenshot path"),
        Some(dir) => {
            let metadata = fs::metadata(dir);
            let should_create_dir = match metadata {
                Err(_) => true,
                Ok(metadata) => !metadata.is_dir(),
            };
            if should_create_dir {
                fs::create_dir_all(&dir).unwrap();
            }
            if include_alpha {
                // TODO: there's some redundant conversions here
                // TODO: why is flipping the image necessary?
                let image_buf = image::ImageBuffer::from_raw(
                    window_size.x as u32,
                    window_size.y as u32,
                    pixels,
                )
                .unwrap();
                let img = DynamicImage::ImageRgba8(image_buf).flipv();
                img.to_rgba8().save(&path).unwrap();
            } else {
                let image_buf = image::ImageBuffer::from_raw(
                    window_size.x as u32,
                    window_size.y as u32,
                    pixels,
                )
                .unwrap();
                let img = DynamicImage::ImageRgb8(image_buf).flipv();
                img.to_rgb8().save(&path).unwrap();
            }
        }
    }
}
