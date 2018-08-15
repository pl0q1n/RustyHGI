extern crate bit_vec;
extern crate divans;
extern crate image;
#[macro_use]
extern crate clap;

use bit_vec::BitVec;
use image::GenericImage;
use image::Luma;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::vec;

enum Quantizator {
    LoselessCompression,
    LowCompression,
    MediumCompression,
    HighCompression,
}

enum Interpolator {
    Crossed,
    Line,
    Previous,
}

struct PositionMap {
    positions: BitVec,
    width: u32,
    height: u32,
}

impl PositionMap {
    fn get_val(&self, x: u32, y: u32) -> bool {
        return self.positions.get((x * self.width + y) as usize).unwrap();
    }
    fn set_val(&mut self, x: u32, y: u32) {
        self.positions.set((x * self.width + y) as usize, true);
    }
}

struct Params {
    quantizator: Quantizator,
    interpolator: Interpolator,
    dimension: (u32, u32),
    scale_level: u8,
}

struct Writer {
    img: image::GrayImage,
}

struct Encoder {}

struct Reader {
    img: image::GrayImage,
}

struct Decoder {}

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
    let mut img = res.unwrap().to_luma();
    let grid_level = matches
        .value_of("LEVEL")
        .map(|l| l.parse())
        .unwrap_or(Ok(4usize))
        .unwrap();

    println!("image path {}", image_path);
    println!("dimensions {:?}", img.dimensions());
    println!("Grid level {}", grid_level);

    let (width, height) = img.dimensions();
    let mut grid = Vec::<Vec<u8>>::with_capacity(grid_level + 1);
    let mut result = Vec::<Vec<u8>>::with_capacity(width as usize);
    result.resize(width as usize, vec![0; height as usize]);

    //let poses = PositionMap { positions: }

    grid.resize(grid_level + 1, Vec::new());
    let mut grid_depth = 0usize;
    let mut ind = 2usize.pow(grid.len() as u32 - 1);

    type CoordHolder = (usize, usize);
    type PredictMap = HashMap<CoordHolder, u8>;

    let mut positions = PositionMap {
        positions: BitVec::from_elem((width * height) as usize, false),
        width,
        height,
    };

    let mut predictions = Vec::<PredictMap>::new();

    let depth = grid.len() - 1;
    predictions.resize(depth + 1 as usize, PredictMap::new());

    while ind >= 1 {
        let mut k = 0;
        for x in (0..width).step_by(ind) {
            for y in (0..height).step_by(ind) {
                // println!("{} x {}", x, y);
                // for lol in (0.. width) {
                //     for kek in (0..height) {
                //         if positions.get_val(lol, kek) {
                //             print!("1");
                //         }
                //         else {
                //             print!("0");
                //         }
                //     }
                //     println!("");
                // }
                if !positions.get_val(x, y) {
                    if grid_depth == 0 {
                        grid[grid_depth].push(img.get_pixel(x, y).data[0]);
                        predictions[grid_depth]
                            .insert((x as usize, y as usize), img.get_pixel(x, y).data[0]);
                    } else {
                        let (post_inter_value, predicted_value) = {
                            let mut curr_level = &predictions[grid_depth - 1];
                            let mut left_top_val = 0;
                            let mut right_top_val = 0;
                            let mut left_bot_val = 0;
                            let mut right_bot_val = 0;
                            let is_in_prev_lvl = |x: u32| {
                                let curr_lvl_check = x % (ind as u32 * 2) == 0;
                                if !curr_lvl_check {
                                    return curr_lvl_check;
                                }
                                for lvl in 0..grid_depth - 1 {
                                    let curr_step = 2u32.pow((depth - lvl) as u32);
                                    if x % curr_step == 0 {
                                        return false;
                                    }
                                }
                                return true;
                            };

                            let mut x_mod = x % (ind * 2) as u32;
                            let mut y_mod = y % (ind * 2) as u32;

                            let mut x_top_cord = x - x_mod;
                            let mut x_bot_cord = x + (ind as u32 * 2 - x_mod);
                            let mut y_left_cord = y - y_mod;
                            let mut y_right_cord = y + (ind as u32 * 2 - y_mod);

                            let mut bot_out_of_range = x_bot_cord >= width;
                            let mut right_out_of_range = y_right_cord >= height;

                            if bot_out_of_range {
                                x_bot_cord = width - 1;
                            }
                            if right_out_of_range {
                                y_right_cord = height - 1;
                            }
                            if !bot_out_of_range
                                && !right_out_of_range
                                && is_in_prev_lvl(x_top_cord)
                                && is_in_prev_lvl(x_bot_cord)
                                && is_in_prev_lvl(y_left_cord)
                                && is_in_prev_lvl(y_right_cord)
                            {
                                //println!("{} | {}", x_bot_cord, y_right_cord);
                                left_top_val = *curr_level
                                    .get(&(x_top_cord as usize, y_left_cord as usize))
                                    .unwrap();
                                right_top_val = *curr_level
                                    .get(&(x_top_cord as usize, y_right_cord as usize))
                                    .unwrap();
                                left_bot_val = *curr_level
                                    .get(&(x_bot_cord as usize, y_left_cord as usize))
                                    .unwrap();
                                right_bot_val = *curr_level
                                    .get(&(x_bot_cord as usize, y_right_cord as usize))
                                    .unwrap();
                            } else {
                                if is_in_prev_lvl(x_top_cord) && is_in_prev_lvl(y_left_cord) {
                                    left_top_val = *curr_level
                                        .get(&(x_top_cord as usize, y_left_cord as usize))
                                        .unwrap();
                                } else {
                                    left_top_val = 255;
                                }

                                if is_in_prev_lvl(x_top_cord) && is_in_prev_lvl(y_right_cord) {
                                    right_top_val = *curr_level
                                        .get(&(x_top_cord as usize, y_right_cord as usize))
                                        .unwrap();
                                } else {
                                    right_top_val = 255;
                                }

                                if is_in_prev_lvl(x_bot_cord) && is_in_prev_lvl(y_left_cord) {
                                    left_bot_val = *curr_level
                                        .get(&(x_bot_cord as usize, y_left_cord as usize))
                                        .unwrap();
                                } else {
                                    left_bot_val = 255;
                                }

                                if is_in_prev_lvl(x_bot_cord) && is_in_prev_lvl(y_right_cord) {
                                    right_bot_val = *curr_level
                                        .get(&(x_bot_cord as usize, y_right_cord as usize))
                                        .unwrap();
                                } else {
                                    right_bot_val = 255;
                                }
                            }
                            let find_average_safety = | lhs: u8, rhs: u8 | {
                                ((lhs as usize + rhs as usize + 1) / 2) as u8
                            };

                            let left_inter = find_average_safety(left_top_val, left_bot_val);
                            let right_inter = find_average_safety(right_bot_val, right_top_val);
                            let top_inter = find_average_safety(right_top_val, left_top_val);
                            let bot_inter = find_average_safety(right_top_val, left_bot_val);

                            let predicted_value = ((left_inter as u16
                                + right_inter as u16
                                + top_inter as u16
                                + bot_inter as u16
                                + 1) / 4) as u8;
                            let post_inter_value =
                                img.get_pixel(x, y).data[0].saturating_sub(predicted_value);

                            result[x as usize][y as usize] = post_inter_value;
                            (post_inter_value, predicted_value)
                        };
                        // Quantization with precision = 15;
                        let quanted_postinter_value = ((2 * 15 + 1)
                            * ((post_inter_value as f64 + 15.0) / (2.0 * 15.0 + 1.0)).floor()
                                as usize
                            % 256) as u8;

                        img.put_pixel(
                            x,
                            y,
                            Luma {
                                data: [quanted_postinter_value.saturating_add(predicted_value)],
                            },
                        );
                        grid[grid_depth].push(quanted_postinter_value);
                        predictions[grid_depth].insert((x as usize, y as usize), post_inter_value);
                    }
                    k += 1;
                    positions.set_val(x, y);
                }
            }
        }
        ind /= 2;
        grid_depth += 1;
        println!("{}", grid_depth);
    }
}
