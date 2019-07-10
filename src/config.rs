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

#[derive(StructOpt, Debug)]
#[structopt(name = "silicon", rename_all = "kebab")]
pub struct Config {
    /// The syntax highlight theme. It can be a theme name or path to a .tmTheme file.
    #[structopt(long, default_value = "base16-eighties.dark")]
    theme: String,

    /// The base font.
    #[structopt(long)]
    font: Option<String>,

    /// Size of the base font.
    #[structopt(long, value_name = "size", default_value = "27.0")]
    font_size: f32,

    /// The CJK font.
    #[structopt(long, value_name = "font")]
    cjk_font: Option<String>,

    /// Size of the CJK font.
    #[structopt(long, value_name = "size", default_value = "27.0")]
    cjk_size: f32,

    /// Pad between lines
    #[structopt(long, default_value = "2")]
    line_pad: u32,

    // List all themes.
    // #[structopt(long)]
    // list_themes: bool,

    // Build theme cache.
    // #[structopt(long)]
    // build_cache: bool,
    /// Read input from clipboard.
    #[structopt(long)]
    from_clipboard: bool,

    // Copy the output image to clipboard.
    // #[structopt(short = "c", long)]
    // to_clipboard: bool,
    /// Write output image to specific location instead of cwd.
    #[structopt(short = "o", long, value_name = "path")]
    output: Option<PathBuf>,

    /// Hide the window controls.
    #[structopt(long)]
    no_window_controls: bool,

    /// Hide the line number.
    #[structopt(long)]
    no_line_number: bool,

    /// Background color of the image
    #[structopt(
        long,
        value_name = "color",
        default_value = "#abb8c3",
        parse(try_from_str = "parse_str_color")
    )]
    background: Rgba<u8>,

    /// Color of shadow
    #[structopt(
        long,
        value_name = "color",
        default_value = "#555555",
        parse(try_from_str = "parse_str_color")
    )]
    shadow_color: Rgba<u8>,

    /// Blur radius of the shadow
    #[structopt(long, value_name = "radius", default_value = "10.0")]
    shadow_blur_radius: f32,

    /// Pad horiz
    #[structopt(long, default_value = "80")]
    pad_horiz: u32,

    /// Pad vert
    #[structopt(long, default_value = "100")]
    pad_vert: u32,

    /// Shadow's offset in Y axis
    #[structopt(long, value_name = "offset", default_value = "0")]
    shadow_offset_y: i32,

    /// Shadow's offset in X axis
    #[structopt(long, value_name = "offset", default_value = "0")]
    shadow_offset_x: i32,

    /// The language for syntax highlighting. You can use full name ("Rust") or file extension ("rs").
    #[structopt(short = "l", long)]
    language: Option<String>,

    // Draw a custom text on the bottom right corner
    // #[structopt(long)]
    // watermark: Option<String>,
    /// File to read. If not set, stdin will be use.
    #[structopt(value_name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
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
                    .ok_or(format_err!("There is no such a language: {}", language))?
            } else {
                ps.find_syntax_by_first_line(&code)
                    .ok_or(format_err!("Failed to detect the language"))?
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
                    .ok_or(format_err!("There is no such a language: {}", language))?
            } else {
                ps.find_syntax_for_file(path)?
                    .ok_or(format_err!("Failed to detect the language"))?
            };

            return Ok((language, s));
        }

        let mut stdin = stdin();
        let mut s = String::new();
        stdin.read_to_string(&mut s)?;

        let language = if let Some(language) = &self.language {
            ps.find_syntax_by_token(language)
                .ok_or(format_err!("There is no such a language: {}", language))?
        } else {
            ps.find_syntax_by_first_line(&s)
                .ok_or(format_err!("Failed to detect the language"))?
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
        let mut formatter = ImageFormatterBuilder::new()
            .line_pad(self.line_pad)
            .font_size(self.font_size)
            .cjk_font_size(self.cjk_size);

        if let Some(name) = &self.font {
            formatter = formatter.font(name, self.font_size)?;
        }
        if let Some(name) = &self.cjk_font {
            formatter = formatter.cjk_font(name, self.cjk_size)?;
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

    pub fn output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }
}
