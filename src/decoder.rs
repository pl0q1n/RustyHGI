use image::{GrayImage, ImageBuffer};
use utils::{average, get_interp_pixels, gray, GridU8, Metadata, PredictMap};

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

        let mut predictions = Vec::new();
        predictions.resize(levels + 1 as usize, PredictMap::new());

        let mut grid_ind = 0;

        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let value = input[level][grid_ind];
                img.put_pixel(column, line, gray(value));
                predictions[level].insert((column as usize, line as usize), value);
                grid_ind += 1;
            }
        }

        for level in 0..levels {
            let e = levels - level;
            let start = 1 << (e - 1);
            let step = 1 << e;
            let substep = start;

            let mut grid_ind = 0;
            let mut column = 0;
            while column < width {
                for line in (start..height).step_by(step) {
                    let post_inter_value = {
                        let mut curr_level = &predictions[level];
                        let value = input[level + 1][grid_ind];

                        let prediction = get_interp_pixels(
                            levels,
                            level + 1,
                            (width, height),
                            (column, line),
                            curr_level,
                            value,
                        ).prediction();

                        average(value, prediction) as u8
                    };

                    let pixel = gray(post_inter_value);
                    img.put_pixel(column, line, pixel);
                    predictions[level + 1].insert((column as usize, line as usize), post_inter_value);
                    grid_ind += 1;
                }

                column += substep;
                if column >= width {
                    break;
                }

                for line in (0..height).step_by(substep as usize) {
                    let post_inter_value = {
                        let mut curr_level = &predictions[level];
                        let value = input[level + 1][grid_ind];

                        let prediction = get_interp_pixels(
                            levels,
                            level + 1,
                            (width, height),
                            (column, line),
                            curr_level,
                            value,
                        ).prediction();

                        average(value, prediction) as u8
                    };

                    let pixel = gray(post_inter_value);
                    img.put_pixel(column, line, pixel);
                    predictions[level + 1].insert((column as usize, line as usize), post_inter_value);
                    grid_ind += 1;
                }
                column += substep;
            }
        }

        return img;
    }
}
