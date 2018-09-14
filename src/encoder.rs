use image::GrayImage;
use std::cmp;
use std::iter::repeat;
use utils::{get_interp_pixels, gray, GridU8, Metadata, PositionMap, PredictMap};

pub struct EncoderGrayscale {}

pub trait Encoder {
    type Input;
    type Output;

    fn encode(&mut self, metadata: &Metadata, input: Self::Input) -> Self::Output;
}

impl Encoder for EncoderGrayscale {
    type Input = GrayImage;
    type Output = GridU8;

    fn encode(&mut self, metadata: &Metadata, mut input: Self::Input) -> Self::Output {
        let (width, height) = metadata.dimensions;
        let mut grid = GridU8::with_capacity(metadata.scale_level + 1);
        grid.resize(metadata.scale_level + 1, Vec::new());

        let levels = metadata.scale_level;
        let mut positions = PositionMap::new(width, height);
        let mut predictions = Vec::<PredictMap>::new();
        predictions.resize(levels + 1 as usize, PredictMap::new());

        let level = 0;
        let step = 1 << levels;
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(1 << levels) {
                let pix_val = input.get_pixel(column, line).data[0];

                grid[level].push(pix_val);
                predictions[level].insert((column as usize, line as usize), pix_val);
                positions.set_val(column, line);
            }
        }

        for level in 0..levels {
            let e = levels - level;
            let start = 1 << (e - 1);
            let step = 1 << e;
            let substep = start;

            let encode_pixel = |column: u32,
                                line: u32,
                                previous_level: &PredictMap,
                                input: &GrayImage|
             -> (u8, u8) {
                let (post_inter_value, predicted_value) = {
                    let prediction = get_interp_pixels(
                        levels,
                        level + 1,
                        (width, height),
                        (column, line),
                        previous_level,
                        0,
                    ).prediction();

                    let pix_value = input.get_pixel(column, line).data[0];
                    let post_inter_value = (pix_value as i32 - prediction as i32).abs() as u8;
                    (post_inter_value, prediction)
                };
                let quanted_postinter_value = metadata.quantizator.quantize(post_inter_value);
                (quanted_postinter_value, predicted_value)
            };

            let mut column = 0;
            while column < width {
                for line in (start..height).step_by(step) {
                    let (quanted_postinter_value, predicted_value) = encode_pixel(column, line, &predictions[level], &input);

                    let pixel = gray(quanted_postinter_value.saturating_add(predicted_value));
                    input.put_pixel(column, line, pixel);
                    grid[level + 1].push(quanted_postinter_value);
                    predictions[level + 1]
                        .insert((column as usize, line as usize), quanted_postinter_value);
                }

                column += substep;
                if column >= width {
                    break;
                }

                for line in (0..height).step_by(substep as usize) {
                    let (quanted_postinter_value, predicted_value) = encode_pixel(column, line, &predictions[level], &input);
                    let pixel = gray(quanted_postinter_value.saturating_add(predicted_value));
                    input.put_pixel(column, line, pixel);
                    grid[level + 1].push(quanted_postinter_value);
                    predictions[level + 1]
                        .insert((column as usize, line as usize), quanted_postinter_value);
                }
                column += substep;
            }
        }
        return grid;
    }
}