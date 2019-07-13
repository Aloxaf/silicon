use conv::ValueInto;
use euclid::{Point2D, Rect};
use failure::Error;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::error::SelectionError;
use font_kit::family_handle::FamilyHandle;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use image::{GenericImage, Pixel};
use imageproc::definitions::Clamp;
use imageproc::pixelops::weighted_sum;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontType {
    REGULAR,
    BOLD,
    ITALIC,
    BOLDITALIC,
}

use FontType::*;

#[derive(Debug)]
pub struct ImageFont(HashMap<FontType, Font>);

impl ImageFont {
    pub fn new(name: &str) -> Result<Self, Error> {
        let mut fonts = HashMap::new();

        let family = Self::load_family(name)?;
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

        Ok(Self(fonts))
    }

    fn load_family(family_name: &str) -> Result<FamilyHandle, SelectionError> {
        SystemSource::new().select_family_by_name(family_name)
    }

    pub fn get_by_type(&self, t: FontType) -> &Font {
        self.0
            .get(&t)
            .unwrap_or_else(|| self.0.get(&REGULAR).unwrap())
    }
}

#[derive(Debug)]
pub struct FontCollection {
    fonts: Vec<ImageFont>,
}

impl FontCollection {
    pub fn new(font_list: &[&str]) -> Result<Self, Error> {
        let mut fonts = vec![];
        for name in font_list {
            match ImageFont::new(name) {
                Ok(font) => fonts.push(font),
                Err(err) => eprintln!("[warning] Error occurs when load font `{}`: {} ", name, err),
            }
        }
        Ok(Self { fonts })
    }

    fn glyph_for_char(&self, c: char, style: FontType) -> Option<(u32, &Font)> {
        for font in &self.fonts {
            let result = font.get_by_type(style);
            if let Some(id) = result.glyph_for_char(c) {
                return Some((id, result));
            }
        }
        None
    }

    fn layout(&self, text: &str, style: FontType, size: f32) -> Vec<PositionGlyph> {
        let mut delta_x = 0;

        text.chars()
            .filter_map(|c| {
                self.glyph_for_char(c, style).map(|(id, font)| {
                    let raster_rect = font
                        .raster_bounds(
                            id,
                            size,
                            &FontTransform::identity(),
                            &Point2D::zero(),
                            HintingOptions::None,
                            RasterizationOptions::GrayscaleAa,
                        )
                        .unwrap();
                    delta_x += raster_rect.size.width + raster_rect.origin.x;
                    let x = delta_x;
                    let y = raster_rect.size.height - raster_rect.origin.y;

                    dbg!((x, y));

                    PositionGlyph {
                        id,
                        font: font.clone(),
                        size,
                        raster_rect,
                        position: Point2D::new(x, y),
                    }
                })
            })
            .collect()
    }

    pub fn draw_text_mut<I>(
        &self,
        image: &mut I,
        color: I::Pixel,
        x: u32,
        y: u32,
        size: f32,
        style: FontType,
        text: &str,
    ) where
        I: GenericImage,
        <I::Pixel as Pixel>::Subpixel: ValueInto<f32> + Clamp<f32>,
    {
        let metrics = self.fonts[0].0.get(&REGULAR).unwrap().metrics();
        let offset = (metrics.descent / metrics.units_per_em as f32 * size).round() as i32;

        let glyphs = self.layout(text, style, size);

        dbg!(offset);

        for glyph in glyphs {
            glyph.draw(offset, |px, py, v| {
                if v <= std::f32::EPSILON {
                    return;
                }
                let (x, y) = (px + x, py + y);
                let pixel = image.get_pixel(x, y);
                let weighted_color = weighted_sum(pixel, color, 1.0 - v, v);
                image.put_pixel(x, y, weighted_color);
            })
        }
    }
}

struct PositionGlyph {
    id: u32,
    font: Font,
    size: f32,
    position: Point2D<i32>,
    raster_rect: Rect<i32>,
}

impl PositionGlyph {
    fn draw<O: FnMut(u32, u32, f32)>(&self, offset: i32, mut o: O) {
        let mut canvas = Canvas::new(&self.raster_rect.size.to_u32(), Format::A8);

        let origin = Point2D::new(
            -self.raster_rect.origin.x,
            self.raster_rect.size.height + self.raster_rect.origin.y,
        )
        .to_f32();

        self.font
            .rasterize_glyph(
                &mut canvas,
                self.id,
                self.size,
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

                println!("{} {} {}", px, py, val);

                o(px as u32, py as u32, val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::new_font::{FontCollection, FontType, ImageFont};
    use image::{Rgb, RgbImage};

    #[test]
    fn load_single_font() {
        let font = ImageFont::new("Hack");
        dbg!(font);
    }

    #[test]
    fn load_fonts() {
        let fonts = FontCollection::new(&["Hack", "SimSun", "Blobmoji", "xx"]);
        dbg!(fonts);
    }

    #[test]
    fn draw_text() {
        let fonts = FontCollection::new(&["Hack", "SimSun", "Blobmoji"]).unwrap();
        let mut image = RgbImage::new(200, 200);
        fonts.draw_text_mut(
            &mut image,
            Rgb([255, 0, 0]),
            0,
            0,
            27.0,
            FontType::REGULAR,
            "AB你好✨✨",
        );
        image.save("test.png").unwrap();
    }
}
