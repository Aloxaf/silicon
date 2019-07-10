#[macro_use]
extern crate failure;

use crate::config::Config;
use crate::utils::{add_window_controls, round_corner};
use failure::Error;
use structopt::StructOpt;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

mod blur;
mod config;
mod font;
mod formatter;
mod utils;

fn run() -> Result<(), Error> {
    let config: Config = Config::from_args();

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let (syntax, code) = config.get_source_code(&ps)?;

    let theme = config.theme(&ts)?;

    let mut h = HighlightLines::new(syntax, &theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight(line, &ps))
        .collect::<Vec<_>>();

    let mut formatter = config.get_formatter()?;

    let mut image = formatter.format(&highlight, &theme);

    if !config.no_window_controls() {
        add_window_controls(&mut image);
    }

    round_corner(&mut image, 12);

    let image = config.get_shadow_adder().apply_to(&image);

    let path = config.output();
    image
        .save(path)
        .map_err(|e| format_err!("Failed to save image to {}: {}", path.display(), e))?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[error] {}", e);
    }
}
