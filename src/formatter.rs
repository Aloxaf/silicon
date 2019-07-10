use crate::font::ImageFont;
use crate::utils::ToRgba;
use failure::Error;
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::Font;
use syntect::highlighting::{Color, Style, Theme};
use itertools::Itertools;

pub struct ImageFormatter<'a> {
    /// pad between lines
    /// Default: 2
    line_pad: u32,
    /// pad between code and edge of code area. [top, bottom, left, right]
    /// Default: 25
    code_pad: u32,
    /// pad of top of the code area
    /// Default: 50
    code_pad_top: u32,
    /// show line number
    /// Default: true
    line_number: bool,
    /// pad between code and line number
    /// Default: 6
    line_number_pad: u32,
    /// number of columns of line number area
    /// Default: Auto detect
    line_number_chars: u32,
    /// font of english character, should be mono space font
    /// Default: Hack (builtin)
    font: ImageFont<'a>,
    /// font size of `.font`. (width, height)
    font_size: (u32, u32),
    /// font of cjk character
    /// Default: None
    cjk_font: Option<ImageFont<'a>>,
    /// font size of `.cjk_font`. (width, height)
    cjk_font_size: (u32, u32),
}

pub struct ImageFormatterBuilder<'a> {
    /// pad between lines
    line_pad: u32,
    /// show line number
    line_number: bool,
    /// pad of top of the code area
    code_pad_top: u32,
    /// font of english character, should be mono space font
    font: ImageFont<'a>,
    /// font size of `.font`. (width, height)
    font_size: (u32, u32),
    /// font of cjk character
    cjk_font: Option<ImageFont<'a>>,
    /// font size of `.cjk_font`. (width, height)
    cjk_font_size: (u32, u32),
}

impl<'a> ImageFormatterBuilder<'a> {
    pub fn new() -> Self {
        let font = ImageFont::from_bytes(
            include_bytes!("../assets/fonts/Hack-Regular.ttf"),
            include_bytes!("../assets/fonts/Hack-Italic.ttf"),
            include_bytes!("../assets/fonts/Hack-Bold.ttf"),
            26.0,
        )
        .unwrap();
        Self {
            line_pad: 2,
            line_number: true,
            code_pad_top: 50,
            font_size: font.get_size(),
            cjk_font: None,
            cjk_font_size: (0, 0),
            font,
        }
    }

    pub fn line_number(mut self, show: bool) -> Self {
        self.line_number = show;
        self
    }

    pub fn line_pad(mut self, pad: u32) -> Self {
        self.line_pad = pad;
        self
    }

    pub fn code_pad_top(mut self, pad: u32) -> Self {
        self.code_pad_top = pad;
        self
    }

    pub fn font(mut self, name: &str, size: f32) -> Result<Self, Error> {
        let font = ImageFont::new(name, size)
            .map_err(|e| format_err!("Cannot load font `{}`: {}", name, e))?;
        self.font_size = font.get_size();
        self.font = font;
        Ok(self)
    }

    pub fn cjk_font(mut self, name: &str, size: f32) -> Result<Self, Error> {
        let font = ImageFont::new(name, size)
            .map_err(|e| format_err!("Cannot load font `{}`: {}", name, e))?;
        self.cjk_font_size = font.get_size();
        self.cjk_font = Some(font);
        Ok(self)
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font.set_size(size);
        // TODO: font_size 和 font_size 命名
        self.font_size = self.font.get_size();
        self
    }

    pub fn cjk_font_size(mut self, size: f32) -> Self {
        if let Some(cjk_font) = &mut self.cjk_font {
            cjk_font.set_size(size);
            self.cjk_font_size = cjk_font.get_size();
        }
        self
    }

    pub fn build(mut self) -> ImageFormatter<'a> {
        ImageFormatter {
            line_pad: self.line_pad,
            code_pad: 25,
            code_pad_top: self.code_pad_top,
            line_number: self.line_number,
            line_number_pad: 6,
            line_number_chars: 0,
            font_size: self.font_size,
            cjk_font_size: self.cjk_font_size,
            cjk_font: std::mem::replace(&mut self.cjk_font, None),
            font: self.font,
        }
    }
}

struct Drawable<'a> {
    /// max width of the picture
    max_width: u32,
    /// max number of line of the picture
    max_lineno: u32,
    drawables: Vec<(u32, u32, Color, &'a Font<'a>, String)>,
}

impl<'a> ImageFormatter<'a> {
    /// calculate the X coordinate after some number of characters
    fn get_char_x(&self, charno: u32, cjk_charno: u32) -> u32 {
        charno * self.font_size.0
            + cjk_charno * self.cjk_font_size.0
            + self.code_pad
            + if self.line_number {
                self.font_size.0 * self.line_number_chars + 2 * self.line_number_pad
            } else {
                0
            }
    }

    /// calculate the height of a line
    fn get_line_height(&self) -> u32 {
        self.font_size.1 + self.line_pad
    }

    /// calculate the Y coordinate of a line
    fn get_line_y(&self, lineno: u32) -> u32 {
        lineno * self.get_line_height() + self.code_pad + self.code_pad_top
    }

    /// calculate the coordinate of text
    fn _get_text_pos(&self, charno: u32, cjk_charno: u32, lineno: u32) -> (u32, u32) {
        (self.get_char_x(charno, cjk_charno), self.get_line_y(lineno))
    }

    /// calculate the size of code area
    fn get_image_size(&self, max_width: u32, lineno: u32) -> (u32, u32) {
        (
            max_width + self.code_pad,
            self.get_line_y(lineno + 1) + self.code_pad,
        )
    }

    /// create
    fn create_drawables<'b>(&'b self, v: &'b [Vec<(Style, &'a str)>]) -> Drawable<'b>
    where
        'a: 'b,
    {
        let mut drawables = vec![];
        let (mut max_width, mut max_lineno) = (0, 0);

        for (i, tokens) in v.iter().enumerate() {
            let height = self.get_line_y(i as u32);
            let (mut charno, mut cjk_charno) = (0, 0);

            for (style, text) in tokens {
                // render ASCII character and non-ASCII character with different font
                for (_k, group) in &text.chars().group_by(|c| c.is_ascii()) {
                    // '\n' can't be draw to image
                    let text = group.collect::<String>();
                    let text: &str = text.trim_end_matches('\n');

                    let width = self.get_char_x(charno, cjk_charno);

                    if text.is_empty() {
                        continue;
                    }

                    if text.as_bytes()[0].is_ascii() ||self.cjk_font.is_none()  {
                        // let text = text.trim_end_matches('\n');
                        let font = self.font.get_by_style(style);

                        drawables.push((width, height, style.foreground, font, text.to_owned()));
                        // TODO: UNDERLINE & combine of these

                        charno += text.len() as u32;
                    } else if let Some(cjk_font) = &self.cjk_font {
                        let font = cjk_font.get_by_style(style);

                        drawables.push((width, height, style.foreground, font, text.to_owned()));
                        cjk_charno += text.chars().count() as u32;
                    }
                }
                max_width = max_width.max(self.get_char_x(charno, cjk_charno));
            }
            max_lineno = i as u32;
        }

        Drawable {
            max_width,
            max_lineno,
            drawables,
        }
    }

    fn draw_line_number(&self, image: &mut DynamicImage, lineno: u32, color: Rgba<u8>) {
        for i in 0..=lineno {
            let line_mumber = format!("{:>width$}", i + 1, width = self.line_number_chars as usize);
            draw_text_mut(
                image,
                color,
                self.code_pad,
                self.get_line_y(i),
                self.font.get_scale(),
                &self.font,
                &line_mumber,
            );
        }
    }

    // TODO: &mut ?
    pub fn format(&mut self, v: &[Vec<(Style, &str)>], theme: &Theme) -> DynamicImage {
        if self.line_number {
            self.line_number_chars = (v.len() as f32).log10().ceil() as u32;
        } else {
            self.line_number_chars = 0;
            self.line_number_pad = 0;
        }

        let drawables = self.create_drawables(v);

        let size = self.get_image_size(drawables.max_width, drawables.max_lineno);

        let foreground = theme.settings.foreground.unwrap();
        let background = theme.settings.background.unwrap();

        let foreground = foreground.to_rgba();
        let background = background.to_rgba();

        let mut image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(size.0, size.1, background));

        if self.line_number {
            self.draw_line_number(&mut image, drawables.max_lineno, foreground);
        }

        for (x, y, color, font, text) in drawables.drawables {
            let color = color.to_rgba();
            draw_text_mut(&mut image, color, x, y, self.font.get_scale(), &font, &text);
        }

        image
    }
}
