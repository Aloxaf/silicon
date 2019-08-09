#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

use crate::clipboard::dump_image_to_clipboard;
use crate::config::Config;
use crate::utils::*;
use failure::Error;
use structopt::StructOpt;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

pub mod blur;
pub mod config;
pub mod error;
pub mod font;
pub mod formatter;
pub mod utils;

mod clipboard;

fn run() -> Result<(), Error> {
    let config: Config = Config::from_args();

    let (ps, ts) = init_syntect();

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

    let image = formatter.format(&highlight, &theme);

    if config.to_clipboard {
        dump_image_to_clipboard(&image)?;
    } else {
        let path = &config.output.unwrap();
        image
            .save(path)
            .map_err(|e| format_err!("Failed to save image to {}: {}", path.display(), e))?;
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("[error] {}", e);
    }
}
