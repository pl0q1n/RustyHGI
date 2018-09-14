#[macro_use]
extern crate criterion;
extern crate image;

extern crate hgi;

use hgi::{Metadata, Quantizator, Encoder, EncoderGrayscale, Interpolator};

use criterion::Criterion;

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

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Luma([(x * y) as u8]);
    }

    (metadata, imgbuf)
}

fn benchmarks(c: &mut Criterion) {
    c.bench_function("compression", |bencher| {
        let (metadata, image) = get_test_image(1920, 1080, 4);
        let mut encoder = EncoderGrayscale {};
        bencher.iter_with_setup(|| image.clone(), |image| encoder.encode(&metadata, image));
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(3);
    targets = benchmarks
);
criterion_main!(benches);
