#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Grid {
    buffer: Vec<u8>,
    width: usize
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut buffer = Vec::with_capacity(size);
        unsafe { buffer.set_len(size) };

        Grid {
            buffer,
            width
        }
    }

    #[inline(always)]
    pub unsafe fn set(&mut self, (column, line): (u32, u32), value: u8) {
        *self.buffer.get_unchecked_mut(line as usize * self.width + column as usize) = value;
    }

    #[inline(always)]
    pub unsafe fn get(&self, column: u32, line: u32) -> u8 {
        *self.buffer.get_unchecked(line as usize * self.width + column as usize)
    }

    pub fn print(&self) {
        for value in self.buffer.iter() {
            print!("{} ", value);
        }
    }
}