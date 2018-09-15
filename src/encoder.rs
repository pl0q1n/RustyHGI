use image::GrayImage;
use utils::{get_interp_pixels, traverse_level, GridU8, Metadata, gray};

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
        let (width, height) = (metadata.width, metadata.height);
        let mut grid = GridU8::with_capacity(metadata.scale_level + 1);
        grid.resize(metadata.scale_level + 1, Vec::new());

        let levels = metadata.scale_level;

        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let pix_val = input.get_pixel(column, line).data[0];

                grid[level].push(pix_val);
            }
        }

        for level in 0..levels {
            traverse_level(level, levels, width, height, |column, line| {
                let prediction = get_interp_pixels(
                    levels,
                    level + 1,
                    (column, line),
                    &input,
                    0,
                ).prediction();

                let actual_value = input.get_pixel(column, line).data[0];
                let diff = actual_value.wrapping_sub(prediction);
                let quanted_diff = metadata.quantizator.quantize(diff);

                grid[level + 1].push(quanted_diff);
                let pixel = gray(prediction.wrapping_add(quanted_diff));
                input.put_pixel(column, line, pixel);
            });
        }
        return grid;
    }
}
