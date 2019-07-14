use crate::formatter::{ImageFormatter, ImageFormatterBuilder};
use crate::utils::{ShadowAdder, ToRgba};
use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Error;
use image::Rgba;
use std::fs::File;
use std::io::{stdin, Read};
use std::path::PathBuf;
use structopt::StructOpt;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

fn parse_str_color(s: &str) -> Result<Rgba<u8>, Error> {
    Ok(s.to_rgba()
        .map_err(|_| format_err!("Invalid color: `{}`", s))?)
}

fn parse_font_str(s: &str) -> Vec<(String, f32)> {
    let mut result = vec![];
    for font in s.split(';') {
        let tmp = font.split('=').collect::<Vec<&str>>();
        let font_name = tmp[0].to_owned();
        let font_size = tmp
            .get(1)
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(26.0);
        result.push((font_name, font_size));
    }
    result
}

#[derive(StructOpt, Debug)]
#[structopt(name = "silicon", rename_all = "kebab")]
pub struct Config {
    /// The syntax highlight theme. It can be a theme name or path to a .tmTheme file.
    #[structopt(long, default_value = "Dracula")]
    pub theme: String,

    /// The font list. eg. 'Hack; SimSun=31'
    #[structopt(long, parse(from_str = "parse_font_str"))]
    pub font: Option<Vec<(String, f32)>>,

    /// Pad between lines
    #[structopt(long, default_value = "2")]
    pub line_pad: u32,

    /// List all themes.
    #[structopt(long)]
    pub list_themes: bool,

    /// Read input from clipboard.
    #[structopt(long)]
    pub from_clipboard: bool,

    // Copy the output image to clipboard.
    #[structopt(short = "c", long)]
    pub to_clipboard: bool,

    /// Write output image to specific location instead of cwd.
    #[structopt(short = "o", long, value_name = "path")]
    pub output: Option<PathBuf>,

    /// Hide the window controls.
    #[structopt(long)]
    pub no_window_controls: bool,

    /// Hide the line number.
    #[structopt(long)]
    pub no_line_number: bool,

    /// Background color of the image
    #[structopt(
        long,
        value_name = "color",
        default_value = "#abb8c3",
        parse(try_from_str = "parse_str_color")
    )]
    pub background: Rgba<u8>,

    /// Color of shadow
    #[structopt(
        long,
        value_name = "color",
        default_value = "#555555",
        parse(try_from_str = "parse_str_color")
    )]
    pub shadow_color: Rgba<u8>,

    /// Blur radius of the shadow
    #[structopt(long, value_name = "radius", default_value = "70.0")]
    pub shadow_blur_radius: f32,

    /// Pad horiz
    #[structopt(long, default_value = "80")]
    pub pad_horiz: u32,

    /// Pad vert
    #[structopt(long, default_value = "100")]
    pub pad_vert: u32,

    /// Shadow's offset in Y axis
    #[structopt(long, value_name = "offset", default_value = "0")]
    pub shadow_offset_y: i32,

    /// Shadow's offset in X axis
    #[structopt(long, value_name = "offset", default_value = "0")]
    pub shadow_offset_x: i32,

    /// The language for syntax highlighting. You can use full name ("Rust") or file extension ("rs").
    #[structopt(short = "l", long)]
    pub language: Option<String>,

    // Draw a custom text on the bottom right corner
    // #[structopt(long)]
    // watermark: Option<String>,
    /// File to read. If not set, stdin will be use.
    #[structopt(value_name = "FILE", parse(from_os_str))]
    pub file: Option<PathBuf>,
}

impl Config {
    pub fn get_source_code<'a>(
        &self,
        ps: &'a SyntaxSet,
    ) -> Result<(&'a SyntaxReference, String), Error> {
        if self.from_clipboard {
            let mut ctx = ClipboardContext::new()
                .map_err(|e| format_err!("failed to access clipboard: {}", e))?;
            let code = ctx
                .get_contents()
                .map_err(|e| format_err!("failed to access clipboard: {}", e))?;

            let language = if let Some(language) = &self.language {
                ps.find_syntax_by_token(language)
                    .ok_or_else(|| format_err!("Unsupported language: {}", language))?
            } else {
                ps.find_syntax_by_first_line(&code)
                    .ok_or_else(|| format_err!("Failed to detect the language"))?
                // TODO: else ?
            };
            return Ok((language, code));
        }

        if let Some(path) = &self.file {
            let mut s = String::new();
            let mut file = File::open(path)?;
            file.read_to_string(&mut s)?;

            let language = if let Some(language) = &self.language {
                ps.find_syntax_by_token(language)
                    .ok_or_else(|| format_err!("Unsupported language: {}", language))?
            } else {
                ps.find_syntax_for_file(path)?
                    .ok_or_else(|| format_err!("Failed to detect the language"))?
            };

            return Ok((language, s));
        }

        let mut stdin = stdin();
        let mut s = String::new();
        stdin.read_to_string(&mut s)?;

        let language = if let Some(language) = &self.language {
            ps.find_syntax_by_token(language)
                .ok_or_else(|| format_err!("Unsupported language: {}", language))?
        } else {
            ps.find_syntax_by_first_line(&s)
                .ok_or_else(|| format_err!("Failed to detect the language"))?
            // TODO: else ?
        };

        Ok((language, s))
    }

    pub fn theme(&self, ts: &ThemeSet) -> Result<Theme, Error> {
        if let Some(theme) = ts.themes.get(&self.theme) {
            return Ok(theme.clone());
        } else {
            return Ok(ThemeSet::get_theme(&self.theme)?);
        }
        // &ts.themes[&self.theme]
    }

    pub fn no_window_controls(&self) -> bool {
        self.no_window_controls
    }

    pub fn get_formatter(&self) -> Result<ImageFormatter, Error> {
        let mut formatter = ImageFormatterBuilder::new().line_pad(self.line_pad);
        if let Some(fonts) = &self.font {
            formatter = formatter.font(fonts)?;
        }
        if self.no_line_number {
            formatter = formatter.line_number(false);
        }
        if self.no_window_controls {
            formatter = formatter.code_pad_top(0);
        }

        Ok(formatter.build())
    }

    pub fn get_shadow_adder(&self) -> ShadowAdder {
        ShadowAdder::new()
            .background(self.background)
            .shadow_color(self.shadow_color)
            .blur_radius(self.shadow_blur_radius)
            .pad_horiz(self.pad_horiz)
            .pad_vert(self.pad_vert)
            .offset_x(self.shadow_offset_x)
            .offset_y(self.shadow_offset_y)
    }
}
