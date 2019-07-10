use image::RgbaImage;
use std::os::raw::c_uchar;

#[link(name = "gauss")]
#[link(name = "m")]
extern "C" {
    fn GaussianBlurFilter(
        input: *const c_uchar,
        output: *mut c_uchar,
        width: i32,
        height: i32,
        stride: i32,
        sigma: f32,
    );
}

pub fn gaussian_blur(image: RgbaImage, sigma: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let stride = 4 * width;
    let raw = image.as_flat_samples();
    //let raw = image.into_raw();
    let mut out = raw.samples.to_owned();

    unsafe {
        GaussianBlurFilter(
            raw.samples.as_ptr(),
            out.as_mut_ptr(),
            width as i32,
            height as i32,
            stride as i32,
            sigma,
        );
    }

    RgbaImage::from_raw(width, height, out).unwrap()
}
