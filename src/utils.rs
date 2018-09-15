use image::{ImageBuffer, Luma};

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

use std::cmp;

#[inline]
pub fn get_interp_pixels(
    levels: usize,
    level: usize,
    (width, height): (u32, u32),
    (x, y): (u32, u32),
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    default_val: u8,
) -> CrossedValues {
    let mut values = CrossedValues::default();
    // step size on previous level
    let step = 1 << (levels - level + 1);
    let x_mod = x % step as u32;
    let y_mod = y % step as u32;

    let x_top = x - x_mod;
    let x_bot = cmp::min(x + (step - x_mod), height - 1);
    let y_left = y - y_mod;
    let y_right = cmp::min(y + (step - y_mod), width - 1);

    let is_on_prev_lvl = |x| is_on_prev_lvl(levels, level, x);

    if is_on_prev_lvl(x_top)
        && is_on_prev_lvl(x_bot)
        && is_on_prev_lvl(y_left)
        && is_on_prev_lvl(y_right)
    {
        let get_pixel = |x, y| image.get_pixel(x, y).data[0];

        values.left_top  = get_pixel(x_top, y_left);
        values.right_top = get_pixel(x_top, y_right);
        values.left_bot  = get_pixel(x_bot, y_left);
        values.right_bot = get_pixel(x_bot, y_right);
    } else {
        let get_pix_val = |x, y| {
            if is_on_prev_lvl(x) && is_on_prev_lvl(y) {
                image.get_pixel(x, y).data[0]
            } else {
                default_val
            }
        };
        values.left_top  = get_pix_val(x_top, y_left);
        values.right_top = get_pix_val(x_top, y_right);
        values.left_bot  = get_pix_val(x_bot, y_left);
        values.right_bot = get_pix_val(x_bot, y_right);
    }
    return values;
}
