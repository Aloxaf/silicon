use failure::Error;
use image::DynamicImage;

#[cfg(target_os = "windows")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    use std::ffi::c_void;
    use clipboard_win::{Clipboard, formats, image::Bitmap};
    use image::ImageOutputFormat;

    let mut buffer = vec![];
    image.write_to(&mut buffer, ImageOutputFormat::BMP)?;

    let bmp = Bitmap::new(buffer.as_mut_ptr() as *mut c_void)?;

    Clipboard::new()
        .unwrap()
        .set(formats::CF_BITMAP, bmp.as_bytes())
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e))?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> Result<(), Error> {
    use std::process::Command;
    use image::ImageOutputFormat;

    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::PNG)?;

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

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn dump_image_to_clipboard(_image: &DynamicImage) -> Result<(), Error> {
    Err(format_err!(
        "This feature hasn't been implemented for your system"
    ))
}
