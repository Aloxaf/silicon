use crate::font::{FontCollection, FontStyle};
use crate::utils::{copy_alpha, ToRgba};
use failure::Error;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use syntect::highlighting::{Color, Style, Theme};

pub struct ImageFormatter {
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
    font: FontCollection,
    /// Highlight lines
    highlight_lines: Vec<u32>,
}

pub struct ImageFormatterBuilder {
    /// pad between lines
    line_pad: u32,
    /// show line number
    line_number: bool,
    /// pad of top of the code area
    code_pad_top: u32,
    /// font of english character, should be mono space font
    font: FontCollection,
    /// Highlight lines
    highlight_lines: Vec<u32>,
}

impl<'a> ImageFormatterBuilder {
    pub fn new() -> Self {
        Self {
            line_pad: 2,
            line_number: true,
            code_pad_top: 50,
            font: FontCollection::default(),
            highlight_lines: vec![],
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

    // TODO: move this Result to `build`
    pub fn font<S: AsRef<str>>(mut self, fonts: &[(S, f32)]) -> Result<Self, Error> {
        let font = FontCollection::new(fonts)?;
        self.font = font;
        Ok(self)
    }

    pub fn highlight_lines(mut self, lines: Vec<u32>) -> Self {
        self.highlight_lines = lines;
        self
    }

    pub fn build(self) -> ImageFormatter {
        ImageFormatter {
            line_pad: self.line_pad,
            code_pad: 25,
            code_pad_top: self.code_pad_top,
            line_number: self.line_number,
            line_number_pad: 6,
            line_number_chars: 0,
            font: self.font,
            highlight_lines: self.highlight_lines,
        }
    }
}

struct Drawable {
    /// max width of the picture
    max_width: u32,
    /// max number of line of the picture
    max_lineno: u32,
    /// arguments for draw_text_mut
    drawables: Vec<(u32, u32, Color, FontStyle, String)>,
}

impl ImageFormatter {
    /// calculate the height of a line
    fn get_line_height(&self) -> u32 {
        self.font.get_font_height() + self.line_pad
    }

    /// calculate the Y coordinate of a line
    fn get_line_y(&self, lineno: u32) -> u32 {
        lineno * self.get_line_height() + self.code_pad + self.code_pad_top
    }

    /// calculate the size of code area
    fn get_image_size(&self, max_width: u32, lineno: u32) -> (u32, u32) {
        (
            (max_width + self.code_pad).max(150),
            self.get_line_y(lineno + 1) + self.code_pad,
        )
    }

    /// Calculate where code start
    fn get_left_pad(&self) -> u32 {
        self.code_pad
            + if self.line_number {
                let tmp = format!("{:>width$}", 0, width = self.line_number_chars as usize);
                2 * self.line_number_pad + self.font.get_text_len(&tmp)
            } else {
                0
            }
    }

    /// create
    fn create_drawables(&self, v: &[Vec<(Style, &str)>]) -> Drawable {
        let mut drawables = vec![];
        let (mut max_width, mut max_lineno) = (0, 0);

        for (i, tokens) in v.iter().enumerate() {
            let height = self.get_line_y(i as u32);
            let mut width = self.get_left_pad();

            for (style, text) in tokens {
                let text = text.trim_end_matches('\n');
                if text.is_empty() {
                    continue;
                }

                drawables.push((
                    width,
                    height,
                    style.foreground,
                    style.font_style.into(),
                    text.to_owned(),
                ));

                width += self.font.get_text_len(text);

                max_width = max_width.max(width);
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
            self.font.draw_text_mut(
                image,
                color,
                self.code_pad,
                self.get_line_y(i),
                FontStyle::REGULAR,
                &line_mumber,
            );
        }
    }

    fn highlight_lines(&self, image: &mut DynamicImage, lines: &[u32]) {
        let width = image.width();
        let height = self.font.get_font_height() + self.line_pad;
        let mut color = image.get_pixel(20, 20);
        color
            .data
            .iter_mut()
            .for_each(|n| *n = (*n).saturating_add(20));

        let shadow = RgbaImage::from_pixel(width, height, color);

        for i in lines {
            let y = self.get_line_y(*i - 1);
            copy_alpha(&shadow, image.as_mut_rgba8().unwrap(), 0, y);
        }
    }

    // TODO: &mut ?
    pub fn format(&mut self, v: &[Vec<(Style, &str)>], theme: &Theme) -> DynamicImage {
        if self.line_number {
            self.line_number_chars = ((v.len() as f32).log10() + 1.0).floor() as u32;
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

        if !self.highlight_lines.is_empty() {
            self.highlight_lines(&mut image, &self.highlight_lines);
        }
        if self.line_number {
            self.draw_line_number(&mut image, drawables.max_lineno, foreground);
        }

        for (x, y, color, style, text) in drawables.drawables {
            let color = color.to_rgba();
            self.font
                .draw_text_mut(&mut image, color, x, y, style, &text);
        }

        image
    }
}
