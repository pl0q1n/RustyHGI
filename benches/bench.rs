#[macro_use]
extern crate criterion;
extern crate bincode;
extern crate hgi;
extern crate image;

use hgi::{
    Archive, Decoder, DecoderGrayscale, Encoder, EncoderGrayscale, Interpolator, Metadata,
    QuantizationLevel,
};

use criterion::{Benchmark, Criterion, Throughput};

type Pixel = image::Luma<u8>;
type Subpixel = <Pixel as image::Pixel>::Subpixel;
type Container = Vec<Subpixel>;
type GrayscaleBuffer = image::ImageBuffer<Pixel, Container>;

fn get_test_image(width: u32, height: u32, levels: usize) -> (Metadata, GrayscaleBuffer) {
    let metadata = Metadata {
        quantization_level: QuantizationLevel::Medium,
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
    c.bench(
        "encode",
        Benchmark::new("encode", |bencher| {
            let (metadata, image) = get_test_image(1920, 1080, 4);
            let mut encoder = EncoderGrayscale {};
            bencher.iter_with_large_setup(
                || image.clone(),
                |image| drop(encoder.encode(&metadata, image)),
            );
        }).throughput(Throughput::Bytes(1920 * 1080)),
    );

    c.bench(
        "decode",
        Benchmark::new("decode", |bencher| {
            let (metadata, image) = get_test_image(1920, 1080, 4);
            let mut encoder = EncoderGrayscale {};
            let grid = encoder.encode(&metadata, image);
            let mut decoder = DecoderGrayscale {};

            bencher.iter_with_large_drop(|| decoder.decode(&metadata, &grid));
        }).throughput(Throughput::Bytes(1920 * 1080)),
    );

    c.bench_function("serialization", |bencher| {
        let (metadata, image) = get_test_image(1920, 1080, 4);
        let mut encoder = EncoderGrayscale {};
        let grid = encoder.encode(&metadata, image);
        let archive = Archive { metadata, grid };
        let serialized_size = bincode::serialized_size(&archive).unwrap() as usize;

        bencher.iter_with_large_setup(
            || Vec::with_capacity(serialized_size),
            |mut buffer| {
                archive.serialize_to_writer(&mut buffer).unwrap();
            },
        );
    });

    c.bench_function("compression", |bencher| {
        let (metadata, image) = get_test_image(1920, 1080, 4);
        let mut encoder = EncoderGrayscale {};

        bencher.iter_with_large_setup(
            || (Vec::with_capacity(1920 * 1080), image.clone()),
            |(mut buffer, image)| {
                let grid = encoder.encode(&metadata, image);
                let archive = Archive {
                    metadata: metadata.clone(),
                    grid,
                };
                archive.serialize_to_writer(&mut buffer).unwrap();
            },
        );
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(25);
    targets = benchmarks
);
criterion_main!(benches);
