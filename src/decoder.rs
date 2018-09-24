use image::GrayImage;
use interpolator::Interpolator;
use utils::{gray, traverse_level, GridU8};

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

    pub fn decode(&mut self, (width, height): (u32, u32), grid: &GridU8) -> GrayImage {
        let mut img = GrayImage::new(width, height);
        let levels = grid.len() - 1;

        let mut index = 0;
        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let value = grid[level][index];
                img.put_pixel(column, line, gray(value));
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
                        .interpolate(levels, level + 1, (column, line), &img);

                let pixel = gray(prediction.wrapping_add(diff));
                img.put_pixel(column, line, pixel);
                index += 1;
            };

            traverse_level(level, levels, width, height, process_pixel);
        }

        return img;
    }
}
