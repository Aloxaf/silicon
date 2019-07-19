//! Format the output of syntect into an image
use crate::error::FontError;
use crate::font::{FontCollection, FontStyle};
use crate::utils::*;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use syntect::highlighting::{Color, Style, Theme};

pub struct ImageFormatter {
    /// pad between lines
    /// Default: 2
    line_pad: u32,
    /// pad between code and edge of code area.
    /// Default: 25
    code_pad: u32,
    /// pad of top of the code area
    /// Default: 50
    code_pad_top: u32,
    /// show line number
    /// Default: true
    line_number: bool,
    /// round corner
    /// Default: true
    round_corner: bool,
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
    /// Shadow adder
    shadow_adder: Option<ShadowAdder>,
}

#[derive(Default)]
pub struct ImageFormatterBuilder<S> {
    /// Pad between lines
    line_pad: u32,
    /// Show line number
    line_number: bool,
    /// Font of english character, should be mono space font
    font: Vec<(S, f32)>,
    /// Highlight lines
    highlight_lines: Vec<u32>,
    /// Whether show the window controls
    window_controls: bool,
    /// Whether round the corner of the image
    round_corner: bool,
    /// Shadow adder,
    shadow_adder: Option<ShadowAdder>,
}

// FXIME: cannot use `ImageFormatterBuilder::new().build()` bacuse cannot infer type for `S`
impl<S: AsRef<str> + Default> ImageFormatterBuilder<S> {
    pub fn new() -> Self {
        Self {
            line_pad: 2,
            line_number: true,
            window_controls: true,
            round_corner: true,
            ..Default::default()
        }
    }

    /// Whether show the line number
    pub fn line_number(mut self, show: bool) -> Self {
        self.line_number = show;
        self
    }

    /// Set the pad between lines
    pub fn line_pad(mut self, pad: u32) -> Self {
        self.line_pad = pad;
        self
    }

    /// Set the font
    pub fn font(mut self, fonts: Vec<(S, f32)>) -> Self {
        self.font = fonts;
        self
    }

    /// Whether show the windows controls
    pub fn window_controls(mut self, show: bool) -> Self {
        self.window_controls = show;
        self
    }

    /// Whether round the corner
    pub fn round_corner(mut self, b: bool) -> Self {
        self.round_corner = b;
        self
    }

    /// Add the shadow
    pub fn shadow_adder(mut self, adder: ShadowAdder) -> Self {
        self.shadow_adder = Some(adder);
        self
    }

    /// Set the lines to highlight.
    pub fn highlight_lines(mut self, lines: Vec<u32>) -> Self {
        self.highlight_lines = lines;
        self
    }

    pub fn build(self) -> Result<ImageFormatter, FontError> {
        let font = if self.font.is_empty() {
            FontCollection::default()
        } else {
            FontCollection::new(&self.font)?
        };

        let code_pad_top = if self.window_controls { 50 } else { 0 };

        Ok(ImageFormatter {
            line_pad: self.line_pad,
            code_pad: 25,
            line_number: self.line_number,
            line_number_pad: 6,
            line_number_chars: 0,
            highlight_lines: self.highlight_lines,
            round_corner: self.round_corner,
            shadow_adder: self.shadow_adder,
            code_pad_top,
            font,
        })
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

    fn draw_line_number(&self, image: &mut DynamicImage, lineno: u32, mut color: Rgba<u8>) {
        for i in color.data.iter_mut() {
            *i = (*i).saturating_sub(20);
        }
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

        for i in color.data.iter_mut() {
            *i = (*i).saturating_add(40);
        }

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

        // draw_window_controls == true
        if self.code_pad_top != 0 {
            add_window_controls(&mut image);
        }

        if self.round_corner {
            round_corner(&mut image, 12);
        }

        if let Some(adder) = &self.shadow_adder {
            adder.apply_to(&image)
        } else {
            image
        }
    }
}
