use image::GrayImage;
use interpolator::Interpolator;
use utils::{gray, traverse_level, Level};

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

    pub fn decode(&mut self, (width, height): (u32, u32), grid: &[Level]) -> GrayImage {
        let mut image = GrayImage::new(width, height);
        let levels = grid.len() - 1;

        let mut index = 0;
        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let value = grid[level][index];
                image.put_pixel(column, line, gray(value));
                index += 1;
            }
        }

        for level in 0..levels {
            let mut index = 0;

            let process_pixel = #[inline(always)]
            |column, line| {
                let diff = grid[level + 1][index];
                let prediction =
                    self.interpolator
                        .interpolate(levels, level + 1, (column, line), &image);

                let pixel = gray(prediction.wrapping_add(diff));
                image.put_pixel(column, line, pixel);
                index += 1;
            };

            traverse_level(level, levels, width, height, process_pixel);
        }

        image
    }
}
