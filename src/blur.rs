//! Fast (linear time) implementation of the Gaussian Blur algorithm in Rust
//!
//! This file is originally from https://github.com/fschutt/fastblur
//! Edited by aloxaf <aloxafx@gmail.com> to process RgbaImage

use std::cmp::min;

use image::RgbaImage;
use rayon::prelude::*;

#[derive(Copy, Clone)]
struct SharedMutPtr(*mut [[u8; 4]]);

unsafe impl Sync for SharedMutPtr {}

impl SharedMutPtr {
    #[allow(clippy::mut_from_ref)]
    unsafe fn get(&self) -> &mut [[u8; 4]] {
        &mut *self.0
    }
}

pub fn gaussian_blur(image: RgbaImage, sigma: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut raw = image.into_raw();
    let len = raw.len();

    // fastblur::gaussian_blur only accepts Vec<[u8; 4]>
    unsafe {
        raw.set_len(len / 4);

        let ptr = &mut *(&mut raw as *mut Vec<u8> as *mut Vec<[u8; 4]>);
        gaussian_blur_impl(ptr, width as usize, height as usize, sigma);

        raw.set_len(len);
    }

    RgbaImage::from_raw(width, height, raw).unwrap()
}

fn gaussian_blur_impl(data: &mut [[u8; 4]], width: usize, height: usize, blur_radius: f32) {
    let bxs = create_box_gauss(blur_radius, 3);
    let mut backbuf = data.to_vec();

    box_blur(
        &mut backbuf,
        data,
        width,
        height,
        ((bxs[0] - 1) / 2) as usize,
    );
    box_blur(
        &mut backbuf,
        data,
        width,
        height,
        ((bxs[1] - 1) / 2) as usize,
    );
    box_blur(
        &mut backbuf,
        data,
        width,
        height,
        ((bxs[2] - 1) / 2) as usize,
    );
}

#[inline]
fn create_box_gauss(sigma: f32, n: usize) -> Vec<i32> {
    let n_float = n as f32;

    // Ideal averaging filter width
    let w_ideal = (12.0 * sigma * sigma / n_float).sqrt() + 1.0;
    let mut wl: i32 = w_ideal.floor() as i32;

    if wl % 2 == 0 {
        wl -= 1;
    };

    let wu = wl + 2;

    let wl_float = wl as f32;
    let m_ideal = (12.0 * sigma * sigma
        - n_float * wl_float * wl_float
        - 4.0 * n_float * wl_float
        - 3.0 * n_float)
        / (-4.0 * wl_float - 4.0);
    let m: usize = m_ideal.round() as usize;

    let mut sizes = Vec::<i32>::new();

    for i in 0..n {
        if i < m {
            sizes.push(wl);
        } else {
            sizes.push(wu);
        }
    }

    sizes
}

/// Needs 2x the same image
#[inline]
fn box_blur(
    backbuf: &mut [[u8; 4]],
    frontbuf: &mut [[u8; 4]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    box_blur_horz(backbuf, frontbuf, width, height, blur_radius);
    box_blur_vert(frontbuf, backbuf, width, height, blur_radius);
}

#[inline]
fn box_blur_vert(
    backbuf: &[[u8; 4]],
    frontbuf: &mut [[u8; 4]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    let frontbuf = SharedMutPtr(frontbuf as *mut [[u8; 4]]);
    (0..width).into_par_iter().for_each(|i| {
        let col_start = i; //inclusive
        let col_end = i + width * (height - 1); //inclusive
        let mut ti: usize = i;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius * width;

        let fv: [u8; 4] = backbuf[col_start];
        let lv: [u8; 4] = backbuf[col_end];

        let mut val_r: isize = (blur_radius as isize + 1) * isize::from(fv[0]);
        let mut val_g: isize = (blur_radius as isize + 1) * isize::from(fv[1]);
        let mut val_b: isize = (blur_radius as isize + 1) * isize::from(fv[2]);
        let mut val_a: isize = (blur_radius as isize + 1) * isize::from(fv[3]);

        // Get the pixel at the specified index, or the first pixel of the column
        // if the index is beyond the top edge of the image
        let get_top = |i: usize| {
            if i < col_start {
                fv
            } else {
                backbuf[i]
            }
        };

        // Get the pixel at the specified index, or the last pixel of the column
        // if the index is beyond the bottom edge of the image
        let get_bottom = |i: usize| {
            if i > col_end {
                lv
            } else {
                backbuf[i]
            }
        };

        for j in 0..min(blur_radius, height) {
            let bb = backbuf[ti + j * width];
            val_r += isize::from(bb[0]);
            val_g += isize::from(bb[1]);
            val_b += isize::from(bb[2]);
            val_a += isize::from(bb[3]);
        }
        if blur_radius > height {
            val_r += (blur_radius - height) as isize * isize::from(lv[0]);
            val_g += (blur_radius - height) as isize * isize::from(lv[1]);
            val_b += (blur_radius - height) as isize * isize::from(lv[2]);
            val_a += (blur_radius - height) as isize * isize::from(lv[3]);
        }

        for _ in 0..min(height, blur_radius + 1) {
            let bb = get_bottom(ri);
            ri += width;
            val_r += isize::from(bb[0]) - isize::from(fv[0]);
            val_g += isize::from(bb[1]) - isize::from(fv[1]);
            val_b += isize::from(bb[2]) - isize::from(fv[2]);
            val_a += isize::from(bb[3]) - isize::from(fv[3]);

            let frontbuf = unsafe { frontbuf.get() };
            frontbuf[ti] = [
                round(val_r as f32 * iarr) as u8,
                round(val_g as f32 * iarr) as u8,
                round(val_b as f32 * iarr) as u8,
                round(val_a as f32 * iarr) as u8,
            ];
            ti += width;
        }

        if height > blur_radius {
            // otherwise `(height - blur_radius)` will underflow
            for _ in (blur_radius + 1)..(height - blur_radius) {
                let bb1 = backbuf[ri];
                ri += width;
                let bb2 = backbuf[li];
                li += width;

                val_r += isize::from(bb1[0]) - isize::from(bb2[0]);
                val_g += isize::from(bb1[1]) - isize::from(bb2[1]);
                val_b += isize::from(bb1[2]) - isize::from(bb2[2]);
                val_a += isize::from(bb1[3]) - isize::from(bb2[3]);

                let frontbuf = unsafe { frontbuf.get() };
                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                    round(val_a as f32 * iarr) as u8,
                ];
                ti += width;
            }

            for _ in 0..min(height - blur_radius - 1, blur_radius) {
                let bb = get_top(li);
                li += width;

                val_r += isize::from(lv[0]) - isize::from(bb[0]);
                val_g += isize::from(lv[1]) - isize::from(bb[1]);
                val_b += isize::from(lv[2]) - isize::from(bb[2]);
                val_a += isize::from(lv[3]) - isize::from(bb[3]);

                let frontbuf = unsafe { frontbuf.get() };
                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                    round(val_a as f32 * iarr) as u8,
                ];
                ti += width;
            }
        }
    });
}

#[inline]
fn box_blur_horz(
    backbuf: &[[u8; 4]],
    frontbuf: &mut [[u8; 4]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    let frontbuf = SharedMutPtr(frontbuf as *mut [[u8; 4]]);
    (0..height).into_par_iter().for_each(|i| {
        let row_start: usize = i * width; // inclusive
        let row_end: usize = (i + 1) * width - 1; // inclusive
        let mut ti: usize = i * width; // VERTICAL: $i;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius;

        let fv: [u8; 4] = backbuf[row_start];
        let lv: [u8; 4] = backbuf[row_end]; // VERTICAL: $backbuf[ti + $width - 1];

        let mut val_r: isize = (blur_radius as isize + 1) * isize::from(fv[0]);
        let mut val_g: isize = (blur_radius as isize + 1) * isize::from(fv[1]);
        let mut val_b: isize = (blur_radius as isize + 1) * isize::from(fv[2]);
        let mut val_a: isize = (blur_radius as isize + 1) * isize::from(fv[3]);

        // Get the pixel at the specified index, or the first pixel of the row
        // if the index is beyond the left edge of the image
        let get_left = |i: usize| {
            if i < row_start {
                fv
            } else {
                backbuf[i]
            }
        };

        // Get the pixel at the specified index, or the last pixel of the row
        // if the index is beyond the right edge of the image
        let get_right = |i: usize| {
            if i > row_end {
                lv
            } else {
                backbuf[i]
            }
        };

        for j in 0..min(blur_radius, width) {
            let bb = backbuf[ti + j]; // VERTICAL: ti + j * width
            val_r += isize::from(bb[0]);
            val_g += isize::from(bb[1]);
            val_b += isize::from(bb[2]);
            val_a += isize::from(bb[3]);
        }
        if blur_radius > width {
            val_r += (blur_radius - height) as isize * isize::from(lv[0]);
            val_g += (blur_radius - height) as isize * isize::from(lv[1]);
            val_b += (blur_radius - height) as isize * isize::from(lv[2]);
            val_a += (blur_radius - height) as isize * isize::from(lv[3]);
        }

        // Process the left side where we need pixels from beyond the left edge
        for _ in 0..min(width, blur_radius + 1) {
            let bb = get_right(ri);
            ri += 1;
            val_r += isize::from(bb[0]) - isize::from(fv[0]);
            val_g += isize::from(bb[1]) - isize::from(fv[1]);
            val_b += isize::from(bb[2]) - isize::from(fv[2]);
            val_a += isize::from(bb[3]) - isize::from(fv[3]);

            let frontbuf = unsafe { frontbuf.get() };
            frontbuf[ti] = [
                round(val_r as f32 * iarr) as u8,
                round(val_g as f32 * iarr) as u8,
                round(val_b as f32 * iarr) as u8,
                round(val_a as f32 * iarr) as u8,
            ];
            ti += 1; // VERTICAL : ti += width, same with the other areas
        }

        if width > blur_radius {
            // otherwise `(width - blur_radius)` will underflow
            // Process the middle where we know we won't bump into borders
            // without the extra indirection of get_left/get_right. This is faster.
            for _ in (blur_radius + 1)..(width - blur_radius) {
                let bb1 = backbuf[ri];
                ri += 1;
                let bb2 = backbuf[li];
                li += 1;

                val_r += isize::from(bb1[0]) - isize::from(bb2[0]);
                val_g += isize::from(bb1[1]) - isize::from(bb2[1]);
                val_b += isize::from(bb1[2]) - isize::from(bb2[2]);
                val_a += isize::from(bb1[3]) - isize::from(bb2[3]);

                let frontbuf = unsafe { frontbuf.get() };
                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                    round(val_a as f32 * iarr) as u8,
                ];
                ti += 1;
            }

            // Process the right side where we need pixels from beyond the right edge
            for _ in 0..min(width - blur_radius - 1, blur_radius) {
                let bb = get_left(li);
                li += 1;

                val_r += isize::from(lv[0]) - isize::from(bb[0]);
                val_g += isize::from(lv[1]) - isize::from(bb[1]);
                val_b += isize::from(lv[2]) - isize::from(bb[2]);
                val_a += isize::from(lv[3]) - isize::from(bb[3]);

                let frontbuf = unsafe { frontbuf.get() };
                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                    round(val_a as f32 * iarr) as u8,
                ];
                ti += 1;
            }
        }
    });
}

#[inline]
/// Fast rounding for x <= 2^23.
/// This is orders of magnitude faster than built-in rounding intrinsic.
///
/// Source: https://stackoverflow.com/a/42386149/585725
fn round(mut x: f32) -> f32 {
    x += 12_582_912.0;
    x -= 12_582_912.0;
    x
}
