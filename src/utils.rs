use image::Luma;

pub type Level = Vec<u8>;
pub type GridU8 = Vec<Level>;

#[inline(always)]
pub fn gray(value: u8) -> Luma<u8> {
    Luma { data: [value] }
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

    for line in (0..height).step_by(step) {
        for column in (start..width).step_by(step) {
            f(column, line);
        }

        let line = line + substep;
        if line >= height {
            break;
        }

        for column in  (0..width).step_by(substep as usize) {
            f(column, line);
        }
    }
}