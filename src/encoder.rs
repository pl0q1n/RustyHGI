use image::GrayImage;
use std::cmp;
use std::iter::repeat;
use utils::{get_interp_pixels, gray, GridU8, Metadata, PositionMap, PredictMap};

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
        let (width, height) = metadata.dimensions;
        let mut grid = GridU8::with_capacity(metadata.scale_level + 1);
        grid.resize(metadata.scale_level + 1, Vec::new());
        let mut grid_depth = 0usize;
        let mut ind = 2usize.pow(grid.len() as u32 - 1);

        let mut positions = PositionMap::new(width, height);

        let mut predictions = Vec::<PredictMap>::new();

        let depth = grid.len() - 1;
        predictions.resize(depth + 1 as usize, PredictMap::new());

        for line in (0..height).step_by(ind) {
            for column in (0..width).step_by(ind) {
                let pix_val = input.get_pixel(column, line).data[0];

                grid[grid_depth].push(pix_val);
                predictions[grid_depth]
                    .insert((column as usize, line as usize), pix_val);
                positions.set_val(column, line);
            }
        }

        ind /= 2;
        grid_depth += 1;

        while ind >= 1 {
            let iter = (0..height)
                .step_by(ind)
                .flat_map(move |y| repeat(y).zip((0..width).step_by(ind)));

            for (line, column) in iter {
                if !positions.get_val(column, line) {
                    let (post_inter_value, predicted_value) = {
                        let mut curr_level = &predictions[grid_depth - 1];

                        let prediction = get_interp_pixels(
                            depth,
                            grid_depth,
                            (width, height),
                            (column, line),
                            curr_level,
                            255,
                        ).prediction();

                        let pix_value = input.get_pixel(column, line).data[0];
                        let post_inter_value =
                            255 - (cmp::max(pix_value, prediction)
                                - cmp::min(pix_value, prediction));
                        //input.get_pixel(x, y).data[0].wrapping_sub(predicted_value);
                        (post_inter_value, prediction)
                    };
                    let quanted_postinter_value = metadata.quantizator.quantize(post_inter_value);

                    let pixel = gray(quanted_postinter_value.saturating_add(predicted_value));
                    input.put_pixel(column, line, pixel);
                    grid[grid_depth].push(quanted_postinter_value);
                    predictions[grid_depth].insert((column as usize, line as usize), post_inter_value);
                
                    positions.set_val(column, line);
                }
            }
            ind /= 2;
            grid_depth += 1;
            println!("{}", grid_depth);
        }
        return grid;
    }
}
