use crate::crc32::custom_crc32::CustomCrc32;
use crate::crc32::Crc32Digest;

use super::UpdateFn;

const CRC32_POLYNOMIAL: u32 = 0x04C11DB7;
const CRC32_LOOKUP_TABLE: [[u32; 256]; 16] =
    CustomCrc32::generate_lookup_table_16(CRC32_POLYNOMIAL);

#[cfg(feature = "hardware")]
const CRC32_POLYNOMIAL_64: u64 = 0x104C11DB7u64;
#[cfg(feature = "hardware")]
const REVERSE_CRC32_POLYNOMIAL_64: u64 = 0x1DB710641u64;
#[cfg(feature = "hardware")]
const CRC32_SIMD_CONSTANTS: [u64; 7] =
    CustomCrc32::generate_simd_reflected_constants(CRC32_POLYNOMIAL_64);

pub struct Crc32 {
    state: u32,
    compute: UpdateFn,
}

impl Crc32 {
    pub fn new() -> Self {
        Self {
            state: 0,
            compute: Self::compute_naive,
        }
    }

    pub fn new_lookup() -> Self {
        Self {
            state: 0,
            compute: Self::compute_lookup,
        }
    }

    #[cfg(feature = "hardware")]
    pub fn new_hardware() -> Self {
        Self {
            state: 0,
            compute: Self::compute_hardware,
        }
    }

    #[cfg(feature = "hardware")]
    pub fn new_simd() -> Self {
        Self {
            state: 0,
            compute: Self::compute_simd,
        }
    }

    fn compute_lookup(prev_crc: u32, data: &[u8]) -> u32 {
        CustomCrc32::crc32_lookup(prev_crc, &CRC32_LOOKUP_TABLE, data)
    }

    fn compute_naive(prev_crc: u32, data: &[u8]) -> u32 {
        CustomCrc32::crc32_naive(prev_crc, CRC32_POLYNOMIAL, data)
    }

    #[cfg(feature = "hardware")]
    fn compute_hardware(prev_crc: u32, data: &[u8]) -> u32 {
        Self::compute_lookup(prev_crc, data)
    }

    #[cfg(feature = "hardware")]
    fn compute_simd(mut prev_crc: u32, mut data: &[u8]) -> u32 {
        (prev_crc, data) = CustomCrc32::crc32_simd(
            prev_crc,
            CRC32_SIMD_CONSTANTS,
            REVERSE_CRC32_POLYNOMIAL_64,
            data,
        );
        //prev_crc
        Self::compute_lookup(prev_crc, data)
    }
}

impl Crc32Digest for Crc32 {
    fn update(&mut self, data: &[u8]) {
        self.state = (self.compute)(self.state, data);
    }

    fn digest(&self) -> u32 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static EMPTY_DATA: &[u8; 0] = b"";
    static SMALL_DATA_1: &[u8; 9] = b"123456789";
    static SMALL_DATA_2: &[u8; 11] = b"hello-world";
    static LARGE_DATA_1: &[u8; 144] = include_bytes!("../../sample_files/test_data.txt");
    static LARGE_DATA_2: &[u8; 241] = include_bytes!("../../sample_files/test_data_odd_size.txt");

    const EMPTY_DATA_CRC32: u32 = 0;
    const SMALL_DATA_1_CRC32: u32 = 0xCBF43926;
    const SMALL_DATA_2_CRC32: u32 = 2983461467;
    const LARGE_DATA_1_CRC32: u32 = 0x4BF7E46F;
    const LARGE_DATA_2_CRC32: u32 = 0x7EC1A494;

    fn test_naive(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32::new();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc32_naive() {
        test_naive(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_naive(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_naive(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_naive(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_naive(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }

    fn test_lookup(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32::new_lookup();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc32_lookup() {
        test_lookup(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_lookup(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_lookup(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_lookup(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_lookup(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }

    fn test_simd(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32::new_simd();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    #[cfg(feature = "hardware")]
    fn test_crc32_simd() {
        test_simd(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_simd(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_simd(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_simd(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_simd(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }
}
