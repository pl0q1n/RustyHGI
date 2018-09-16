use image::{ImageBuffer, Luma};

pub type GridU8 = Vec<Vec<u8>>;

#[inline(always)]
pub fn gray(value: u8) -> Luma<u8> {
    Luma { data: [value] }
}

arg_enum! {
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum QuantizationLevel {
    Loseless,
    Low,
    Medium,
    High,
}
}

pub struct Quantizator {
    table: [u8; 256]
}

impl Quantizator {
    pub fn new(level: QuantizationLevel) -> Self {
        let error = match level {
            QuantizationLevel::Loseless => 0,
            QuantizationLevel::Low => 10,
            QuantizationLevel::Medium => 20,
            QuantizationLevel::High => 30,
        };

        let scale = 2 * error + 1;
        let quantize = |x| {
            let r = (x as usize + error) / scale;
            let v = r * scale;
            v as u8
        };

        let mut table = [0; 256];
        for x in 0..table.len() {
            table[x] = quantize(x as u8);
        }
        Quantizator { table }        
    }
    
    #[inline]
    pub fn quantize(&self, value: u8) -> u8 {
        self.table[value as usize]
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Interpolator {
    Crossed,
    Line,
    Previous,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub quantization_level: QuantizationLevel,
    pub interpolator: Interpolator,
    pub width: u32,
    pub height: u32,
    pub scale_level: usize,
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
        let average = |x, y| (x as usize + y as usize + 1) >> 1; // div 2

        let left  = average(self.left_top,  self.left_bot);
        let right = average(self.right_bot, self.right_top);
        let top   = average(self.right_top, self.left_top);
        let bot   = average(self.right_bot, self.left_bot);

        let average = (left + right + top + bot + 1) >> 2; // div 4

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
        let mut column = start;
        while column < width {
            f(column as u32, line as u32);
            column += step;
        }

        line += substep;
        if line >= height {
            break;
        }

        let mut column = 0;
        while column < width {
            f(column as u32, line as u32);
            column += substep;
        }
        line += substep;
    }
}

#[inline]
pub fn interpolate(
    levels: usize,
    level: usize,
    (x, y): (u32, u32), // column, line
    image: &ImageBuffer<Luma<u8>, Vec<u8>>
) -> u8 {
    // step size on previous level
    let step = 1 << (levels - level + 1);
    let mask = step - 1;

    let x_top   = x - (x & mask); 
    let y_left  = y - (y & mask);
    let x_bot   = x_top + step;
    let y_right = y_left + step;

    let get_pixel = |x, y| {
        if x < image.width() && y < image.height() {
            image.get_pixel(x, y).data[0]
        } else {
            0
        }
    };

    CrossedValues {
        left_top:  get_pixel(x_top, y_left),
        right_top: get_pixel(x_top, y_right),
        left_bot:  get_pixel(x_bot, y_left),
        right_bot: get_pixel(x_bot, y_right)
    }.prediction()
}
