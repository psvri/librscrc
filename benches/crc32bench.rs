use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use librscrc::prelude::*;

fn bench_crc(c: &mut Criterion) {
    let mut group = c.benchmark_group("crc32");
    //let big: &[u8; 23] = b"1234567890Hello, World!";
    //let big = include_bytes!("../sample_files/test_data.txt");
    //let big = include_bytes!("../sample_files/test_data_odd_size.txt");
    let big = include_bytes!("../Cargo.lock");
    //let big = include_bytes!("../sample_files/granular_concrete_diff_1k.jpg");
    group.throughput(Throughput::Bytes(big.len() as u64));

    group.bench_with_input(
        BenchmarkId::new("crc32c_naive", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc32C::new_naive();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc32c_lookup", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc32C::new_lookup();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc32c_hardware", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc32C::new_hardware();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc32c_simd", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc32C::new_simd();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc32_simd", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc32::new_simd();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("custom_crc32_simd", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = CustomCrc32::new_simd(0x104C11DB7u64);
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("custom_crc32_lookup", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = CustomCrc32::new_lookup(0x04C11DB7);
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("custom_crc32_naive", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = CustomCrc32::new_naive(0x04C11DB7);
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc32fast", big.len() as u64),
        &big,
        |b, data| b.iter(|| crc32fast::hash(*data)),
    );
}

criterion_group!(benches, bench_crc);
criterion_main!(benches);
