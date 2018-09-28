use image::GrayImage;
use interpolator::Interpolator;
use quantizator::Quantizator;
use utils::{gray, traverse_level, GridU8};

pub struct Encoder<I, Q> {
    interpolator: I,
    quantizator: Q,
    scale_level: usize,
}

impl<I, Q> Encoder<I, Q>
where
    Q: Quantizator,
    I: Interpolator,
{
    pub fn new(interpolator: I, quantizator: Q, scale_level: usize) -> Self {
        Encoder {
            quantizator,
            interpolator,
            scale_level,
        }
    }

    pub fn encode(&mut self, mut input: GrayImage) -> GridU8 {
        let (width, height) = input.dimensions();
        let levels = self.scale_level;
        let mut grid = GridU8::new();
        grid.resize(levels + 1, Vec::new());

        let level = 0;
        let step: usize = 1 << levels;

        // preallocate grid
        let mut previous_level_pixels = ((width + step as u32 - 1) / step as u32) * ((height + step as u32 - 1) / step as u32);
        grid[level].reserve(previous_level_pixels as usize);
        unsafe { grid[level].set_len(previous_level_pixels as usize) };

        for level in 0..levels {
            let step = 1 << (levels - level - 1);
            let n = (width / step) * (height / step);
            let npixels = n - previous_level_pixels;
            grid[level + 1].reserve(npixels as usize);
            unsafe { grid[level + 1].set_len(npixels as usize) };
            previous_level_pixels = n;
        }

        // initialize first level with pixel values
        let mut index = 0;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let pixel = input.get_pixel(column, line).data[0];
                grid[level][index] = pixel;
                index += 1;
            }
        }

        for level in 0..levels {
            let mut index = 0;
            let current_level = &mut grid[level + 1];

            let process_pixel = #[inline(always)]
            |column, line| {
                let prediction =
                    self.interpolator
                        .interpolate(levels, level + 1, (column, line), &input);

                let actual_value = input.get_pixel(column, line).data[0];
                let diff = actual_value.wrapping_sub(prediction);
                let mut quanted_diff = self.quantizator.quantize(diff);

                let overflow = prediction.checked_add(quanted_diff).is_none();
                let overflow_is_expected = prediction.checked_add(diff).is_none();
                if overflow != overflow_is_expected {
                    quanted_diff = diff;
                }

                current_level[index] = quanted_diff;
                index += 1;
                let pixel = gray(prediction.wrapping_add(quanted_diff));
                input.put_pixel(column, line, pixel);
            };

            traverse_level(level, levels, width, height, process_pixel);
        }

        grid
    }
}
