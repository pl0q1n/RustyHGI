extern crate bincode;
extern crate bit_vec;
extern crate byteorder;
extern crate flate2;
extern crate image;
extern crate serde;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::fs::File;

use structopt::StructOpt;

mod archive;
mod decoder;
mod encoder;
mod options;
mod utils;

use archive::Archive;
use decoder::{Decoder, DecoderGrayscale};
use encoder::{Encoder, EncoderGrayscale};
use options::{DecodeOpts, EncodeOpts, Opts};
use utils::{GridU8, Interpolator, Metadata};

fn encode(opts: &EncodeOpts) -> Result<(), Box<Error>> {
    let image = image::open(&opts.io.input)?.to_luma();
    let dimensions = image.dimensions();

    let metadata = Metadata {
        quantizator: opts.quantizator,
        interpolator: Interpolator::Crossed,
        dimensions,
        scale_level: opts.level,
    };

    let mut encoder = EncoderGrayscale {};
    let grid = encoder.encode(&metadata, image);
    println!("Grid size: {}", grid.len());

    let archive = Archive { metadata, grid };
    let mut output = File::create(&opts.io.output)?;
    archive.serialize_to_writer(&mut output)?;

    Ok(())
}

fn decode(opts: &DecodeOpts) -> Result<(), Box<Error>> {
    let mut input = File::open(&opts.io.input)?;
    let archive = Archive::<GridU8>::deserialize_from_reader(&mut input)?;
    let mut decoder = DecoderGrayscale {};
    let image = decoder.decode(&archive.metadata, &archive.grid);
    image.save(&opts.io.output)?;
    Ok(())
}

fn run() -> Result<(), Box<Error>> {
    match Opts::from_args() {
        Opts::Encode(opts) => encode(&opts),
        Opts::Decode(opts) => decode(&opts),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occured: {}", e);
    }
}
