use failure::Error;
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use rusttype::{Font, FontCollection, Scale};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use syntect::highlighting::FontStyle;

pub struct ImageFont<'a> {
    normal: Font<'a>,
    italic: Font<'a>,
    bold: Font<'a>,
    size: f32,
}

impl<'a> ImageFont<'a> {
    pub fn new(font: &str, size: f32) -> Result<Self, Error> {
        let normal = Self::search_font(font, &Properties::new())?;
        let normal = Self::load_font(normal)?;

        let italic = Self::search_font(font, &Properties::new().style(Style::Italic))?;
        let italic = Self::load_font(italic)?;

        let bold = Self::search_font(font, &Properties::new().weight(Weight::BOLD))?;
        let bold = Self::load_font(bold)?;

        Ok(Self {
            normal,
            italic,
            bold,
            size,
        })
    }

    pub fn get_by_style(&self, style: &syntect::highlighting::Style) -> &Font<'a> {
        if style.font_style.contains(FontStyle::BOLD) {
            &self.bold
        } else if style.font_style.contains(FontStyle::ITALIC) {
            &self.italic
        } else {
            &self.normal
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

    /// load font from path
    fn load_font(handle: Handle) -> Result<Font<'a>, Error> {
        if let Handle::Path { path, .. } = handle {
            let mut file = File::open(path)?;
            let mut bytes = vec![];

            file.read_to_end(&mut bytes)?;
            // may contain multiple fonts
            Ok(FontCollection::from_bytes(bytes)?
                .into_fonts()
                .next()
                .unwrap()?)
        } else {
            unreachable!()
        }
    }

    pub fn from_bytes(
        normal: &'a [u8],
        italic: &'a [u8],
        bold: &'a [u8],
        size: f32,
    ) -> Result<Self, Error> {
        let normal = FontCollection::from_bytes(normal)?.into_font()?;
        let italic = FontCollection::from_bytes(italic)?.into_font()?;
        let bold = FontCollection::from_bytes(bold)?.into_font()?;

        Ok(Self {
            normal,
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
        let scale = self.get_scale();
        let c = self.normal.glyph(c);
        let g = c.scaled(scale);

        let v_metrics = self.normal.v_metrics(scale);

        let width = g.h_metrics().advance_width.round() as u32;
        let height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        (width, height)
    }

    pub fn get_scale(&self) -> Scale {
        Scale::uniform(self.size)
    }
}

impl<'a> Deref for ImageFont<'a> {
    type Target = rusttype::Font<'a>;

    fn deref(&self) -> &Self::Target {
        &self.normal
    }
}
