use image::{GrayImage, ImageBuffer};
use std::iter::repeat;
use utils::{average, get_interp_pixels, gray, GridU8, Metadata, PositionMap, PredictMap};

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
        let (width, height) = metadata.dimensions;
        let mut grid_ind = 0;
        let mut img = ImageBuffer::new(width, height);
        let mut grid_depth = 0usize;
        let depth = input.len() - 1;
        let mut ind = 2usize.pow(depth as u32);

        let mut positions = PositionMap::new(width, height);
        let mut predictions = Vec::<PredictMap>::new();

        predictions.resize(depth + 1 as usize, PredictMap::new());

        for line in (0..height).step_by(ind) {
            for column in (0..width).step_by(ind) {
                let value = input[grid_depth][grid_ind];
                img.put_pixel(column, line, gray(value));
                predictions[grid_depth].insert((column as usize, line as usize), value);
                grid_ind += 1;
                positions.set_val(column, line);
            }
        }

        ind /= 2;
        grid_depth += 1;

        while ind >= 1 {
            grid_ind = 0;
            let iter = (0..height)
                .step_by(ind)
                .flat_map(move |line| repeat(line).zip((0..width).step_by(ind)));

            for (line, column) in iter {
                if !positions.get_val(column, line) {
                    let post_inter_value = {
                        let mut curr_level = &predictions[grid_depth - 1];
                        let value = input[grid_depth][grid_ind];

                        let prediction = get_interp_pixels(
                            depth,
                            grid_depth,
                            (width, height),
                            (column, line),
                            curr_level,
                            value,
                        ).prediction();

                        average(value, prediction) as u8
                    };

                    let pixel = gray(post_inter_value);
                    img.put_pixel(column, line, pixel);
                    predictions[grid_depth].insert((column as usize, line as usize), post_inter_value);
                    grid_ind += 1;

                    positions.set_val(column, line);
                }
            }
            ind /= 2;
            grid_depth += 1;
            println!("{}", grid_depth);
        }
        return img;
    }
}
