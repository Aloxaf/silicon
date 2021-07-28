#[macro_use]
extern crate anyhow;

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

mod config;
use crate::config::{config_file, get_args_from_config_file};
use config::Config;
use silicon::utils::{get_lines_of_code, init_syntect};

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
    let mut args = get_args_from_config_file();
    let mut args_cli = std::env::args_os();
    args.insert(0, args_cli.next().unwrap());
    args.extend(args_cli);
    let config: Config = Config::from_iter(args);
    println!("{:?}", config);

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
    } else if config.config_file {
        println!("{}", config_file().to_string_lossy());
        return Ok(());
    }

    let (syntax, mut code) = config.get_source_code(&ps)?;
    if config.end != 0 {
        code = get_lines_of_code(config.start, config.end, &code)
        .map_err(|_e| format_err!("The value of 'start' cannot be larger than that of 'end'"))?;
    }
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
