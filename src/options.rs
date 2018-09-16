use std::path::PathBuf;

use utils::QuantizationLevel;


#[derive(StructOpt, Debug)]
#[structopt(
    name = "hgi",
    about = "Actually trying to compress the image",
    author = "pl0q1n & 0xd34d10cc",
    version = "0.1.0"
)]
pub enum Opts {
    #[structopt(name = "encode")]
    Encode {
        #[structopt(flatten)]
        io: IO,

        #[structopt(flatten)]
        options: EncodingOptions
    },

    #[structopt(name = "decode")]
    Decode {
        #[structopt(flatten)]
        io: IO
    },

    #[structopt(name = "test")]
    Test {
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        #[structopt(short="s", long="suffix", default_value="")]
        suffix: String, // output files suffix

        #[structopt(flatten)]
        options: EncodingOptions
    },
}

#[derive(StructOpt, Debug)]
pub struct IO {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: PathBuf,
}



#[derive(StructOpt, Debug)]
pub struct EncodingOptions {
    #[structopt(short = "l", long = "level", default_value = "4")]
    pub level: usize,

    #[structopt(
        short = "q",
        long = "quantizator",
        raw(possible_values = "&QuantizationLevel::variants()", case_insensitive = "true"),
        default_value = "medium"
    )]
    pub quantization_level: QuantizationLevel,
}