use bincode;
use byteorder::{WriteBytesExt, ReadBytesExt, LE};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{self, Write, Read};
use utils::{GridU8, Metadata};
use std::error::Error;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;

const MAGIC : u32 = 0xBAADA555;

#[derive(Serialize, Deserialize)]
pub struct Archive<G> {
    pub metadata: Metadata,
    pub grid: G,
}

impl<G: Serialize + DeserializeOwned> Archive<G> {
    pub fn serialize_to_writer<W: Write>(&self, mut w: &mut W) -> Result<(), Box<Error>> {
        w.write_u32::<LE>(MAGIC)?;
        bincode::serialize_into(&mut w, &self.metadata)?;
        let mut encoder = DeflateEncoder::new(w, Compression::best());
        bincode::serialize_into(&mut encoder, &self.grid)?;
        Ok(())
    }

    pub fn deserialize_from_reader<R: Read>(mut r: &mut R) -> Result<Self, Box<Error>> where Archive<G>: 'static {
        let magic = r.read_u32::<LE>()?;
        if magic != MAGIC {
            return Err("incorrect magic number".into());
        };
        let metadata = bincode::deserialize_from::<_, Metadata>(&mut r)?;
        let mut decoder = DeflateDecoder::new(r);
        let grid = bincode::deserialize_from::<_, G>(decoder)?;
        Ok(Archive{metadata, grid})
    }
}

