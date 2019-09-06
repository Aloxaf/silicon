use crate::formatter::{ImageFormatter, ImageFormatterBuilder};
use crate::utils::{ShadowAdder, ToRgba};
use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Error;
use image::Rgba;
use std::fs::File;
use std::io::{stdin, Read};
use std::num::ParseIntError;
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
        let tmp = font.split('=').collect::<Vec<_>>();
        let font_name = tmp[0].to_owned();
        let font_size = tmp
            .get(1)
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(26.0);
        result.push((font_name, font_size));
    }
    result
}

fn parse_line_range(s: &str) -> Result<Vec<u32>, ParseIntError> {
    let mut result = vec![];
    for range in s.split(';') {
        let range: Vec<u32> = range
            .split('-')
            .map(|s| s.parse::<u32>())
            .collect::<Result<Vec<_>, _>>()?;
        if range.len() == 1 {
            result.push(range[0])
        } else {
            for i in range[0]..=range[1] {
                result.push(i);
            }
        }
    }
    Ok(result)
}

// https://github.com/TeXitoi/structopt/blob/master/CHANGELOG.md#support-optional-vectors-of-arguments-for-distinguishing-between--o-1-2--o-and-no-option-provided-at-all-by-sphynx-180
type FontList = Vec<(String, f32)>;
type Lines = Vec<u32>;

#[derive(StructOpt, Debug)]
#[structopt(name = "silicon")]
pub struct Config {
    /// Background color of the image
    #[structopt(
        long,
        short,
        value_name = "COLOR",
        default_value = "#aaaaff",
        parse(try_from_str = parse_str_color)
    )]
    pub background: Rgba<u8>,

    /// Read input from clipboard.
    #[structopt(long)]
    pub from_clipboard: bool,

    /// File to read. If not set, stdin will be use.
    #[structopt(value_name = "FILE", parse(from_os_str))]
    pub file: Option<PathBuf>,

    /// The font list. eg. 'Hack; SimSun=31'
    #[structopt(long, short, value_name = "FONT", parse(from_str = parse_font_str))]
    pub font: Option<FontList>,

    /// Lines to high light. rg. '1-3; 4'
    #[structopt(long, value_name = "LINES", parse(try_from_str = parse_line_range))]
    pub highlight_lines: Option<Lines>,

    /// The language for syntax highlighting. You can use full name ("Rust") or file extension ("rs").
    #[structopt(short, value_name = "LANG", long)]
    pub language: Option<String>,

    /// Pad between lines
    #[structopt(long, value_name = "PAD", default_value = "2")]
    pub line_pad: u32,

    /// List all themes.
    #[structopt(long)]
    pub list_themes: bool,

    /// Write output image to specific location instead of cwd.
    #[structopt(
        short,
        long,
        value_name = "PATH",
        required_unless_one = &["list-themes", "to-clipboard"]
    )]
    pub output: Option<PathBuf>,

    /// Hide the window controls.
    #[structopt(long)]
    pub no_window_controls: bool,

    /// Hide the line number.
    #[structopt(long)]
    pub no_line_number: bool,

    /// Don't round the corner
    #[structopt(long)]
    pub no_round_corner: bool,

    /// Pad horiz
    #[structopt(long, value_name = "PAD", default_value = "80")]
    pub pad_horiz: u32,

    /// Pad vert
    #[structopt(long, value_name = "PAD", default_value = "100")]
    pub pad_vert: u32,

    /// Color of shadow
    #[structopt(
        long,
        value_name = "COLOR",
        default_value = "#555555",
        parse(try_from_str = parse_str_color)
    )]
    pub shadow_color: Rgba<u8>,

    /// Blur radius of the shadow. (set it to 0 to hide shadow)
    #[structopt(long, value_name = "R", default_value = "0")]
    pub shadow_blur_radius: f32,

    /// Shadow's offset in Y axis
    #[structopt(long, value_name = "Y", default_value = "0")]
    pub shadow_offset_y: i32,

    /// Shadow's offset in X axis
    #[structopt(long, value_name = "X", default_value = "0")]
    pub shadow_offset_x: i32,

    /// Tab width
    #[structopt(long, value_name = "WIDTH", default_value = "4")]
    pub tab_width: u8,

    /// The syntax highlight theme. It can be a theme name or path to a .tmTheme file.
    #[structopt(long, value_name = "THEME", default_value = "Dracula")]
    pub theme: String,

    // Copy the output image to clipboard.
    #[structopt(short = "c", long)]
    pub to_clipboard: bool,
    // Draw a custom text on the bottom right corner
    // #[structopt(long)]
    // watermark: Option<String>,
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
        };

        Ok((language, s))
    }

    pub fn theme(&self, ts: &ThemeSet) -> Result<Theme, Error> {
        if let Some(theme) = ts.themes.get(&self.theme) {
            Ok(theme.clone())
        } else {
            Ok(ThemeSet::get_theme(&self.theme)?)
        }
    }

    pub fn get_formatter(&self) -> Result<ImageFormatter, Error> {
        let formatter = ImageFormatterBuilder::new()
            .line_pad(self.line_pad)
            .window_controls(!self.no_window_controls)
            .line_number(!self.no_line_number)
            .font(self.font.clone().unwrap_or_else(|| vec![]))
            .round_corner(!self.no_round_corner)
            .window_controls(!self.no_window_controls)
            .shadow_adder(self.get_shadow_adder())
            .tab_width(self.tab_width)
            .highlight_lines(self.highlight_lines.clone().unwrap_or_else(|| vec![]));

        Ok(formatter.build()?)
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

    pub fn get_expanded_output(&self) -> Option<PathBuf> {
        let need_expand = self.output.as_ref().map(|p| p.starts_with("~")) == Some(true);

        if let (Ok(home_dir), true) = (std::env::var("HOME"), need_expand) {
            self.output
                .as_ref()
                .map(|p| p.to_string_lossy().replacen("~", &home_dir, 1).into())
        } else {
            self.output.clone()
        }
    }
}
