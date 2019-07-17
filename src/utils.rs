use failure::Error;
use image::imageops::{crop, resize};
use image::Pixel;
use image::{DynamicImage, FilterType, GenericImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;

#[cfg(target_os = "linux")]
use {image::ImageOutputFormat, std::process::Command};

pub trait ToRgba {
    type Target;
    fn to_rgba(&self) -> Self::Target;
}

impl ToRgba for str {
    type Target = Result<Rgba<u8>, std::num::ParseIntError>;
    fn to_rgba(&self) -> Self::Target {
        let rgb = u32::from_str_radix(&self[1..], 16)?;
        Ok(Rgba([
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            0xff,
        ]))
    }
}

impl ToRgba for syntect::highlighting::Color {
    type Target = Rgba<u8>;
    fn to_rgba(&self) -> Self::Target {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

pub fn add_window_controls(image: &mut DynamicImage) {
    let color = [
        ("#FF5F56", "#E0443E"),
        ("#FFBD2E", "#DEA123"),
        ("#27C93F", "#1AAB29"),
    ];

    let mut background = image.get_pixel(37, 37);
    background.data[3] = 0;

    let mut title_bar = RgbaImage::from_pixel(120 * 3, 40 * 3, background);

    for (i, (fill, outline)) in color.iter().enumerate() {
        draw_filled_circle_mut(
            &mut title_bar,
            (((i * 40) as i32 + 20) * 3, 20 * 3),
            11 * 3,
            outline.to_rgba().unwrap(),
        );
        draw_filled_circle_mut(
            &mut title_bar,
            (((i * 40) as i32 + 20) * 3, 20 * 3),
            10 * 3,
            fill.to_rgba().unwrap(),
        );
    }
    // create a big image and resize it to blur the edge
    // it looks better than `blur()`
    let title_bar = resize(&title_bar, 120, 40, FilterType::Triangle);

    copy_alpha(&title_bar, image.as_mut_rgba8().unwrap(), 15, 15);
}

/// Add the shadow for image
#[derive(Debug)]
pub struct ShadowAdder {
    background: Rgba<u8>,
    shadow_color: Rgba<u8>,
    blur_radius: f32,
    pad_horiz: u32,
    pad_vert: u32,
    offset_x: i32,
    offset_y: i32,
}

impl ShadowAdder {
    pub fn new() -> Self {
        Self {
            background: "#abb8c3".to_rgba().unwrap(),
            shadow_color: "#707070".to_rgba().unwrap(),
            blur_radius: 50.0,
            pad_horiz: 80,
            pad_vert: 100,
            offset_x: 0,
            offset_y: 0,
        }
    }

    /// Set the background color
    pub fn background(mut self, color: Rgba<u8>) -> Self {
        self.background = color;
        self
    }

    /// Set the shadow color
    pub fn shadow_color(mut self, color: Rgba<u8>) -> Self {
        self.shadow_color = color;
        self
    }

    /// Set the shadow size
    pub fn blur_radius(mut self, sigma: f32) -> Self {
        self.blur_radius = sigma;
        self
    }

    ///
    pub fn pad_horiz(mut self, pad: u32) -> Self {
        self.pad_horiz = pad;
        self
    }

    pub fn pad_vert(mut self, pad: u32) -> Self {
        self.pad_vert = pad;
        self
    }

    pub fn offset_x(mut self, offset: i32) -> Self {
        self.offset_x = offset;
        self
    }

    pub fn offset_y(mut self, offset: i32) -> Self {
        self.offset_y = offset;
        self
    }

    pub fn apply_to(&self, image: &DynamicImage) -> DynamicImage {
        // the size of the final image
        let width = image.width() + self.pad_horiz * 2;
        let height = image.height() + self.pad_vert * 2;

        // create the shadow
        let mut shadow = RgbaImage::from_pixel(width, height, self.background);

        if self.blur_radius > 0.0 {
            let rect = Rect::at(
                self.pad_horiz as i32 + self.offset_x,
                self.pad_vert as i32 + self.offset_y,
            )
            .of_size(image.width(), image.height());

            draw_filled_rect_mut(&mut shadow, rect, self.shadow_color);

            shadow = crate::blur::gaussian_blur(shadow, self.blur_radius);
        }
        // it's to slow!
        // shadow = blur(&shadow, self.blur_radius);

        // copy the original image to the top of it
        copy_alpha(
            image.as_rgba8().unwrap(),
            &mut shadow,
            self.pad_horiz,
            self.pad_vert,
        );

        DynamicImage::ImageRgba8(shadow)
    }
}

/// copy from src to dst, taking into account alpha channels
pub(crate) fn copy_alpha(src: &RgbaImage, dst: &mut RgbaImage, x: u32, y: u32) {
    assert!(src.width() + x <= dst.width());
    assert!(src.height() + y <= dst.height());
    for j in 0..src.height() {
        for i in 0..src.width() {
            unsafe {
                let s = src.unsafe_get_pixel(i, j);
                let mut d = dst.unsafe_get_pixel(i + x, j + y);
                match s.data[3] {
                    255 => d = s,
                    0 => (/* do nothing */),
                    _ => d.blend(&s),
                }
                dst.unsafe_put_pixel(i + x, j + y, d);
            }
        }
    }
}

/// Round the corner of the image
pub fn round_corner(image: &mut DynamicImage, radius: u32) {
    // draw a circle with given foreground on given background
    // then split it into four pieces and paste them to the four corner of the image
    let mut circle =
        RgbaImage::from_pixel(radius * 2 + 1, radius * 2 + 1, Rgba([255, 255, 255, 0]));

    let width = image.width();
    let height = image.height();

    let foreground = image.get_pixel(width - 1, height - 1);

    // TODO: need a blur on edge
    draw_filled_circle_mut(
        &mut circle,
        (radius as i32, radius as i32),
        radius as i32,
        foreground,
    );

    let part = crop(&mut circle, 0, 0, radius, radius);
    image.copy_from(&part, 0, 0);

    let part = crop(&mut circle, radius + 1, 0, radius, radius);
    image.copy_from(&part, width - radius, 0);

    let part = crop(&mut circle, 0, radius + 1, radius, radius);
    image.copy_from(&part, 0, height - radius);

    let part = crop(&mut circle, radius + 1, radius + 1, radius, radius);
    image.copy_from(&part, width - radius, height - radius);
}

// `draw_filled_circle_mut` doesn't work well with small radius in imageproc v0.18.0
// it has been fixed but still have to wait for releasing
// issue: https://github.com/image-rs/imageproc/issues/328
// PR: https://github.com/image-rs/imageproc/pull/330
/// Draw as much of a circle, including its contents, as lies inside the image bounds.
pub(crate) fn draw_filled_circle_mut<I>(image: &mut I, center: (i32, i32), radius: i32, color: I::Pixel)
where
    I: GenericImage,
    I::Pixel: 'static,
{
    let mut x = 0i32;
    let mut y = radius;
    let mut p = 1 - radius;
    let x0 = center.0;
    let y0 = center.1;

    while x <= y {
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

#[cfg(target_os = "linux")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::PNG)?;
    Command::new("xclip")
        .args(&[
            "-sel",
            "clip",
            "-t",
            "image/png",
            temp.path().to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e))?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn dump_image_to_clipboard(_image: &DynamicImage) -> Result<(), Error> {
    Err(format_err!(
        "This feature hasn't been implemented for your system"
    ))
}
