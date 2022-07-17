pub(crate) mod crc64ecma;
mod custom_crc64;
mod crc64iso;

type UpdateFn = fn(u64, &[u8]) -> u64;

pub trait Crc64Digest {
    /// Update digest with data
    fn update(&mut self, data: &[u8]);

    /// Returns crc32 digest
    fn digest(&self) -> u64;
}

pub use crc64ecma::Crc64ECMA;