use image::{GrayImage, Luma};
use std::iter::repeat;
use utils::{is_on_prev_lvl, Metadata, PositionMap, PredictMap};

pub type GridU8 = Vec<Vec<u8>>;

#[derive(Default)]
struct CrossedValues {
    left_top: u8,
    right_top: u8,
    left_bot: u8,
    right_bot: u8,
}

fn get_interp_pixels(
    total_depth: usize,
    current_depth: usize,
    (width, height): (u32, u32),
    (x, y): (u32, u32),
    curr_level: &PredictMap,
) -> CrossedValues {
    let mut values = CrossedValues::default();
    let ind = 2usize.pow((total_depth - current_depth) as u32);
    let x_mod = x % (ind * 2) as u32;
    let y_mod = y % (ind * 2) as u32;

    let x_top_cord = x - x_mod;
    let mut x_bot_cord = x + (ind as u32 * 2 - x_mod);
    let y_left_cord = y - y_mod;
    let mut y_right_cord = y + (ind as u32 * 2 - y_mod);

    let bot_out_of_range = x_bot_cord >= width;
    let right_out_of_range = y_right_cord >= height;

    if bot_out_of_range {
        x_bot_cord = width - 1;
    }
    if right_out_of_range {
        y_right_cord = height - 1;
    }
    if !bot_out_of_range
        && !right_out_of_range
        && is_on_prev_lvl(total_depth, current_depth, x_top_cord)
        && is_on_prev_lvl(total_depth, current_depth, x_bot_cord)
        && is_on_prev_lvl(total_depth, current_depth, y_left_cord)
        && is_on_prev_lvl(total_depth, current_depth, y_right_cord)
    {
        values.left_top = *curr_level
            .get(&(x_top_cord as usize, y_left_cord as usize))
            .unwrap();
        values.right_top = *curr_level
            .get(&(x_top_cord as usize, y_right_cord as usize))
            .unwrap();
        values.left_bot = *curr_level
            .get(&(x_bot_cord as usize, y_left_cord as usize))
            .unwrap();
        values.right_bot = *curr_level
            .get(&(x_bot_cord as usize, y_right_cord as usize))
            .unwrap();
    } else {
        let get_pix_val = |x, y| {
            if is_on_prev_lvl(total_depth, current_depth, x)
                && is_on_prev_lvl(total_depth, current_depth, y)
            {
                *curr_level
                    .get(&(x as usize, y as usize))
                    .unwrap()
            } else {
                255
            }
        };
        values.left_top = get_pix_val(x_top_cord, y_left_cord);
        values.right_top = get_pix_val(x_top_cord, y_right_cord);
        values.left_bot = get_pix_val(x_bot_cord, y_left_cord);
        values.right_bot = get_pix_val(x_bot_cord, y_right_cord);
    }
    return values;
}

fn get_average(lhs: u8, rhs: u8) -> u8 {
    ((lhs as usize + rhs as usize + 1) / 2) as u8
}

fn get_predicted_val(values: CrossedValues) -> u8 {
    let left_inter = get_average(values.left_top, values.left_bot);
    let right_inter = get_average(values.right_bot, values.right_top);
    let top_inter = get_average(values.right_top, values.left_top);
    let bot_inter = get_average(values.right_top, values.left_bot);

    ((left_inter as u16 + right_inter as u16 + top_inter as u16 + bot_inter as u16 + 1) / 4) as u8
}

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
