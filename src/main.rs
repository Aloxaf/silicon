#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

use crate::config::Config;
use crate::utils::{add_window_controls, dump_image_to_clipboard, round_corner};
use failure::Error;
use image::ImageFormat;
use std::io::stdout;
use structopt::StructOpt;
use syntect::dumps;
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

    let ps = dumps::from_binary::<SyntaxSet>(include_bytes!("../assets/syntaxes.bin")); //SyntaxSet::load_defaults_newlines();
    let ts = dumps::from_binary::<ThemeSet>(include_bytes!("../assets/themes.bin")); // ThemeSet::load();

    if config.list_themes {
        for i in ts.themes.keys() {
            println!("{}", i);
        }
        return Ok(());
    }

    let (syntax, code) = config.get_source_code(&ps)?;

    let theme = config.theme(&ts)?;

    let mut h = HighlightLines::new(syntax, &theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight(line, &ps))
        .collect::<Vec<_>>();

    let mut formatter = config.get_formatter()?;

    let mut image = formatter.format(&highlight, &theme);

    if !config.no_window_controls {
        add_window_controls(&mut image);
    }
    if !config.no_round_corner {
        round_corner(&mut image, 12);
    }

    let image = config.get_shadow_adder().apply_to(&image);

    if config.to_clipboard {
        dump_image_to_clipboard(&image)?;
    } else if let Some(path) = &config.output {
        image
            .save(path)
            .map_err(|e| format_err!("Failed to save image to {}: {}", path.display(), e))?;
    } else {
        let mut stdout = stdout();
        image.write_to(&mut stdout, ImageFormat::PNG)?;
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("[error] {}", e);
    }
}
