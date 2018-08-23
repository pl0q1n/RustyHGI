extern crate bit_vec;
extern crate serde;
extern crate bincode;
extern crate byteorder;
extern crate image;
extern crate flate2;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

mod utils;
mod encoder;
mod decoder;
mod archive;

use utils::*;
use encoder::Encoder;
use encoder::EncoderGrayscale;
use decoder::Decoder;
use decoder::DecoderGrayscale;
use archive::Archive;
use std::fs::File;
use std::io::Write;

fn main() {
    let matches = clap_app!(primify =>
        (version: "0.1.0")
        (author: "pl0q1n & 0xd34d10cc")
        (about: "Actually trying to compress the image")
        (@arg INPUT:      -i --input       +takes_value +required "Input file name")
        (@arg LEVEL:      -l --level       +takes_value           "Scale level of grid")
        //(@arg QUANTIZATOR -q --quantizator +takes_value           "Type of Quantizator")
    ).get_matches();

    let image_path = matches.value_of("INPUT").unwrap();
    let res = image::open(&image_path);
    if let Err(e) = res {
        println!("An error occurred: {}", e);
        panic!();
    }
    let img = res.unwrap().to_luma();
    let grid_level = matches
        .value_of("LEVEL")
        .map(|l| l.parse())
        .unwrap_or(Ok(4usize))
        .unwrap();
    let quantizator = Quantizator::HighCompression;
    let interpolator = Interpolator::Crossed;
    let dimension = img.dimensions();
    let metadata = Metadata {
        quantizator,
        interpolator,
        dimension,
        scale_level: grid_level,
    };
    println!("image path {}", image_path);
    println!("dimensions {:?}", dimension);
    println!("Grid level {}", grid_level);

    let mut encoder = EncoderGrayscale {};
    let mut grid = encoder.encode(metadata.clone(), img);

    println!("grid size: {}", grid.len());
    {
        let arch = Archive{metadata, grid};
        let mut file = File::create("compressed_not").unwrap();
        arch.serialize_to_writer(&mut file).unwrap();
    }

    let mut file = File::open("compressed_not").unwrap();
    let archive: Archive<GridU8> = Archive::deserialize_from_reader(&mut file).unwrap();
    let mut decoder = DecoderGrayscale {};
    let img = decoder.decode(&archive.metadata, &archive.grid);
    


    img.save("test.png").unwrap();
}
