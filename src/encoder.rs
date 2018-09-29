use image::{GrayImage, GenericImage};
use interpolator::Interpolator;
use quantizator::Quantizator;
use grid::Grid;
use utils::{gray, traverse_level};

pub struct Encoder<I, Q> {
    interpolator: I,
    quantizator: Q,
    scale_level: usize,
}

impl<I, Q> Encoder<I, Q>
where
    Q: Quantizator,
    I: Interpolator,
{
    pub fn new(interpolator: I, quantizator: Q, scale_level: usize) -> Self {
        Encoder {
            quantizator,
            interpolator,
            scale_level,
        }
    }

    fn initialize_first_level(&self, image: &GrayImage, grid: &mut Grid) {
        let (width, height) = image.dimensions();
        let step: usize = 1 << self.scale_level;

        // initialize first level with pixel values
        for line in (0..height).step_by(step) {
            for column in (0..width).step_by(step) {
                let pixel = unsafe { image.unsafe_get_pixel(column, line).data[0] };
                unsafe { grid.set((column, line), pixel) };
            }
        }
    }

    pub fn encode(&mut self, mut input: GrayImage) -> Grid {
        let (width, height) = input.dimensions();
        let levels = self.scale_level;
        let mut grid = Grid::new(width as usize, height as usize);
        self.initialize_first_level(&input, &mut grid);

        for level in 0..levels {
            {
            let process_pixel = #[inline(always)]
            |column, line| {
                let prediction =
                    self.interpolator
                        .interpolate(levels, level + 1, (column, line), &input);

                let actual_value = unsafe { input.unsafe_get_pixel(column, line).data[0] };
                let diff = actual_value.wrapping_sub(prediction);
                let mut quanted_diff = self.quantizator.quantize(diff);

                let overflow = prediction.checked_add(quanted_diff).is_none();
                let overflow_is_expected = prediction.checked_add(diff).is_none();
                if overflow != overflow_is_expected {
                    quanted_diff = diff;
                }

                unsafe { grid.set((column, line), quanted_diff) };
                let pixel = gray(prediction.wrapping_add(quanted_diff));
                unsafe { input.unsafe_put_pixel(column, line, pixel) };
            };

            traverse_level(level, levels, width, height, process_pixel);
            }
        }

        grid
    }
}
