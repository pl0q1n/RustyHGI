#[macro_use]
extern crate criterion;
extern crate bincode;
extern crate hgi;
extern crate image;

use image::GrayImage;

use hgi::interpolator::{self, Crossed, InterpolationType};
use hgi::quantizator::{self, Linear, QuantizationLevel};
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
    let (width, height) = (1920u32, 1080u32);
    let size = width * height;
    let levels = 4;

    c.bench(
        "memory",
        Benchmark::new("memory", move |bencher| {
            let mut v = vec![0u8; size as usize];
            for (i, x) in v.iter_mut().enumerate() {
                *x = i as u8;
            }
            let mut mem: Vec<u8> = Vec::with_capacity(size as usize);
            unsafe { mem.set_len(size as usize) };

            bencher.iter(|| {
                unsafe { ::std::ptr::copy_nonoverlapping(v.as_ptr(), mem.as_mut_ptr(), v.len()) };
            });
        }).throughput(Throughput::Bytes(size)),
    );

    c.bench(
        "nop_encode",
        Benchmark::new("nop_encode", move |bencher| {
            let (_metadata, image) = get_test_image(width, height, levels);
            let interpolator = interpolator::LeftTop;
            let quantizator = quantizator::NoOp;
            let mut encoder = Encoder::new(interpolator, quantizator, levels);
            bencher.iter_with_large_setup(|| image.clone(), |image| drop(encoder.encode(image)));
        }).throughput(Throughput::Bytes(size)),
    );

    c.bench(
        "encode",
        Benchmark::new("encode", move |bencher| {
            let (_metadata, image) = get_test_image(width, height, levels);
            let interpolator = Crossed;
            let quantizator = Linear::from(QuantizationLevel::Lossless);
            let mut encoder = Encoder::new(interpolator, quantizator, levels);
            bencher.iter_with_large_setup(|| image.clone(), |image| drop(encoder.encode(image)));
        }).throughput(Throughput::Bytes(size)),
    );

    c.bench(
        "decode",
        Benchmark::new("decode", move |bencher| {
            let (_metadata, image) = get_test_image(width, height, levels);
            let interpolator = Crossed;
            let quantizator = Linear::from(QuantizationLevel::Lossless);
            let mut encoder = Encoder::new(interpolator, quantizator, levels);
            let grid = encoder.encode(image);
            let mut decoder = Decoder::new(Crossed);

            bencher.iter_with_large_drop(|| decoder.decode((width, height), levels, &grid));
        }).throughput(Throughput::Bytes(size)),
    );

    c.bench_function("serialization", move |bencher| {
        let (metadata, image) = get_test_image(width, height, levels);
        let interpolator = Crossed;
        let quantizator = Linear::from(QuantizationLevel::Lossless);
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

    c.bench_function("compression", move |bencher| {
        let (metadata, image) = get_test_image(width, height, levels);
        let interpolator = Crossed;
        let quantizator = Linear::from(QuantizationLevel::Lossless);
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
