#![feature(stmt_expr_attributes)]

extern crate bincode;
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
use std::io::{Write, BufWriter, BufReader};
use std::fs::File;
use std::path::Path;

use structopt::StructOpt;

mod archive;
mod decoder;
mod encoder;
mod options;
mod utils;
mod quantizator;
mod interpolator;

use archive::Archive;
use decoder::Decoder;
use encoder::Encoder;
use interpolator::Crossed;
use quantizator::Linear;
use options::{IO, EncodingOptions, Opts};
use utils::{GridU8, InterpolationType, Metadata};


fn encode(io: &IO, opts: &EncodingOptions) -> Result<(), Box<Error>> {
    let image = image::open(&io.input)?.to_luma();
    let quantizator = Linear::from(opts.quantization_level);
    let interpolator = Crossed;
    let mut encoder = Encoder::new(interpolator, quantizator, opts.level);
    let (width, height) = image.dimensions();
    let grid = encoder.encode(image);

    let metadata = Metadata {
        quantization_level: opts.quantization_level,
        interpolation: InterpolationType::Crossed,
        width,
        height,
        scale_level: opts.level,
    };
    let archive = Archive { metadata, grid };
    let mut output = BufWriter::new(File::create(&io.output)?);
    archive.serialize_to_writer(&mut output)?;

    Ok(())
}

fn decode(io: &IO) -> Result<(), Box<Error>> {
    let mut input = BufReader::new(File::open(&io.input)?);
    let archive = Archive::<GridU8>::deserialize_from_reader(&mut input)?;
    let dimensions = (archive.metadata.width, archive.metadata.height);
    let mut decoder = Decoder::new(Crossed);
    let image = decoder.decode(dimensions, &archive.grid);
    image.save(&io.output)?;
    Ok(())
}

fn test(input: &Path, suffix: &str, opts: &EncodingOptions) -> Result<(), Box<Error>> {
    let image_before = image::open(input)?.to_luma();

    let quantizator = Linear::from(opts.quantization_level);
    let interpolator = Crossed;
    let mut encoder = Encoder::new(interpolator, quantizator, opts.level);
    let grid = encoder.encode(image_before.clone());

    let mut decoder = Decoder::new(Crossed);
    let image_after = decoder.decode(image_before.dimensions(), &grid);

    let mut sd = 0usize;
    for (x, y, before) in image_before.enumerate_pixels() {
        let before = before.data[0];
        let after = image_after[(x, y)].data[0];

        let diff = (before as i32 - after as i32).abs() as usize;

        sd += diff * diff;
    }

    let metadata = Metadata {
        quantization_level: opts.quantization_level,
        interpolation: InterpolationType::Crossed,
        width: image_before.width(),
        height: image_before.height(),
        scale_level: opts.level,
    };
    let archive = Archive { metadata, grid };
    let mut buffer = Vec::new();
    archive.serialize_to_writer(&mut buffer)?;

    let uncompressed = image_before.height() * image_before.width();
    sd /= uncompressed as usize;
    let compressed = buffer.len();
    println!("Uncompressed: {} kb", uncompressed / 1024);
    println!("Compressed:   {} kb", compressed / 1024);
    println!("Ratio:        {:.2}", uncompressed as f64 / compressed as f64);
    println!("SD:           {:.2}", (sd as f64).sqrt());

    let filename = input.file_stem().unwrap().to_string_lossy().into_owned() + suffix;
    image_after.save(filename.clone() + ".png")?;

    let mut output = BufWriter::new(File::create(filename + ".hgi")?);
    output.write_all(&buffer)?;

    Ok(())
}

fn run() -> Result<(), Box<Error>> {
    match Opts::from_args() {
        Opts::Encode { io, options } => encode(&io, &options),
        Opts::Decode { io } => decode(&io),
        Opts::Test { input, suffix, options } => test(&input, &suffix, &options)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occured: {}", e);
    }
}