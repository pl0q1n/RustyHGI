use bit_vec::BitVec;
use std::collections::HashMap;

pub type CoordHolder = (usize, usize);
pub type PredictMap = HashMap<CoordHolder, u8>;
pub type GridU8 = Vec<Vec<u8>>;

#[derive(Clone, Serialize, Deserialize)]
pub enum Quantizator {
    LoselessCompression,
    LowCompression,
    MediumCompression,
    HighCompression,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Interpolator {
    Crossed,
    Line,
    Previous,
}

pub struct PositionMap {
    positions: BitVec,
    width: u32,
}

impl PositionMap {
    pub fn new(width: u32, height: u32) -> Self {
        PositionMap {
            positions: BitVec::from_elem((width * height) as usize, false),
            width,
        }
    }

    pub fn get_val(&self, x: u32, y: u32) -> bool {
        return self.positions.get((y * self.width + x) as usize).unwrap();
    }
    pub fn set_val(&mut self, x: u32, y: u32) {
        self.positions.set((y * self.width + x) as usize, true);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub quantizator: Quantizator,
    pub interpolator: Interpolator,
    pub dimension: (u32, u32),
    pub scale_level: usize,
}

// Check that coord is processed on previous level
pub fn is_on_prev_lvl(total_depth: usize, current_depth: usize, coord: u32) -> bool {
    let ind = 2usize.pow((total_depth - current_depth) as u32);
    let curr_lvl_check = coord % (ind as u32 * 2) == 0;
    if !curr_lvl_check {
        return curr_lvl_check;
    }
    for lvl in 0..current_depth - 1 {
        let curr_step = 2u32.pow((total_depth - lvl) as u32);
        if coord % curr_step == 0 {
            return false;
        }
    }
    return true;
}

#[derive(Default)]
pub struct CrossedValues {
    pub left_top: u8,
    pub right_top: u8,
    pub left_bot: u8,
    pub right_bot: u8,
}

pub fn get_interp_pixels(
    total_depth: usize,
    current_depth: usize,
    (width, height): (u32, u32),
    (x, y): (u32, u32),
    curr_level: &PredictMap,
    default_val: u8,
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
                *curr_level.get(&(x as usize, y as usize)).unwrap()
            } else {
                default_val
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

pub fn get_predicted_val(values: CrossedValues) -> u8 {
    let left_inter = get_average(values.left_top, values.left_bot);
    let right_inter = get_average(values.right_bot, values.right_top);
    let top_inter = get_average(values.right_top, values.left_top);
    let bot_inter = get_average(values.right_top, values.left_bot);

    ((left_inter as u16 + right_inter as u16 + top_inter as u16 + bot_inter as u16 + 1) / 4) as u8
}
