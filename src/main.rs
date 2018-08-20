extern crate bit_vec;
extern crate image;
#[macro_use]
extern crate clap;

mod utils;
mod encoder;
mod decoder;

use utils::*;
use encoder::Encoder;
use encoder::EncoderGrayscale;
use decoder::Decoder;
use decoder::DecoderGrayscale;

fn main() {
    let matches = clap_app!(primify =>
        (version: "0.0.1")
        (author: "0xd34d10cc - anime")
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

    let mut decoder = DecoderGrayscale {};
    let img = decoder.decode(metadata, grid);

    img.save("test.png").unwrap();
}
