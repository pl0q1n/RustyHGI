use image::Luma;

pub type GridU8 = Vec<Vec<u8>>;

#[inline(always)]
pub fn gray(value: u8) -> Luma<u8> {
    Luma { data: [value] }
}

arg_enum! {
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum QuantizationLevel {
    Lossless,
    Low,
    Medium,
    High,
}
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InterpolationType {
    Crossed,
    Line,
    Previous,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub quantization_level: QuantizationLevel,
    pub interpolation: InterpolationType,
    pub width: u32,
    pub height: u32,
    pub scale_level: usize,
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