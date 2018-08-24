use std::path::PathBuf;

use utils::Quantizator;


#[derive(StructOpt, Debug)]
#[structopt(
    name = "hgi",
    about = "Actually trying to compress the image",
    author = "pl0q1n & 0xd34d10cc",
    version = "0.1.0"
)]
pub enum Opts {
    #[structopt(name = "encode")]
    Encode(EncodeOpts),

    #[structopt(name = "decode")]
    Decode(DecodeOpts),
}

#[derive(StructOpt, Debug)]
pub struct IO {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct EncodeOpts {
    #[structopt(flatten)]
    pub io: IO,
    
    #[structopt(short = "l", long = "level", default_value = "4")]
    pub level: usize,

    #[structopt(
        short = "q",
        long = "quantizator",
        raw(possible_values = "&Quantizator::variants()", case_insensitive = "true"),
        default_value = "medium"
    )]
    pub quantizator: Quantizator,
}

#[derive(StructOpt, Debug)]
pub struct DecodeOpts {
    #[structopt(flatten)]
    pub io: IO
}
