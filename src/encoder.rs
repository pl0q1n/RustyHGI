use image::GrayImage;
use utils::{Quantizator, interpolate, traverse_level, GridU8, Metadata, gray};

pub struct EncoderGrayscale {}

pub trait Encoder {
    type Input;
    type Output;

    fn encode(&mut self, metadata: &Metadata, input: Self::Input) -> Self::Output;
}

impl Encoder for EncoderGrayscale {
    type Input = GrayImage;
    type Output = GridU8;

    fn encode(&mut self, metadata: &Metadata, mut input: Self::Input) -> Self::Output {
        let quantizator = Quantizator::new(metadata.quantization_level);
        let (width, height) = input.dimensions();
        let levels = metadata.scale_level;
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
            traverse_level(level, levels, width, height, |column, line| {
                let prediction = interpolate(
                    levels,
                    level + 1,
                    (column, line),
                    &input
                );

                let actual_value = input.get_pixel(column, line).data[0];
                let diff = actual_value.wrapping_sub(prediction);
                let mut quanted_diff = quantizator.quantize(diff);

                let overflow = prediction.checked_add(quanted_diff).is_none();
                let overflow_is_expected = prediction.checked_add(diff).is_none();
                if  overflow != overflow_is_expected {
                    quanted_diff = diff;
                }

                grid[level + 1].push(quanted_diff);
                let pixel = gray(prediction.wrapping_add(quanted_diff));
                input.put_pixel(column, line, pixel);
            });
        }
        return grid;
    }
}
