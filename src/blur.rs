use image::RgbaImage;

pub fn gaussian_blur(image: RgbaImage, sigma: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut raw = image.into_raw();
    let len = raw.len();

    // fastblur::gaussian_blur only accepts Vec<[u8; 4]>
    unsafe {
        raw.set_len(len / 4);

        let ptr = &mut *(&mut raw as *mut Vec<u8> as *mut Vec<[u8; 4]>);
        fastblur::gaussian_blur(ptr, width as usize, height as usize, sigma);

        raw.set_len(len);
    }

    RgbaImage::from_raw(width, height, raw).unwrap()
}
