extern crate image;
extern crate divans;
#[macro_use]
extern crate clap;

use image::GenericImage;
use bit_vec::BitVec;
use std::io;
use std::fs::File;
use std::io::{Read, Write};
use std::env;
use std::vec;
use std::collections::HashMap;



enum  Quantizator {
    LoselessCompression,
    LowCompression,
    MediumCompression,
    HighCompression
}

enum Interpolator {
    Crossed,
    Line,
    Previous
}

struct PositionMap {
    positions: BitVec,
    width: u32,
    height: u32,
}

impl PositionMap {
    fn get_val(&self, x: u32, y: u32) -> bool {
        return self.positions.get(x * self.width + y )
    }
}

struct Params {
    quantizator : Quantizator,
    interpolator: Interpolator,
    dimension :  (u32, u32),
    scale_level: u8,

}

struct Writer {
    img : image::GrayImage
}

struct Encoder {

}

struct Reader {
    img : image::GrayImage
}

struct Decoder {

}

fn is_in_prev_lvl(x: usize, ind: usize, grid_depth: usize, curr_depth: usize) -> bool {
    let curr_lvl_check = x % (ind * 2) == 0;
    if !curr_lvl_check {
        return curr_lvl_check;
    };
    for lvl in 0..(grid_depth - 1) {
        let curr_step = 2usize.pow((curr_depth - lvl) as u32);
        if x % curr_step == 0 {
            return false;
        }
    }
    return false;
}

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
    let grid_level = matches.value_of("LEVEL").map(|l| l.parse()).unwrap_or(Ok(4usize)).unwrap();

    println!("image path {}", image_path);
    println!("dimensions {:?}", img.dimensions());
    println!("Grid level {}", grid_level);

    let (width, height) = img.dimensions();
    let mut grid = Vec::<Vec<u8>>::with_capacity(grid_level + 1);
    //let poses = PositionMap { positions: }

    grid.resize(grid_level + 1, Vec::new());
    let mut grid_depth = 0usize;
    let mut ind = 2usize.pow(grid.len() as u32 - 1); 
    
    type CoordHolder = (usize, usize);
    type PredictMap = HashMap<CoordHolder, u8>;

    let mut positions = PositionMap{ positions: BitVec::new(), width, height };
    let mut predictions = Vec::<PredictMap>::new();
    while ind >= 1 {
        let mut k = 0;
        let mut curr_level = &predictions[grid_depth - 1]; 
        for x in (0..width).step_by(ind) {
            for y in (0..height).step_by(ind) {
                if !positions.get_val(x, y) {
                    if grid_depth == 0 {
                        grid[grid_depth].push(img.get_pixel(x, y).data[0]);
                        predictions[grid_depth][&(x as usize, y as usize)] = grid[grid_depth][k];
                    }
                    else {

                    }
                }
            }
        }
    } 
    println!("Hello, world!");
}
