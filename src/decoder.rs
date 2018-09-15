use image::{GrayImage, ImageBuffer};
use utils::{average, get_interp_pixels, gray, traverse_level, GridU8, Metadata};

pub struct DecoderGrayscale {}

pub trait Decoder {
    type Input;
    type Output;

    fn decode(&mut self, metadata: &Metadata, input: &Self::Input) -> Self::Output;
}

impl Decoder for DecoderGrayscale {
    type Input = GridU8;
    type Output = GrayImage;

    fn decode(&mut self, metadata: &Metadata, input: &Self::Input) -> Self::Output {
        let (width, height) = (metadata.width, metadata.height);
        let mut img = ImageBuffer::new(width, height);
        let levels = input.len() - 1;

        let mut grid_ind = 0;

        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let value = input[level][grid_ind];
                img.put_pixel(column, line, gray(value));
                grid_ind += 1;
            }
        }

        for level in 0..levels {
            let mut grid_ind = 0;

            traverse_level(level, levels, width, height, |column, line| {
                let post_inter_value = {
                    let value = input[level + 1][grid_ind];

                    let prediction = get_interp_pixels(
                        levels,
                        level + 1,
                        (width, height),
                        (column, line),
                        &img,
                        value,
                    ).prediction();

                    average(value, prediction) as u8
                };

                let pixel = gray(post_inter_value);
                img.put_pixel(column, line, pixel);
                grid_ind += 1;
            });
        }

        return img;
    }
}
