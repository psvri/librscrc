# librscrc

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/psvri/librscrc/tree/main.svg?style=shield)](https://dl.circleci.com/status-badge/redirect/gh/psvri/librscrc/tree/main)

A rust implementation of crc32 and crc32c algorithm. There is support for simd and hardware approaches.

Unsafe code is used for calls to intrinsics. These can be opted out by setting ```default-features = false``` in cargo.toml, there by disabling simd and hardware support.

## Performance

Your milage may vary based on the hardware used. This section is meant to only give comparison of various approches.

- Nightly compiler is required to use hardware instructions on aarch64 with feature flag "nightly" enable.
- Simd and hardware instructions are used if the required cpu features are detected at run time. In case its not found, it falls back to lookup based approach.
- Simd approach is based on the paper published by [intel](https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/fast-crc-computation-generic-polynomials-pclmulqdq-paper.pdf)

| Algorithm(crc32c) | x86_64(throughtput) | aarch64(throughput) |
|---------- |---------------------|---------------------|
| naive | 249.97 MiB/s | 221.81 MiB/s |
| lookup | 3.6584 GiB/s | 2.2660 GiB/s |
| hardware | 10.483 GiB/s | 21.026 GiB/s |
| simd | 25.479 GiB/s | 18.079 GiB/s |