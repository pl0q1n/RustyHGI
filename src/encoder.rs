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
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let pixel = input.get_pixel(column, line).data[0];
                grid[level].push(pixel);
            }
        }

        for level in 0..levels {
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

                grid[level + 1].push(quanted_diff);
                let pixel = gray(prediction.wrapping_add(quanted_diff));
                input.put_pixel(column, line, pixel);
            };

            traverse_level(level, levels, width, height, process_pixel);
        }
        return grid;
    }
}
