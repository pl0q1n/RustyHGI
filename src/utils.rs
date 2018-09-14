use image::Luma;
use std::collections::HashMap;

pub type CoordHolder = (usize, usize);
pub type PredictMap = HashMap<CoordHolder, u8>;
pub type GridU8 = Vec<Vec<u8>>;

#[inline(always)]
pub fn gray(value: u8) -> Luma<u8> {
    Luma { data: [value] }
}

arg_enum! {
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Quantizator {
    Loseless,
    Low,
    Medium,
    High,
}
}

impl Quantizator {
    #[inline]
    pub fn quantize(&self, value: u8) -> u8 {
        let max_error = match self {
            Quantizator::Loseless => return value,
            Quantizator::Low => 10.0,
            Quantizator::Medium => 20.0,
            Quantizator::High => 30.0,
        };

        ((2 * max_error as usize + 1)
            * ((value as f64 + max_error) / (2.0 * max_error + 1.0)).floor() as usize
            % 256) as u8
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Interpolator {
    Crossed,
    Line,
    Previous,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub quantizator: Quantizator,
    pub interpolator: Interpolator,
    pub width: u32,
    pub height: u32,
    pub scale_level: usize,
}

#[inline(always)]
pub fn is_on_prev_lvl(total_levels: usize, level: usize, x: u32) -> bool {
    if x == 0 {
        return level == 1;
    }

    let previous = level - 1;
    x.trailing_zeros() == (total_levels as u32 - previous as u32)
}

#[inline(always)]
pub fn average(x: u8, y: u8) -> usize {
    (x as usize + y as usize + 1) / 2
}

#[derive(Default)]
pub struct CrossedValues {
    pub left_top: u8,
    pub right_top: u8,
    pub left_bot: u8,
    pub right_bot: u8,
}

impl CrossedValues {
    #[inline]
    pub fn prediction(&self) -> u8 {
        let left = average(self.left_top, self.left_bot);
        let right = average(self.right_bot, self.right_top);
        let top = average(self.right_top, self.left_top);
        let bot = average(self.right_bot, self.left_bot);

        let average = (left + right + top + bot + 1) / 4;

        average as u8
    }
}

#[inline]
pub fn traverse_level<F>(level: usize, levels: usize, width: u32, height: u32, mut f: F)
where
    F: FnMut(u32, u32),
{
    let e = levels - level;
    let start = 1 << (e - 1);
    let step = 1 << e;
    let substep = start;

    let mut line = 0;
    while line < height {
        for column in (start..width).step_by(step) {
            f(column as u32, line as u32);
        }

        line += substep;
        if line >= height {
            break;
        }

        for column in (0..width).step_by(substep as usize) {
            f(column as u32, line as u32);
        }
        line += substep;
    }
}

#[inline]
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
