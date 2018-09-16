use image::{GrayImage, ImageBuffer};
use utils::{interpolate, gray, traverse_level, GridU8, Metadata};

pub struct DecoderGrayscale {}

pub trait Decoder {
    type Input;
    type Output;

    fn decode(&mut self, metadata: &Metadata, input: &Self::Input) -> Self::Output;
}

impl Decoder for DecoderGrayscale {
    type Input = GridU8;
    type Output = GrayImage;

    fn decode(&mut self, metadata: &Metadata, grid: &Self::Input) -> Self::Output {
        let (width, height) = (metadata.width, metadata.height);
        let mut img = ImageBuffer::new(width, height);
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

            let process_pixel = #[inline(always)] |column, line| {
                let diff = grid[level + 1][index];
                let prediction = interpolate(
                    levels,
                    level + 1,
                    (column, line),
                    &img,
                );

                let pixel = gray(prediction.wrapping_add(diff));
                img.put_pixel(column, line, pixel);
                index += 1;
            };

            traverse_level(level, levels, width, height, process_pixel);
        }

        return img;
    }
}
