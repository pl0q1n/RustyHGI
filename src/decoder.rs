use image::{GrayImage, GenericImage};
use interpolator::Interpolator;
use utils::{gray, traverse_level};
use grid::Grid;

pub struct Decoder<I> {
    interpolator: I,
}

impl<I> Decoder<I>
where
    I: Interpolator,
{
    pub fn new(interpolator: I) -> Self {
        Decoder {
            interpolator
        }
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

        let chunk_size = 256;
        for height_start in (0..height).step_by(chunk_size) {
            for width_start in (0..width).step_by(chunk_size) {
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

                    use std::cmp::min;
                    let width_end = min(width_start + chunk_size as u32, width);
                    let height_end = min(height_start + chunk_size as u32, height);
                    traverse_level(level, levels, width_start, width_end, height_start, height_end, process_pixel);                
                }
            }
        }
        image
    }
}
