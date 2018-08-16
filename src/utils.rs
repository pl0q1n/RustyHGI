use bit_vec::BitVec;
use std::collections::HashMap;

pub type CoordHolder = (usize, usize);
pub type PredictMap = HashMap<CoordHolder, u8>;

pub enum Quantizator {
    LoselessCompression,
    LowCompression,
    MediumCompression,
    HighCompression,
}

pub enum Interpolator {
    Crossed,
    Line,
    Previous,
}

pub struct PositionMap {
    positions: BitVec,
    width: u32,
    height: u32,
}

impl PositionMap {
    pub fn new(width: u32, height: u32) -> Self {
        PositionMap {
            positions: BitVec::from_elem((width * height) as usize, false),
            width,
            height,
        }
    }

    pub fn get_val(&self, x: u32, y: u32) -> bool {
        return self.positions.get((x * self.width + y) as usize).unwrap();
    }
    pub fn set_val(&mut self, x: u32, y: u32) {
        self.positions.set((x * self.width + y) as usize, true);
    }
}

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
