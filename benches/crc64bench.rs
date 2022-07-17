use crc64fast::Digest;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use librscrc::prelude::*;

fn bench_crc(c: &mut Criterion) {
    let mut group = c.benchmark_group("crc64");
    //let big: &[u8; 23] = b"1234567890Hello, World!";
    //let big = include_bytes!("../sample_files/test_data.txt");
    //let big = include_bytes!("../sample_files/test_data_odd_size.txt");
    let big = include_bytes!("../Cargo.lock");
    //let big = include_bytes!("../sample_files/granular_concrete_diff_1k.jpg");
    group.throughput(Throughput::Bytes(big.len() as u64));

    group.bench_with_input(
        BenchmarkId::new("crc64ecma_naive", big.len() as u64),
        &big,
        |b, data| {
            b.iter(|| {
                let mut crc = Crc64ECMA::new_naive();
                crc.update(*data);
                crc.digest()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("crc64ecmafast", big.len() as u64),
        &big,
        |b, data| b.iter(|| {
            let mut crc = Digest::new();
            crc.write(*data);
            crc.sum64()
        }),
    );
}

criterion_group!(benches, bench_crc);
criterion_main!(benches);