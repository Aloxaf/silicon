//! Manual implementations of some things from imageproc - without all the unnecessary faff
//!
//! The MIT License (MIT)
//!
//! Copyright (c) 2015 PistonDevelopers
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use conv::ValueInto;
use image::{GenericImage, Pixel, Rgba, RgbaImage};

/// Draw as much of a circle, including its contents, as lies inside the image bounds.
pub(crate) fn draw_filled_circle_mut<I>(
    image: &mut I,
    center: (i32, i32),
    radius: i32,
    color: I::Pixel,
) where
    I: GenericImage,
    I::Pixel: 'static,
{
    let mut x = 0i32;
    let mut y = radius;
    let mut p = 1 - radius;
    let x0 = center.0;
    let y0 = center.1;

    while x <= y {
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

/// Draws a line segment on an image in place.
///
/// Draws as much of the line segment between start and end as lies inside the image bounds.
///
/// Uses [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm).
pub(crate) fn draw_line_segment_mut<I>(
    canvas: &mut I,
    start: (f32, f32),
    end: (f32, f32),
    color: I::Pixel,
) where
    I: GenericImage,
    I::Pixel: 'static,
{
    let (width, height) = canvas.dimensions();
    let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

    let line_iterator = BresenhamLineIter::new(start, end);

    for point in line_iterator {
        let x = point.0;
        let y = point.1;

        if in_bounds(x, y) {
            canvas.put_pixel(x as u32, y as u32, color);
        }
    }
}

/// Iterates over the coordinates in a line segment using
/// [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm).
struct BresenhamLineIter {
    dx: f32,
    dy: f32,
    x: i32,
    y: i32,
    error: f32,
    end_x: i32,
    is_steep: bool,
    y_step: i32,
}

impl BresenhamLineIter {
    /// Creates a [`BresenhamLineIter`](struct.BresenhamLineIter.html) which will iterate over the integer coordinates
    /// between `start` and `end`.
    fn new(start: (f32, f32), end: (f32, f32)) -> BresenhamLineIter {
        let (mut x0, mut y0) = (start.0, start.1);
        let (mut x1, mut y1) = (end.0, end.1);

        let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
        if is_steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;

        BresenhamLineIter {
            dx,
            dy: (y1 - y0).abs(),
            x: x0 as i32,
            y: y0 as i32,
            error: dx / 2f32,
            end_x: x1 as i32,
            is_steep,
            y_step: if y0 < y1 { 1 } else { -1 },
        }
    }
}

impl Iterator for BresenhamLineIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<(i32, i32)> {
        if self.x > self.end_x {
            None
        } else {
            let ret = if self.is_steep {
                (self.y, self.x)
            } else {
                (self.x, self.y)
            };

            self.x += 1;
            self.error -= self.dy;
            if self.error < 0f32 {
                self.y += self.y_step;
                self.error += self.dx;
            }

            Some(ret)
        }
    }
}

/// Draws a rectangle and its contents on an image in place.
///
/// Draws as much of the rectangle and its contents as lies inside the image bounds.
pub(crate) fn draw_filled_rect_mut(canvas: &mut RgbaImage, rect: Rect, color: Rgba<u8>) {
    let canvas_bounds = Rect {
        left: 0,
        top: 0,
        width: canvas.width(),
        height: canvas.height(),
    };
    if let Some(intersection) = canvas_bounds.intersect(rect) {
        for dy in 0..intersection.height {
            for dx in 0..intersection.width {
                let x = intersection.left as u32 + dx;
                let y = intersection.top as u32 + dy;
                canvas.put_pixel(x, y, color);
            }
        }
    }
}

/// A rectangular region of non-zero width and height.
/// # Examples
/// ```
/// use imageproc::rect::Rect;
/// use imageproc::rect::Region;
///
/// // Construct a rectangle with top-left corner at (4, 5), width 6 and height 7.
/// let rect = Rect::at(4, 5).of_size(6, 7);
///
/// // Contains top-left point:
/// assert_eq!(rect.left(), 4);
/// assert_eq!(rect.top(), 5);
/// assert!(rect.contains(rect.left(), rect.top()));
///
/// // Contains bottom-right point, at (left + width - 1, top + height - 1):
/// assert_eq!(rect.right(), 9);
/// assert_eq!(rect.bottom(), 11);
/// assert!(rect.contains(rect.right(), rect.bottom()));
/// ```
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Rect {
    /// Greatest y-coordinate reached by rect.
    fn bottom(&self) -> i32 {
        self.top + (self.height as i32) - 1
    }

    /// Greatest x-coordinate reached by rect.
    fn right(&self) -> i32 {
        self.left + (self.width as i32) - 1
    }

    /// Returns the intersection of self and other, or none if they are are disjoint.
    fn intersect(&self, other: Rect) -> Option<Rect> {
        let left = std::cmp::max(self.left, other.left);
        let top = std::cmp::max(self.top, other.top);
        let right = std::cmp::min(self.right(), other.right());
        let bottom = std::cmp::min(self.bottom(), other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(Rect {
            left,
            top,
            width: (right - left) as u32 + 1,
            height: (bottom - top) as u32 + 1,
        })
    }
}

/// Adds pixels with the given weights. Results are clamped to prevent arithmetical overflows.
///
/// # Examples
/// ```
/// # extern crate image;
/// # extern crate imageproc;
/// # fn main() {
/// use image::Rgb;
/// use imageproc::pixelops::weighted_sum;
///
/// let left = Rgb([10u8, 20u8, 30u8]);
/// let right = Rgb([100u8, 80u8, 60u8]);
///
/// let sum = weighted_sum(left, right, 0.7, 0.3);
/// assert_eq!(sum, Rgb([37, 38, 39]));
/// # }
/// ```
pub(crate) fn weighted_sum<P: Pixel>(left: P, right: P, left_weight: f32, right_weight: f32) -> P
where
    P::Subpixel: ValueInto<f32> + Clamp,
{
    left.map2(&right, |p, q| {
        Clamp::clamp(cast(p) * left_weight + cast(q) * right_weight)
    })
}

fn cast<T, U>(x: T) -> U
where
    T: ValueInto<U>,
{
    match x.value_into() {
        Ok(y) => y,
        Err(_) => panic!("Failed to convert"),
    }
}

/// A type to which we can clamp a value of type f32.
/// Implementations are not required to handle `NaN`s gracefully.
pub trait Clamp {
    /// Clamp `x` to a valid value for this type.
    fn clamp(x: f32) -> Self;
}

/// Creates an implementation of Clamp for type To.
macro_rules! implement_clamp {
    ($to:ty) => {
        impl Clamp for $to {
            fn clamp(x: f32) -> $to {
                if x < <$to>::MAX as f32 {
                    if x > <$to>::MIN as f32 {
                        x as $to
                    } else {
                        <$to>::MIN
                    }
                } else {
                    <$to>::MAX
                }
            }
        }
    };
}

impl Clamp for f32 {
    fn clamp(x: f32) -> f32 {
        x
    }
}

implement_clamp!(u8);
implement_clamp!(u16);
