use image::{GrayImage, ImageBuffer, Luma};
use std::iter::repeat;
use utils::{get_interp_pixels, get_predicted_val, GridU8, Metadata, PositionMap, PredictMap};

pub struct DecoderGrayscale {}

pub trait Decoder {
    type Input;
    type Output;

    fn decode(&mut self, metadata: &Metadata, input: &Self::Input) -> Self::Output;
}

impl Decoder for DecoderGrayscale {
    type Input = GridU8;
    type Output = GrayImage;

    fn decode(&mut self, metadata: &Metadata, input: &Self::Input) -> Self::Output {
        let (width, height) = metadata.dimension;
        let mut grid_ind = 0;
        let mut img = ImageBuffer::new(width, height);
        let mut grid_depth = 0usize;
        let depth = input.len() - 1;
        let mut ind = 2usize.pow(depth as u32);

        let mut positions = PositionMap::new(width, height);
        let mut predictions = Vec::<PredictMap>::new();

        predictions.resize(depth + 1 as usize, PredictMap::new());

        for x in (0..width).step_by(ind) {
            for y in (0..height).step_by(ind) {
                img.put_pixel(
                    x,
                    y,
                    Luma {
                        data: [input[grid_depth][grid_ind]],
                    },
                );
                predictions[grid_depth]
                    .insert((x as usize, y as usize), input[grid_depth][grid_ind]);
                grid_ind += 1;
                positions.set_val(x, y);
            }
        }

        ind /= 2;
        grid_depth += 1;

        while ind >= 1 {
            grid_ind = 0;
            let iter = (0..width)
                .step_by(ind)
                .into_iter()
                .flat_map(move |x| (0..height).step_by(ind).zip(repeat(x)));

            for (y, x) in iter {
                if !positions.get_val(x, y) {
                    let post_inter_value = {
                        let mut curr_level = &predictions[grid_depth - 1];

                        let values = get_interp_pixels(
                            depth,
                            grid_depth,
                            (width, height),
                            (x, y),
                            curr_level,
                            input[grid_depth][grid_ind],
                        );
                        let predicted_value = get_predicted_val(values);
                        let post_inter_value = ((input[grid_depth][grid_ind] as u16
                            + predicted_value as u16)
                            / 2) as u8;
                        post_inter_value
                    };
                    img.put_pixel(
                        x,
                        y,
                        Luma {
                            data: [post_inter_value],
                        },
                    );
                    predictions[grid_depth].insert((x as usize, y as usize), post_inter_value);
                    grid_ind += 1;
                }
                positions.set_val(x, y);
            }
            ind /= 2;
            grid_depth += 1;
            println!("{}", grid_depth);
        }
        return img;
    }
}
