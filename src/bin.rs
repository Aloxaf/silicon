#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

use crate::config::Config;
use crate::utils::*;
use anyhow::Error;
use image::DynamicImage;
use structopt::StructOpt;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;
#[cfg(target_os = "windows")]
use {
    clipboard_win::{formats, Clipboard, Setter},
    image::ImageOutputFormat,
};
#[cfg(target_os = "macos")]
use {image::ImageOutputFormat, pasteboard::Pasteboard};
#[cfg(target_os = "linux")]
use {image::ImageOutputFormat, std::process::Command};

pub mod blur;
pub mod config;
pub mod error;
pub mod font;
pub mod formatter;
pub mod utils;

#[cfg(target_os = "linux")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    Command::new("xclip")
        .args(&[
            "-sel",
            "clip",
            "-t",
            "image/png",
            temp.path().to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    unsafe {
        Pasteboard::Image.copy(temp.path().to_str().unwrap());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    let mut temp: Vec<u8> = Vec::new();

    // Convert the image to RGB without alpha because the clipboard
    // of windows doesn't support it.
    let image = DynamicImage::ImageRgb8(image.to_rgb());

    image.write_to(&mut temp, ImageOutputFormat::Bmp)?;

    let _clip =
        Clipboard::new_attempts(10).map_err(|e| format_err!("Couldn't open clipboard: {}", e))?;

    formats::Bitmap
        .write_clipboard(&temp)
        .map_err(|e| format_err!("Failed copy image: {}", e))?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn dump_image_to_clipboard(_image: &DynamicImage) -> Result<(), Error> {
    Err(format_err!(
        "This feature hasn't been implemented for your system"
    ))
}

fn run() -> Result<(), Error> {
    let config: Config = Config::from_args();

    let (ps, ts) = init_syntect();

    if config.list_themes {
        for i in ts.themes.keys() {
            println!("{}", i);
        }
        return Ok(());
    } else if config.list_fonts {
        let source = font_kit::source::SystemSource::new();
        for font in source.all_families().unwrap_or_default() {
            println!("{}", font);
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
        let path = config.get_expanded_output().unwrap();
        image
            .save(&path)
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
