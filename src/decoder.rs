use grid::Grid;
use image::{GenericImage, GrayImage};
use interpolator::Interpolator;
use utils::{gray, traverse_level};

pub struct Decoder<I> {
    interpolator: I,
}

impl<I> Decoder<I>
where
    I: Interpolator,
{
    pub fn new(interpolator: I) -> Self {
        Decoder { interpolator }
    }

    pub fn decode(&mut self, (width, height): (u32, u32), levels: usize, grid: &Grid) -> GrayImage {
        let mut image = GrayImage::new(width, height);

        // initialize first level
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let value = unsafe { grid.get(column, line) };
                unsafe { image.unsafe_put_pixel(column, line, gray(value)) };
            }
        }

        for level in 0..levels {
            let process_pixel = #[inline(always)]
            |column, line| {
                let diff = unsafe { grid.get(column, line) };

                let prediction =
                    self.interpolator
                        .interpolate(levels, level + 1, (column, line), &image);

                let pixel = gray(prediction.wrapping_add(diff));
                unsafe { image.unsafe_put_pixel(column, line, pixel) };
            };

            traverse_level(level, levels, 0, width, 0, height, process_pixel);
        }
        image
    }
}
