use anyhow::{ensure, Result};
use core::slice;
// font_kit already has a wrapper around freetype called Font so use it directly
use font_kit::font::Font;
use font_kit::loaders::freetype::NativeFont;
// use harfbuzz for shaping ligatures
pub use harfbuzz::*;
use harfbuzz_sys as harfbuzz;
use std::mem;

/// font feature tag
pub fn feature_from_tag(tag: &str) -> Result<hb_feature_t> {
    unsafe {
        let mut feature = mem::zeroed();
        ensure!(
            hb_feature_from_string(
                tag.as_ptr() as *const i8,
                tag.len() as i32,
                &mut feature as *mut _
            ) != 0,
            "hb_feature_from_string failed for {}",
            tag
        );
        Ok(feature)
    }
}

/// Harfbuzz font
pub struct HBFont {
    font: *mut hb_font_t,
}

// harfbuzz freetype integration
extern "C" {
    pub fn hb_ft_font_create_referenced(face: NativeFont) -> *mut hb_font_t; // the same as hb_face_t
}

impl Drop for HBFont {
    fn drop(&mut self) {
        unsafe { hb_font_destroy(self.font) }
    }
}

impl HBFont {
    pub fn new(face: &Font) -> HBFont {
        HBFont {
            font: unsafe { hb_ft_font_create_referenced(face.native_font() as _) },
        }
    }
    pub fn shape(&mut self, buffer: &HBBuffer, features: &[hb_feature_t]) {
        unsafe {
            hb_shape(
                self.font,
                buffer.buffer,
                features.as_ptr(),
                features.len() as u32,
            );
        }
    }
}

/// Harfbuzz buffer
pub struct HBBuffer {
    buffer: *mut hb_buffer_t,
}

impl Drop for HBBuffer {
    fn drop(&mut self) {
        unsafe { hb_buffer_destroy(self.buffer) }
    }
}

impl HBBuffer {
    pub fn new() -> Result<HBBuffer> {
        let hb_buf = unsafe { hb_buffer_create() };
        ensure!(
            unsafe { hb_buffer_allocation_successful(hb_buf) } != 0,
            "hb_buffer_create failed!"
        );
        Ok(HBBuffer { buffer: hb_buf })
    }

    pub fn guess_segments_properties(&mut self) {
        unsafe { hb_buffer_guess_segment_properties(self.buffer) };
    }

    pub fn add_utf8(&mut self, s: &[u8]) {
        unsafe {
            hb_buffer_add_utf8(
                self.buffer,
                s.as_ptr() as *const i8,
                s.len() as i32,
                0,
                s.len() as i32,
            );
        }
    }
    pub fn add_str(&mut self, s: &str) {
        self.add_utf8(s.as_bytes());
    }

    pub fn get_glyph_infos(&mut self) -> &[hb_glyph_info_t] {
        unsafe {
            let mut len: u32 = 0;
            let info = hb_buffer_get_glyph_infos(self.buffer, &mut len as *mut u32);
            slice::from_raw_parts(info, len as usize)
        }
    }

    pub fn get_glyph_positions(&mut self) -> &[hb_glyph_position_t] {
        unsafe {
            let mut len: u32 = 0;
            let info = hb_buffer_get_glyph_positions(self.buffer, &mut len as *mut u32);
            slice::from_raw_parts(info, len as usize)
        }
    }
}
