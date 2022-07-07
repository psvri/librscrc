type UpdateFn = fn(u32, &[u8]) -> u32;

pub trait Crc32Digest {
    /// Update digest with data
    fn update(&mut self, data: &[u8]);

    /// Returns crc32 digest
    fn digest(&self) -> u32;
}

mod crc32;
mod crc32c;
mod custom_crc32;

#[cfg(any(
all(feature = "hardware", any(target_arch = "x86", target_arch = "x86_64")),
all(feature = "hardware", any(target_arch = "aarch64"))
))]
mod platform;

pub use crc32::Crc32;
pub use crc32c::Crc32C;
pub use custom_crc32::CustomCrc32;

