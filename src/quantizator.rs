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
    fn error(&self) -> u8;
}

pub struct NoOp;

impl From<QuantizationLevel> for NoOp {
    fn from(_: QuantizationLevel) -> Self {
        NoOp
    }
}

impl Quantizator for NoOp {
    #[inline(always)]
    fn quantize(&self, value: u8) -> u8 {
        value
    }

    fn error(&self) -> u8 {
        0
    }
}

pub struct Linear {
    table: [u8; 256],
    error: u8
}

impl From<QuantizationLevel> for Linear {
    fn from(level: QuantizationLevel) -> Self {
        let error: u8 = match level {
            QuantizationLevel::Lossless => 0,
            QuantizationLevel::Low => 10,
            QuantizationLevel::Medium => 20,
            QuantizationLevel::High => 30,
        };

        let scale = 2 * error as usize + 1;
        let quantize = |x| {
            let r = (x as usize + error as usize) / scale;
            let v = r * scale;
            v as u8
        };

        let mut table = [0; 256];
        for (i, entry) in table.iter_mut().enumerate() {
            *entry = quantize(i as u8);
        }
        Linear { table, error }
    }
}

impl Quantizator for Linear {
    #[inline(always)]
    fn quantize(&self, value: u8) -> u8 {
        self.table[value as usize]
    }

    fn error(&self) -> u8 {
        self.error
    }
}