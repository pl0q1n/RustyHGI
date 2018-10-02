use image::Luma;

#[inline(always)]
pub fn gray(value: u8) -> Luma<u8> {
    Luma { data: [value] }
}

// x is column
// y is line

#[inline]
pub fn traverse_level<F>(level: usize, levels: usize, x1: u32, x2: u32, y1: u32, y2: u32, mut f: F)
where
    F: FnMut(u32, u32),
{
    let e = levels - level;
    let step = 1 << e;
    let substep = 1 << (e - 1);
    let start = x1 + substep;

    let mut line = y1;
    while line < y2 {
        let mut column = start;
        while column < x2 {
            f(column as u32, line as u32);
            column += step;
        }

        line += substep;
        if line >= y2 {
            break;
        }

        let mut column = x1;
        while column < x2 {
            f(column as u32, line as u32);
            column += substep;
        }
        line += substep;
    }
}