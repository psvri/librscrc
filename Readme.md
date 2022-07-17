# librscrc

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/psvri/librscrc/tree/main.svg?style=shield)](https://dl.circleci.com/status-badge/redirect/gh/psvri/librscrc/tree/main) [![librscrc](https://img.shields.io/crates/v/librscrc)](https://crates.io/crates/librscrc) [![docs](https://img.shields.io/docsrs/librscrc)](https://docs.rs/librscrc/0.1.0/librscrc/)

Librscrc is a collection of crc32 algorithms with support for various approaches like simd and table based lookup and
custom polynomial implemented in rust.

Simd is currently supported on the following architectures

- x86
- x86_64
- aarch64

Unsafe code is used for calls to intrinsics. These can be opted out by setting ```default-features = false``` in
Cargo.toml, there by disabling simd and hardware crc intrinsics support.

## Performance

Your mileage may vary based on the hardware used. This section is meant to only give a comparison of various approaches.

- Nightly compiler is required to use hardware crc instructions on aarch64 with feature flag "nightly" enable.
- Simd and hardware instructions are used if the required cpu features are detected at run time. In case it's not found,
  it falls back to lookup based approach.
- Simd approach is based on the paper published
  by [intel](https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/fast-crc-computation-generic-polynomials-pclmulqdq-paper.pdf)

| Algorithm(crc32c) | x86_64(throughput) | aarch64(throughput) |
|-------------------|--------------------|---------------------|
| naive             | 249.97 MiB/s       | 221.81 MiB/s        |
| lookup            | 3.6584 GiB/s       | 2.2660 GiB/s        |
| hardware crc      | 10.483 GiB/s       | 21.026 GiB/s        |
| simd              | 25.479 GiB/s       | 18.079 GiB/s        |