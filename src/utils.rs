use crate::error::ParseColorError;
use image::imageops::{crop_imm, resize, FilterType};
use image::Pixel;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;

pub trait ToRgba {
    type Target;
    fn to_rgba(&self) -> Self::Target;
}

/// Parse hex color (#RRGGBB or #RRGGBBAA)
impl ToRgba for str {
    type Target = Result<Rgba<u8>, ParseColorError>;

    fn to_rgba(&self) -> Self::Target {
        if self.as_bytes()[0] != b'#' {
            return Err(ParseColorError::InvalidDigit);
        }
        let mut color = u32::from_str_radix(&self[1..], 16)?;

        match self.len() {
            // RGB or RGBA
            4 | 5 => {
                let a = if self.len() == 5 {
                    let alpha = (color & 0xf) as u8;
                    color >>= 4;
                    alpha
                } else {
                    0xff
                };

                let r = ((color >> 8) & 0xf) as u8;
                let g = ((color >> 4) & 0xf) as u8;
                let b = (color & 0xf) as u8;

                Ok(Rgba([r << 4 | r, g << 4 | g, b << 4 | b, a << 4 | a]))
            }
            // RRGGBB or RRGGBBAA
            7 | 9 => {
                let alpha = if self.len() == 9 {
                    let alpha = (color & 0xff) as u8;
                    color >>= 8;
                    alpha
                } else {
                    0xff
                };

                Ok(Rgba([
                    (color >> 16) as u8,
                    (color >> 8) as u8,
                    color as u8,
                    alpha,
                ]))
            }
            _ => Err(ParseColorError::InvalidLength),
        }
    }
}

impl ToRgba for syntect::highlighting::Color {
    type Target = Rgba<u8>;
    fn to_rgba(&self) -> Self::Target {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

pub struct WindowControlsParams {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub radius: u32,
}

/// Add the window controls for image
pub(crate) fn add_window_controls(image: &mut DynamicImage, params: &WindowControlsParams) {
    let color = [
        ("#FF5F56", "#E0443E"),
        ("#FFBD2E", "#DEA123"),
        ("#27C93F", "#1AAB29"),
    ];

    let mut background = image.get_pixel(37, 37);
    background.0[3] = 0;

    let mut title_bar = RgbaImage::from_pixel(params.width * 3, params.height * 3, background);
    let step = (params.radius * 2) as i32;
    let spacer = step * 2;
    let center_y = (params.height / 2) as i32;

    for (i, (fill, outline)) in color.iter().enumerate() {
        draw_filled_circle_mut(
            &mut title_bar,
            ((i as i32 * spacer + step) * 3, center_y * 3),
            (params.radius + 1) as i32 * 3,
            outline.to_rgba().unwrap(),
        );
        draw_filled_circle_mut(
            &mut title_bar,
            ((i as i32 * spacer + step) * 3, center_y * 3),
            params.radius as i32 * 3,
            fill.to_rgba().unwrap(),
        );
    }
    // create a big image and resize it to blur the edge
    // it looks better than `blur()`
    let title_bar = resize(
        &title_bar,
        params.width,
        params.height,
        FilterType::Triangle,
    );

    copy_alpha(
        &title_bar,
        image.as_mut_rgba8().unwrap(),
        params.padding,
        params.padding,
    );
}

#[derive(Clone, Debug)]
pub enum Background {
    Solid(Rgba<u8>),
    Image(RgbaImage),
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid("#abb8c3".to_rgba().unwrap())
    }
}

impl Background {
    fn to_image(&self, width: u32, height: u32) -> RgbaImage {
        match self {
            Background::Solid(color) => RgbaImage::from_pixel(width, height, color.to_owned()),
            Background::Image(image) => resize(image, width, height, FilterType::Triangle),
        }
    }
}

/// Add the shadow for image
#[derive(Debug)]
pub struct ShadowAdder {
    background: Background,
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
            background: Background::default(),
            shadow_color: "#707070".to_rgba().unwrap(),
            blur_radius: 50.0,
            pad_horiz: 80,
            pad_vert: 100,
            offset_x: 0,
            offset_y: 0,
        }
    }

    /// Set the background color
    pub fn background(mut self, bg: Background) -> Self {
        self.background = bg;
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
        let mut shadow = self.background.to_image(width, height);
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

impl Default for ShadowAdder {
    fn default() -> Self {
        ShadowAdder::new()
    }
}

/// copy from src to dst, taking into account alpha channels
pub(crate) fn copy_alpha(src: &RgbaImage, dst: &mut RgbaImage, x: u32, y: u32) {
    assert!(src.width() + x <= dst.width());
    assert!(src.height() + y <= dst.height());
    for j in 0..src.height() {
        for i in 0..src.width() {
            // NOTE: Undeprecate in https://github.com/image-rs/image/pull/1008
            #[allow(deprecated)]
            unsafe {
                let s = src.unsafe_get_pixel(i, j);
                let mut d = dst.unsafe_get_pixel(i + x, j + y);
                match s.0[3] {
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
pub(crate) fn round_corner(image: &mut DynamicImage, radius: u32) {
    // draw a circle with given foreground on given background
    // then split it into four pieces and paste them to the four corner of the image
    //
    // the circle is drawn on a bigger image to avoid the aliasing
    // later it will be scaled to the correct size
    // we add +1 (to the radius) to make sure that there is also space for the border to mitigate artefacts when scaling
    // note that the +1 isn't added to the radius when drawing the circle
    let mut circle =
        RgbaImage::from_pixel((radius + 1) * 4, (radius + 1) * 4, Rgba([255, 255, 255, 0]));

    let width = image.width();
    let height = image.height();

    // use the bottom right pixel to get the color of the foreground
    let foreground = image.get_pixel(width - 1, height - 1);

    draw_filled_circle_mut(
        &mut circle,
        (((radius + 1) * 2) as i32, ((radius + 1) * 2) as i32),
        radius as i32 * 2,
        foreground,
    );

    // scale down the circle to the correct size
    let circle = resize(
        &circle,
        (radius + 1) * 2,
        (radius + 1) * 2,
        FilterType::Triangle,
    );

    // top left
    let part = crop_imm(&circle, 1, 1, radius, radius);
    image.copy_from(&*part, 0, 0).unwrap();

    // top right
    let part = crop_imm(&circle, radius + 1, 1, radius, radius - 1);
    image.copy_from(&*part, width - radius, 0).unwrap();

    // bottom left
    let part = crop_imm(&circle, 1, radius + 1, radius, radius);
    image.copy_from(&*part, 0, height - radius).unwrap();

    // bottom right
    let part = crop_imm(&circle, radius + 1, radius + 1, radius, radius);
    image
        .copy_from(&*part, width - radius, height - radius)
        .unwrap();
}

// `draw_filled_circle_mut` doesn't work well with small radius in imageproc v0.18.0
// it has been fixed but still have to wait for releasing
// issue: https://github.com/image-rs/imageproc/issues/328
// PR: https://github.com/image-rs/imageproc/pull/330
/// Draw as much of a circle, including its contents, as lies inside the image bounds.
pub(crate) fn draw_filled_circle_mut<I>(
    image: &mut I,
    center: (i32, i32),
    radius: i32,
    color: I::Pixel,
) where
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

#[cfg(test)]
mod tests {
    use crate::utils::ToRgba;
    use image::Rgba;

    #[test]
    fn to_rgba() {
        assert_eq!("#abcdef".to_rgba(), Ok(Rgba([0xab, 0xcd, 0xef, 0xff])));
        assert_eq!("#abcdef00".to_rgba(), Ok(Rgba([0xab, 0xcd, 0xef, 0x00])));
        assert_eq!("#abc".to_rgba(), Ok(Rgba([0xaa, 0xbb, 0xcc, 0xff])));
        assert_eq!("#abcd".to_rgba(), Ok(Rgba([0xaa, 0xbb, 0xcc, 0xdd])));
    }
}
