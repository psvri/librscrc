#[cfg(all(feature = "nightly", feature = "hardware", target_arch = "aarch64"))]
use core::arch::aarch64::{__crc32b, __crc32cb, __crc32cd, __crc32cw, __crc32d, __crc32w};

#[cfg(all(feature = "nightly", feature = "hardware", target_arch = "aarch64"))]
#[target_feature(enable = "crc")]
pub(crate) unsafe fn compute_crc32_hardware_aarch64(prev_crc: u32, data: &[u8]) -> u32 {
    let mut crc = !prev_crc;
    let mut chunk_iter = data.chunks_exact(8);

    for chunk in chunk_iter.by_ref() {
        crc = __crc32d(crc, u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    chunk_iter = chunk_iter.remainder().chunks_exact(4);

    for chunk in chunk_iter.by_ref() {
        crc = __crc32w(crc, u32::from_le_bytes(chunk.try_into().unwrap()));
    }

    for reminder in chunk_iter.remainder() {
        crc = __crc32b(crc, *reminder);
    }

    !crc
}

#[cfg(all(feature = "nightly", feature = "hardware", target_arch = "aarch64"))]
#[target_feature(enable = "crc")]
pub(crate) unsafe fn compute_crc32c_hardware_aarch64(prev_crc: u32, data: &[u8]) -> u32 {
    let mut crc = !prev_crc;
    let mut chunk_iter = data.chunks_exact(8);

    for chunk in chunk_iter.by_ref() {
        crc = __crc32cd(crc, u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    chunk_iter = chunk_iter.remainder().chunks_exact(4);

    for chunk in chunk_iter.by_ref() {
        crc = __crc32cw(crc, u32::from_le_bytes(chunk.try_into().unwrap()));
    }

    for reminder in chunk_iter.remainder() {
        crc = __crc32cb(crc, *reminder);
    }

    !crc
}
