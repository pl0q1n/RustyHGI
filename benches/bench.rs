#[macro_use]
extern crate criterion;
extern crate bincode;
extern crate hgi;
extern crate image;

use image::GrayImage;

use hgi::interpolator::{Crossed, InterpolationType};
use hgi::quantizator::{Linear, QuantizationLevel};
use hgi::{Archive, Decoder, Encoder, Metadata};

use criterion::{Benchmark, Criterion, Throughput};

fn get_test_image(width: u32, height: u32, levels: usize) -> (Metadata, GrayImage) {
    let metadata = Metadata {
        quantization_level: QuantizationLevel::Medium,
        interpolation: InterpolationType::Crossed,
        width: width,
        height: height,
        scale_level: levels,
    };

    let mut imgbuf = GrayImage::new(width, height);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Luma([(x * y) as u8]);
    }

    (metadata, imgbuf)
}

fn benchmarks(c: &mut Criterion) {
    c.bench(
        "encode",
        Benchmark::new("encode", |bencher| {
            let levels = 4;
            let (_metadata, image) = get_test_image(1920, 1080, levels);
            let interpolator = Crossed;
            let quantizator = Linear::from(QuantizationLevel::Loseless);
            let mut encoder = Encoder::new(interpolator, quantizator, levels);
            bencher.iter_with_large_setup(|| image.clone(), |image| drop(encoder.encode(image)));
        }).throughput(Throughput::Bytes(1920 * 1080)),
    );

    c.bench(
        "decode",
        Benchmark::new("decode", |bencher| {
            let levels = 4;
            let (width, height) = (1920, 1080);
            let (_metadata, image) = get_test_image(width, height, levels);
            let interpolator = Crossed;
            let quantizator = Linear::from(QuantizationLevel::Loseless);
            let mut encoder = Encoder::new(interpolator, quantizator, levels);
            let grid = encoder.encode(image);
            let mut decoder = Decoder::new(Crossed);

            bencher.iter_with_large_drop(|| decoder.decode((width, height), &grid));
        }).throughput(Throughput::Bytes(1920 * 1080)),
    );

    c.bench_function("serialization", |bencher| {
        let levels = 4;
        let (width, height) = (1920, 1080);
        let (metadata, image) = get_test_image(width, height, levels);
        let interpolator = Crossed;
        let quantizator = Linear::from(QuantizationLevel::Loseless);
        let mut encoder = Encoder::new(interpolator, quantizator, levels);
        let grid = encoder.encode(image);
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
        let levels = 4;
        let (width, height) = (1920, 1080);
        let (metadata, image) = get_test_image(width, height, levels);
        let interpolator = Crossed;
        let quantizator = Linear::from(QuantizationLevel::Loseless);
        let mut encoder = Encoder::new(interpolator, quantizator, levels);

        bencher.iter_with_large_setup(
            || {
                (
                    Vec::with_capacity(width as usize * height as usize),
                    image.clone(),
                )
            },
            |(mut buffer, image)| {
                let grid = encoder.encode(image);
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
