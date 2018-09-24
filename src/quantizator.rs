arg_enum! {
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum QuantizationLevel {
    Lossless,
    Low,
    Medium,
    High,
}
}


pub trait Quantizator : From<QuantizationLevel> {
    fn quantize(&self, value: u8) -> u8;
}

pub struct Linear {
    table: [u8; 256]
}

impl From<QuantizationLevel> for Linear {
    fn from(level: QuantizationLevel) -> Self {
        let error = match level {
            QuantizationLevel::Lossless => 0,
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
        Linear { table }
    }
}

impl Quantizator for Linear {
    #[inline(always)]
    fn quantize(&self, value: u8) -> u8 {
        self.table[value as usize]
    }
}