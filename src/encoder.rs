use image::{GrayImage, Luma};
use std::iter::repeat;
use utils::{
    get_interp_pixels, get_predicted_val, is_on_prev_lvl, CrossedValues, GridU8, Metadata,
    PositionMap, PredictMap,
};

pub struct EncoderGrayscale {}

pub trait Encoder {
    type Input;
    type Output;

    fn encode(&mut self, metadata: Metadata, input: Self::Input) -> Self::Output;
}

impl Encoder for EncoderGrayscale {
    type Input = GrayImage;
    type Output = GridU8;

    fn encode(&mut self, metadata: Metadata, mut input: Self::Input) -> Self::Output {
        let (width, height) = metadata.dimension;
        let mut grid = GridU8::with_capacity(metadata.scale_level + 1);
        grid.resize(metadata.scale_level + 1, Vec::new());
        let mut grid_depth = 0usize;
        let mut ind = 2usize.pow(grid.len() as u32 - 1);

        let mut positions = PositionMap::new(width, height);

        let mut predictions = Vec::<PredictMap>::new();

        let depth = grid.len() - 1;
        predictions.resize(depth + 1 as usize, PredictMap::new());

        for x in (0..width).step_by(ind) {
            for y in (0..height).step_by(ind) {
                grid[grid_depth].push(input.get_pixel(x, y).data[0]);
                predictions[grid_depth]
                    .insert((x as usize, y as usize), input.get_pixel(x, y).data[0]);
            }
        }

        ind /= 2;
        grid_depth += 1;

        while ind >= 1 {
            let iter = (0..width)
                .step_by(ind)
                .into_iter()
                .flat_map(move |x| (0..height).step_by(ind).zip(repeat(x)));

            for (y, x) in iter {
                if !positions.get_val(x, y) {
                    let (post_inter_value, predicted_value) = {
                        let mut curr_level = &predictions[grid_depth - 1];

                        let values = get_interp_pixels(
                            depth,
                            grid_depth,
                            (width, height),
                            (x, y),
                            curr_level,
                            255,
                        );
                        let predicted_value = get_predicted_val(values);
                        let post_inter_value =
                            input.get_pixel(x, y).data[0].saturating_sub(predicted_value);
                        (post_inter_value, predicted_value)
                    };
                    // Quantization with precision = 15;
                    let quanted_postinter_value = ((2 * 15 + 1)
                        * ((post_inter_value as f64 + 15.0) / (2.0 * 15.0 + 1.0)).floor() as usize
                        % 256) as u8;

                    input.put_pixel(
                        x,
                        y,
                        Luma {
                            data: [quanted_postinter_value.saturating_add(predicted_value)],
                        },
                    );
                    grid[grid_depth].push(quanted_postinter_value);
                    predictions[grid_depth].insert((x as usize, y as usize), post_inter_value);
                }
                positions.set_val(x, y);
            }
            ind /= 2;
            grid_depth += 1;
            println!("{}", grid_depth);
        }
        return grid;
    }
}
