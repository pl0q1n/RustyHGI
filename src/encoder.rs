use bit_vec::BitVec;
use image::{GrayImage, Luma};
use utils::{PredictMap, is_on_prev_lvl, Metadata, PositionMap};

pub type GridU8 = Vec<Vec<u8>>;

fn get_interp_pixels(
    total_depth: usize,
    current_depth: usize,
    (width, height): (u32, u32),
    (x, y): (u32, u32),
    curr_level: &PredictMap,
) -> (u8, u8, u8, u8) {
    let mut left_top_val = 0;
    let mut right_top_val = 0;
    let mut left_bot_val = 0;
    let mut right_bot_val = 0;
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
        left_top_val = *curr_level
            .get(&(x_top_cord as usize, y_left_cord as usize))
            .unwrap();
        right_top_val = *curr_level
            .get(&(x_top_cord as usize, y_right_cord as usize))
            .unwrap();
        left_bot_val = *curr_level
            .get(&(x_bot_cord as usize, y_left_cord as usize))
            .unwrap();
        right_bot_val = *curr_level
            .get(&(x_bot_cord as usize, y_right_cord as usize))
            .unwrap();
    } else {
        if is_on_prev_lvl(total_depth, current_depth, x_top_cord)
            && is_on_prev_lvl(total_depth, current_depth, y_left_cord)
        {
            left_top_val = *curr_level
                .get(&(x_top_cord as usize, y_left_cord as usize))
                .unwrap();
        } else {
            left_top_val = 255;
        }

        if is_on_prev_lvl(total_depth, current_depth, x_top_cord)
            && is_on_prev_lvl(total_depth, current_depth, y_right_cord)
        {
            right_top_val = *curr_level
                .get(&(x_top_cord as usize, y_right_cord as usize))
                .unwrap();
        } else {
            right_top_val = 255;
        }

        if is_on_prev_lvl(total_depth, current_depth, x_bot_cord)
            && is_on_prev_lvl(total_depth, current_depth, y_left_cord)
        {
            left_bot_val = *curr_level
                .get(&(x_bot_cord as usize, y_left_cord as usize))
                .unwrap();
        } else {
            left_bot_val = 255;
        }

        if is_on_prev_lvl(total_depth, current_depth, x_bot_cord)
            && is_on_prev_lvl(total_depth, current_depth, y_right_cord)
        {
            right_bot_val = *curr_level
                .get(&(x_bot_cord as usize, y_right_cord as usize))
                .unwrap();
        } else {
            right_bot_val = 255;
        }
    }
    return (left_top_val, right_top_val, left_bot_val, right_bot_val);
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
        let mut grid = Vec::<Vec<u8>>::with_capacity(metadata.scale_level + 1);
        grid.resize(metadata.scale_level + 1, Vec::new());
        let mut grid_depth = 0usize;
        let mut ind = 2usize.pow(grid.len() as u32 - 1);

        let mut positions = PositionMap::new(width, height);

        let mut predictions = Vec::<PredictMap>::new();

        let depth = grid.len() - 1;
        predictions.resize(depth + 1 as usize, PredictMap::new());

        while ind >= 1 {
            let mut k = 0;
            for x in (0..width).step_by(ind) {
                for y in (0..height).step_by(ind) {
                    if !positions.get_val(x, y) {
                        if grid_depth == 0 {
                            grid[grid_depth].push(input.get_pixel(x, y).data[0]);
                            predictions[grid_depth]
                                .insert((x as usize, y as usize), input.get_pixel(x, y).data[0]);
                        } else {
                            let (post_inter_value, predicted_value) = {
                                let mut curr_level = &predictions[grid_depth - 1];

                                let (left_top_val, right_top_val, left_bot_val, right_bot_val) =
                                    get_interp_pixels(
                                        depth,
                                        grid_depth,
                                        (width, height),
                                        (x, y),
                                        curr_level,
                                    );
                                let find_average_safety = |lhs: u8, rhs: u8| {
                                    ((lhs as usize + rhs as usize + 1) / 2) as u8
                                };

                                let left_inter = find_average_safety(left_top_val, left_bot_val);
                                let right_inter = find_average_safety(right_bot_val, right_top_val);
                                let top_inter = find_average_safety(right_top_val, left_top_val);
                                let bot_inter = find_average_safety(right_top_val, left_bot_val);

                                let predicted_value = ((left_inter as u16
                                    + right_inter as u16
                                    + top_inter as u16
                                    + bot_inter as u16
                                    + 1) / 4)
                                    as u8;
                                let post_inter_value =
                                    input.get_pixel(x, y).data[0].saturating_sub(predicted_value);
                                (post_inter_value, predicted_value)
                            };
                            // Quantization with precision = 15;
                            let quanted_postinter_value = ((2 * 15 + 1)
                                * ((post_inter_value as f64 + 15.0) / (2.0 * 15.0 + 1.0)).floor()
                                    as usize
                                % 256)
                                as u8;

                            input.put_pixel(
                                x,
                                y,
                                Luma {
                                    data: [quanted_postinter_value.saturating_add(predicted_value)],
                                },
                            );
                            grid[grid_depth].push(quanted_postinter_value);
                            predictions[grid_depth]
                                .insert((x as usize, y as usize), post_inter_value);
                        }
                        k += 1;
                        positions.set_val(x, y);
                    }
                }
            }
            ind /= 2;
            grid_depth += 1;
            println!("{}", grid_depth);
        }
        return grid;
    }
}
