type UpdateFn = fn(u32, &[u8]) -> u32;

pub trait Crc32Digest {
    fn update(&mut self, data: &[u8]);
    fn digest(&self) -> u32;
}

pub mod crc32;
pub mod crc32c;
pub mod custom_crc32;

#[cfg(feature = "hardware")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod platform;
