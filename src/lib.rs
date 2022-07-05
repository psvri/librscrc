#![cfg_attr(
    all(
        feature = "nightly",
        feature = "hardware",
        any(target_arch = "aarch64")
    ),
    feature(stdsimd)
)]
pub mod check32;

pub use check32::crc32::Crc32;
pub use check32::crc32c::Crc32C;
