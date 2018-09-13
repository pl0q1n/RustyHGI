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

        let depth = grid.len() - 1;
        predictions.resize(depth + 1 as usize, PredictMap::new());

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

        for level in 1..(levels + 1) {
            let actual_step = 1 << (levels - level);

            let iter = (0..height)
                .step_by(actual_step)
                .flat_map(move |y| repeat(y).zip((0..width).step_by(actual_step)));

            println!("{}", level);
            for (line, column) in iter {
                if !positions.get_val(column, line) {
                    let (post_inter_value, predicted_value) = {
                        let mut curr_level = &predictions[level - 1];

                        let prediction = get_interp_pixels(
                            depth,
                            level,
                            (width, height),
                            (column, line),
                            curr_level,
                            0,
                        ).prediction();

                        let pix_value = input.get_pixel(column, line).data[0];
                        let post_inter_value = (pix_value as i32 - prediction as i32).abs() as u8;
                        (post_inter_value, prediction)
                    };
                    let quanted_postinter_value = metadata.quantizator.quantize(post_inter_value);

                    let pixel = gray(quanted_postinter_value.saturating_add(predicted_value));
                    input.put_pixel(column, line, pixel);
                    grid[level].push(quanted_postinter_value);
                    predictions[level]
                        .insert((column as usize, line as usize), quanted_postinter_value);

                    positions.set_val(column, line);
                }
            }
        }
        return grid;
    }
}

//////////////////

#[cfg(test)]
mod tests {

    use super::*;
    use decoder::{Decoder, DecoderGrayscale};
    use image;
    use utils::{Interpolator, Quantizator};

    #[test]
    fn losseless_compression_test() {
        let metadata = Metadata {
            quantizator: Quantizator::Loseless,
            interpolator: Interpolator::Crossed,
            dimensions: (8, 8),
            scale_level: 2,
        };

        let mut imgbuf = image::ImageBuffer::new(8, 8);

        // fill image with random pixels
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = image::Luma([(x * y) as u8]);
        }

        for line in imgbuf.chunks(imgbuf.width() as usize) {
            println!("{:?}", line);
        }

        let mut encoder = EncoderGrayscale {};
        let grid = encoder.encode(&metadata, imgbuf.clone());

        let mut decoder = DecoderGrayscale {};
        let image = decoder.decode(&metadata, &grid);

        for line in image.chunks(image.width() as usize) {
            println!("{:?}", line);
        }

        for (x, y, pixel) in imgbuf.enumerate_pixels() {
            assert!((pixel.data[0] as i32 - image[(x, y)].data[0] as i32).abs() < 10);
        }
    }
}
