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
