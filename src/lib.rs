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
pub mod interpolator;
pub mod quantizator;
mod utils;

pub use self::archive::{Archive, Metadata};
pub use self::decoder::Decoder;
pub use self::encoder::Encoder;

#[cfg(test)]
mod tests {
    use image::{GrayImage, Luma};
    use std::io;

    use archive::{Archive, Metadata};
    use decoder::Decoder;
    use encoder::Encoder;
    use interpolator::{Crossed, InterpolationType};
    use quantizator::{Linear, QuantizationLevel, Quantizator};

    fn get_test_image(width: u32, height: u32) -> GrayImage {
        let mut image = GrayImage::new(width, height);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Luma([(x * y) as u8]);
        }

        image
    }

    fn test_error(quantization_level: QuantizationLevel) {
        let levels = 3;
        let (width, height) = (8, 8);
        let image = get_test_image(width, height);

        for line in image.chunks(image.width() as usize) {
            println!("{:2?}", line);
        }

        let quantizator = Linear::from(quantization_level);
        let max_error = quantizator.error() as usize;
        let interpolator = Crossed;
        let mut encoder = Encoder::new(interpolator, quantizator, levels);
        let grid = encoder.encode(image.clone());

        let mut decoder = Decoder::new(Crossed);
        let image = decoder.decode((width, height), &grid);

        let line: String = ::std::iter::repeat('-')
            .take(image.width() as usize * 4)
            .collect();
        println!("{}", line);
        for line in image.chunks(image.width() as usize) {
            println!("{:2?}", line);
        }

        for (x, y, pixel) in image.enumerate_pixels() {
            let before = i32::from(pixel.data[0]);
            let after = i32::from(image[(x, y)].data[0]);
            let diff = (before - after).abs() as usize;
            assert!(diff <= max_error);
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
        let levels = 3;
        let (width, height) = (8, 8);
        let image = get_test_image(8, 8);
        let interpolator = Crossed;
        let quantization_level = QuantizationLevel::Lossless;
        let quantizator = Linear::from(quantization_level);
        let mut encoder = Encoder::new(interpolator, quantizator, levels);
        let grid = encoder.encode(image);

        let metadata = Metadata {
            quantization_level,
            interpolation: InterpolationType::Crossed,
            width,
            height,
            scale_level: levels,
        };
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
