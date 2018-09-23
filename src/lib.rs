#![feature(stmt_expr_attributes)]

extern crate bincode;
extern crate byteorder;
extern crate flate2;
extern crate image;
extern crate serde;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

mod archive;
mod decoder;
mod encoder;
mod utils;

pub use self::archive::Archive;
pub use self::decoder::{Decoder, DecoderGrayscale};
pub use self::encoder::{Encoder, EncoderGrayscale};
pub use self::utils::{Interpolator, Metadata, QuantizationLevel};

#[cfg(test)]
mod tests {
    use image;
    use std::io;

    use archive::Archive;
    use decoder::{Decoder, DecoderGrayscale};
    use encoder::{Encoder, EncoderGrayscale};
    use utils::{Interpolator, Metadata, QuantizationLevel};

    type Pixel = image::Luma<u8>;
    type Subpixel = <Pixel as image::Pixel>::Subpixel;
    type Container = Vec<Subpixel>;
    type GrayscaleBuffer = image::ImageBuffer<Pixel, Container>;

    fn get_test_image(
        width: u32,
        height: u32,
        levels: usize,
        quantizator: QuantizationLevel,
    ) -> (Metadata, GrayscaleBuffer) {
        let metadata = Metadata {
            quantization_level: quantizator,
            interpolator: Interpolator::Crossed,
            width: width,
            height: height,
            scale_level: levels,
        };

        let mut imgbuf = image::ImageBuffer::new(width, height);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = image::Luma([(x * y) as u8]);
        }

        (metadata, imgbuf)
    }

    fn test_error(quantizator: QuantizationLevel) {
        let (metadata, imgbuf) = get_test_image(8, 8, 3, quantizator);

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
            let expected_error = quantizator.max_error();
            for (x, y, pixel) in imgbuf.enumerate_pixels() {
                let before = pixel.data[0] as i32;
                let after = image[(x, y)].data[0] as i32;
                let diff = (before - after).abs() as usize;
                assert!(diff <= expected_error);
            }
    }

    #[test]
    fn lossless_compression() {
        test_error(QuantizationLevel::Lossless);
    }

    #[test]
    fn low_compression() {
        test_error(QuantizationLevel::Low);
    }

    #[test]
    fn medium_compression() {
        test_error(QuantizationLevel::Medium);
    }

    #[test]
    fn high_compression() {
        test_error(QuantizationLevel::High);
    }

    #[test]
    fn serde() {
        let (metadata, imgbuf) = get_test_image(8, 8, 3, QuantizationLevel::Lossless);
        let mut encoder = EncoderGrayscale {};
        let grid = encoder.encode(&metadata, imgbuf);
        let archive = Archive { metadata, grid };
        let mut buffer = Vec::new();
        let res = archive.serialize_to_writer(&mut buffer);
        assert!(res.is_ok());
        let mut cursor = io::Cursor::new(&buffer);
        let res = Archive::deserialize_from_reader(&mut cursor);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), archive);
    }
}
