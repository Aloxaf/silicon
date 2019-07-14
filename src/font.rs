use conv::ValueInto;
use euclid::{Point2D, Rect, Size2D};
use failure::Error;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use image::{GenericImage, Pixel};
use imageproc::definitions::Clamp;
use imageproc::pixelops::weighted_sum;
use std::collections::HashMap;
use std::sync::Arc;
use syntect::highlighting;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FontStyle {
    REGULAR,
    ITALIC,
    BOLD,
    BOLDITALIC,
}

impl From<highlighting::FontStyle> for FontStyle {
    fn from(style: highlighting::FontStyle) -> Self {
        if style.contains(highlighting::FontStyle::BOLD) {
            if style.contains(highlighting::FontStyle::ITALIC) {
                BOLDITALIC
            } else {
                BOLD
            }
        } else if style.contains(highlighting::FontStyle::ITALIC) {
            ITALIC
        } else {
            REGULAR
        }
    }
}

use FontStyle::*;

#[derive(Debug)]
pub struct ImageFont {
    pub fonts: HashMap<FontStyle, Font>,
    pub size: f32,
}

impl ImageFont {
    pub fn new(name: &str, size: f32) -> Result<Self, Error> {
        let mut fonts = HashMap::new();

        let family = SystemSource::new().select_family_by_name(name)?;
        let handles = family.fonts();

        debug!("{:?}", handles);

        for handle in handles {
            let font = handle.load()?;

            let properties: Properties = font.properties();

            debug!("{:?} - {:?}", font, properties);

            // cannot use match because `Weight` didn't derive `Eq`
            match properties.style {
                Style::Normal => {
                    if properties.weight == Weight::NORMAL {
                        fonts.insert(REGULAR, font);
                    } else if properties.weight == Weight::BOLD {
                        fonts.insert(BOLD, font);
                    }
                }
                Style::Italic => {
                    if properties.weight == Weight::NORMAL {
                        fonts.insert(ITALIC, font);
                    } else if properties.weight == Weight::BOLD {
                        fonts.insert(BOLDITALIC, font);
                    }
                }
                _ => (),
            }
        }

        Ok(Self { fonts, size })
    }

    pub fn get_by_style(&self, style: FontStyle) -> &Font {
        self.fonts
            .get(&style)
            .unwrap_or_else(|| self.fonts.get(&REGULAR).unwrap())
    }

    pub fn get_reaular(&self) -> &Font {
        self.fonts.get(&REGULAR).unwrap()
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    /// get the (width, height) of font
    pub fn get_size(&self) -> (u32, u32) {
        self.get_char_size('M')
    }

    /// get the (width, height) of a char
    pub fn get_char_size(&self, c: char) -> (u32, u32) {
        let metrics = self.get_reaular().metrics();
        let advance = self
            .get_reaular()
            .advance(self.get_reaular().glyph_for_char(c).unwrap())
            .unwrap();

        let width = (advance / metrics.units_per_em as f32 * self.size).x.ceil() as u32;
        let height = ((metrics.ascent - metrics.descent) / metrics.units_per_em as f32 * self.size)
            .ceil() as u32;

        (width, height)
    }
}

impl Default for ImageFont {
    fn default() -> Self {
        let l = vec![
            (
                REGULAR,
                include_bytes!("../assets/fonts/Hack-Regular.ttf").to_vec(),
            ),
            (
                ITALIC,
                include_bytes!("../assets/fonts/Hack-Italic.ttf").to_vec(),
            ),
            (
                BOLD,
                include_bytes!("../assets/fonts/Hack-Bold.ttf").to_vec(),
            ),
            (
                BOLDITALIC,
                include_bytes!("../assets/fonts/Hack-BoldItalic.ttf").to_vec(),
            ),
        ];
        let mut fonts = HashMap::new();
        for (style, bytes) in l {
            let font = Font::from_bytes(Arc::new(bytes), 0).unwrap();
            fonts.insert(style, font);
        }

        Self { fonts, size: 26.0 }
    }
}

struct Glyph {
    id: u32,
    raster_rect: Rect<i32>,
    position: Point2D<i32>,
}

fn get_font_size(font: &Font, size: f32) -> Size2D<u32> {
    let metrics = font.metrics();
    let advance = font.advance(font.glyph_for_char('M').unwrap()).unwrap();
    let height =
        ((metrics.ascent - metrics.descent) / metrics.units_per_em as f32 * size).ceil() as u32;
    let width = (advance / metrics.units_per_em as f32 * size).x.ceil() as u32;
    Size2D::new(width, height)
}

fn get_layout(font: &Font, text: &str, size: f32) -> Vec<Option<Glyph>> {
    let font_size = get_font_size(font, size);

    text.chars()
        .enumerate()
        .map(|(i, c)| {
            font.glyph_for_char(c).map(|glyph_id| {
                let raster_rect = font
                    .raster_bounds(
                        glyph_id,
                        size,
                        &FontTransform::identity(),
                        &Point2D::zero(),
                        HintingOptions::None,
                        RasterizationOptions::GrayscaleAa,
                    )
                    .unwrap();
                let x = i as i32 * font_size.width as i32 + raster_rect.origin.x;
                let y = font_size.height as i32 - raster_rect.size.height - raster_rect.origin.y;

                Glyph {
                    id: glyph_id,
                    position: Point2D::new(x, y),
                    raster_rect,
                }
            })
        })
        .collect()
}

impl Glyph {
    // TODO: keep a copy/reference of these arguments in Glyph struct ?
    fn draw<O: FnMut(u32, u32, f32)>(&self, font: &Font, size: f32, offset: i32, mut o: O) {
        let mut canvas = Canvas::new(&self.raster_rect.size.to_u32(), Format::A8);

        let origin = Point2D::new(
            -self.raster_rect.origin.x,
            self.raster_rect.size.height + self.raster_rect.origin.y,
        )
        .to_f32();

        font.rasterize_glyph(
            &mut canvas,
            self.id,
            size,
            &FontTransform::identity(),
            &origin,
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        )
        .unwrap();

        for y in (0..self.raster_rect.size.height).rev() {
            let (row_start, row_end) =
                (y as usize * canvas.stride, (y + 1) as usize * canvas.stride);
            let row = &canvas.pixels[row_start..row_end];

            for x in 0..self.raster_rect.size.width {
                let val = f32::from(row[x as usize]) / 255.0;
                let px = self.position.x + x;
                let py = self.position.y + y + offset;

                o(px as u32, py as u32, val);
            }
        }
    }
}

// edit from https://github.com/wezm/profont
pub fn draw_text_mut<I>(
    image: &mut I,
    color: I::Pixel,
    x: u32,
    y: u32,
    size: f32,
    font: &Font,
    text: &str,
) where
    I: GenericImage,
    <I::Pixel as Pixel>::Subpixel: ValueInto<f32> + Clamp<f32>,
{
    let metrics = font.metrics();
    let offset = (metrics.descent / metrics.units_per_em as f32 * size).round() as i32;

    let glyphs = get_layout(font, text, size);

    for glyph in glyphs {
        if let Some(glyph) = glyph {
            glyph.draw(font, size, offset, |px, py, v| {
                if v <= std::f32::EPSILON {
                    return;
                }
                let (x, y) = (px + x, py + y);
                let pixel = image.get_pixel(x, y);
                let weighted_color = weighted_sum(pixel, color, 1.0 - v, v);
                image.put_pixel(x, y, weighted_color);
            });
        }
    }
}
