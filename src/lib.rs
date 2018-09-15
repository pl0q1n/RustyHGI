extern crate bincode;
extern crate bit_vec;
extern crate byteorder;
extern crate flate2;
extern crate image;
extern crate serde;
#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;

mod utils;
mod encoder;
mod archive;
mod decoder;

pub use self::utils::{Metadata, Quantizator, Interpolator};
pub use self::encoder::{Encoder, EncoderGrayscale};


#[cfg(test)]
mod tests {
    use super::*;
    use decoder::{Decoder, DecoderGrayscale};
    use image;
    use utils::{Interpolator, Quantizator};

    type Pixel = image::Luma<u8>;
    type Subpixel = <Pixel as image::Pixel>::Subpixel;
    type Container = Vec<Subpixel>;
    type GrayscaleBuffer = image::ImageBuffer<Pixel, Container>;

    fn get_test_image(width: u32, height: u32, levels: usize) -> (Metadata, GrayscaleBuffer) {
        let metadata = Metadata {
            quantizator: Quantizator::Loseless,
            interpolator: Interpolator::Crossed,
            width: width,
            height: height,
            scale_level: levels,
        };

        let mut imgbuf = image::ImageBuffer::new(width, height);

        // fill image with random pixels
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = image::Luma([(x * y) as u8]);
        }

        (metadata, imgbuf)
    }

    #[test]
    fn losseless_compression_test() {
        let (metadata, imgbuf) = get_test_image(8, 8, 3);

        for line in imgbuf.chunks(imgbuf.width() as usize) {
            println!("{:2?}", line);
        }

        let mut encoder = EncoderGrayscale {};
        let grid = encoder.encode(&metadata, imgbuf.clone());

        let mut decoder = DecoderGrayscale {};
        let image = decoder.decode(&metadata, &grid);

        let line: String = ::std::iter::repeat('-')
            .take(imgbuf.width() as usize * 4)
            .collect();
        println!("{}", line);
        for line in image.chunks(image.width() as usize) {
            println!("{:2?}", line);
        }

        for (x, y, pixel) in imgbuf.enumerate_pixels() {
            let before = pixel.data[0] as i32;
            let after = image[(x, y)].data[0] as i32;

            assert!((before - after).abs() < 10);
        }
    }
}
