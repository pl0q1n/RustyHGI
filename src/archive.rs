use std::error::Error;
use std::io::{Read, Write};

use bincode;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::de::DeserializeOwned;
use serde::Serialize;

use interpolator::InterpolationType;
use quantizator::QuantizationLevel;

const MAGIC: u32 = 0xBAADA555;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub quantization_level: QuantizationLevel,
    pub interpolation: InterpolationType,
    pub width: u32,
    pub height: u32,
    pub scale_level: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Archive<G> {
    pub metadata: Metadata,
    pub grid: G,
}

impl<G: Serialize + DeserializeOwned> Archive<G> {
    pub fn serialize_to_writer<W: Write>(&self, mut w: &mut W) -> Result<(), Box<Error>> {
        w.write_u32::<LE>(MAGIC)?;
        bincode::serialize_into(&mut w, &self.metadata)?;
        let mut buffer = Vec::with_capacity(bincode::serialized_size(&self.grid)? as usize);
        bincode::serialize_into(&mut buffer, &self.grid)?;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&buffer)?;
        let compressed_bytes = encoder.finish()?;
        w.write_all(&compressed_bytes)?;
        Ok(())
    }

    pub fn deserialize_from_reader<R: Read>(mut r: &mut R) -> Result<Self, Box<Error>>
    where
        Archive<G>: 'static,
    {
        let magic = r.read_u32::<LE>()?;
        if magic != MAGIC {
            return Err("incorrect magic number".into());
        };
        let metadata: Metadata = bincode::deserialize_from(&mut r)?;
        let decoder = DeflateDecoder::new(r);
        let grid: G = bincode::deserialize_from(decoder)?;
        Ok(Archive { metadata, grid })
    }
}
