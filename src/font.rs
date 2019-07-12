use failure::Error;
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use syntect::highlighting::FontStyle;

use conv::ValueInto;
use euclid::{Point2D, Rect, Size2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use image::{GenericImage, Pixel};
use imageproc::definitions::Clamp;
use imageproc::pixelops::weighted_sum;
use std::sync::Arc;

#[derive(Debug)]
pub struct ImageFont {
    pub regular: Font,
    pub italic: Font,
    pub bold: Font,
    pub bolditalic: Font,
    pub size: f32,
}

impl ImageFont {
    pub fn new(font: &str, size: f32) -> Result<Self, Error> {
        let regular = Self::search_font(font, &Properties::new())?;
        let regular = regular.load()?;

        debug!("regular: {:?}", regular);

        if !regular.is_monospace() {
            eprintln!("[warning] You're using a non-monospace font");
        }

        let italic = Self::search_font(font, &Properties::new().style(Style::Italic))?;
        let italic = italic.load()?;

        debug!("italic: {:?}", italic);

        let bold = Self::search_font(font, &Properties::new().weight(Weight::BOLD))?;
        let bold = bold.load()?;

        debug!("bold: {:?}", bold);

        let bolditalic = Self::search_font(
            font,
            &Properties::new().style(Style::Italic).weight(Weight::BOLD),
        )?;
        let bolditalic = bolditalic.load()?;

        debug!("bolditalic: {:?}", bolditalic);

        Ok(Self {
            regular,
            italic,
            bold,
            bolditalic,
            size,
        })
    }

    pub fn get_by_style(&self, style: &syntect::highlighting::Style) -> &Font {
        if style.font_style.contains(FontStyle::BOLD) {
            if style.font_style.contains(FontStyle::ITALIC) {
                &self.bolditalic
            } else {
                &self.bold
            }
        } else if style.font_style.contains(FontStyle::ITALIC) {
            &self.italic
        } else {
            &self.regular
        }
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    /// search a font by it's family name and properties
    fn search_font(family_name: &str, properties: &Properties) -> Result<Handle, Error> {
        let family_names = [FamilyName::Title(family_name.to_owned())];
        Ok(SystemSource::new().select_best_match(&family_names, properties)?)
    }

    pub fn from_bytes(
        regular: Vec<u8>,
        italic: Vec<u8>,
        bold: Vec<u8>,
        bolditalic: Vec<u8>,
        size: f32,
    ) -> Result<Self, Error> {
        let regular = Font::from_bytes(Arc::new(regular), 0)?;
        let italic = Font::from_bytes(Arc::new(italic), 0)?;
        let bold = Font::from_bytes(Arc::new(bold), 0)?;
        let bolditalic = Font::from_bytes(Arc::new(bolditalic), 0)?;

        Ok(Self {
            regular,
            bolditalic,
            italic,
            bold,
            size,
        })
    }

    /// get the (width, height) of font
    pub fn get_size(&self) -> (u32, u32) {
        self.get_char_size('M')
    }

    /// get the (width, height) of a char
    pub fn get_char_size(&self, c: char) -> (u32, u32) {
        let metrics = self.regular.metrics();
        let advance = self
            .regular
            .advance(self.regular.glyph_for_char(c).unwrap())
            .unwrap();

        let width = (advance / metrics.units_per_em as f32 * self.size).x.ceil() as u32;
        let height = ((metrics.ascent - metrics.descent) / metrics.units_per_em as f32 * self.size)
            .ceil() as u32;

        (width, height)
    }

    pub fn get_scale(&self) -> f32 {
        self.size
    }
}

struct Glyph {
    id: u32,
    raster_rect: Rect<i32>,
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
    let offset = (metrics.descent as f32 / metrics.units_per_em as f32 * size).round() as i32;

    let glyphs = text
        .chars()
        .map(|c| {
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
                Glyph {
                    id: glyph_id,
                    raster_rect,
                }
            })
        })
        .collect::<Vec<_>>();

    let metrics = font.metrics();
    let advance = font.advance(font.glyph_for_char('M').unwrap()).unwrap();

    let height =
        ((metrics.ascent - metrics.descent) / metrics.units_per_em as f32 * size).ceil() as u32;
    let width = (advance / metrics.units_per_em as f32 * size).x.ceil() as u32;
    let char_size = Size2D::new(width, height);

    for (i, glyph) in glyphs.iter().enumerate() {
        // TODO: None char ?
        if let Some(glyph) = glyph {
            // TODO: only alloc once?
            let mut canvas = Canvas::new(&glyph.raster_rect.size.to_u32(), Format::A8);

            let origin = Point2D::new(
                -glyph.raster_rect.origin.x,
                glyph.raster_rect.size.height + glyph.raster_rect.origin.y,
            )
            .to_f32();

            font.rasterize_glyph(
                &mut canvas,
                glyph.id,
                size,
                &FontTransform::identity(),
                &origin,
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )
            .unwrap();

            let img_x = i as u32 * char_size.width;
            let img_y = 0 * char_size.height + char_size.height;

            let iy = y;
            let ix = x;
            for y in (0..glyph.raster_rect.size.height as u32).rev() {
                let (row_start, row_end) =
                    (y as usize * canvas.stride, (y + 1) as usize * canvas.stride);
                let row = &canvas.pixels[row_start..row_end];
                for x in 0..glyph.raster_rect.size.width as u32 {
                    let val = row[x as usize];
                    if val != 0 {
                        let pixel_x = img_x as i32 + x as i32 + glyph.raster_rect.origin.x;
                        let pixel_y = img_y as i32 - glyph.raster_rect.size.height + y as i32
                            - glyph.raster_rect.origin.y
                            + offset;

                        if pixel_x >= 0 && pixel_y >= 0 {
                            let pixel = image.get_pixel(ix + pixel_x as u32, iy + pixel_y as u32);
                            let weighted_color = weighted_sum(
                                pixel,
                                color,
                                1.0 - val as f32 / 255.0,
                                val as f32 / 255.0,
                            );
                            image.put_pixel(
                                ix + pixel_x as u32,
                                iy + pixel_y as u32,
                                weighted_color,
                            );
                        }
                    }
                }
            }
        }
    }
}
