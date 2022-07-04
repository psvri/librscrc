use crate::crc32::custom_crc32::CustomCrc32;
use crate::crc32::{Crc32Digest, UpdateFn};

#[cfg(target_arch = "x86_64")]
use crate::crc32::platform::x86::compute_crc32c_hardware_x86_64;

#[cfg(target_arch = "x86")]
use crate::crc32::platform::x86::compute_crc32c_hardware_x86;

const CRC32C_POLYNOMIAL: u32 = 0x1EDC6F41;
const CRC32C_LOOKUP_TABLE: [[u32; 256]; 16] =
    CustomCrc32::generate_lookup_table_16(CRC32C_POLYNOMIAL);

#[cfg(feature = "hardware")]
const CRC32C_POLYNOMIAL_64: u64 = 0x11EDC6F41u64;
#[cfg(feature = "hardware")]
const REVERSE_CRC32C_POLYNOMIAL_64: u64 = 0x105ec76f1u64;
#[cfg(feature = "hardware")]
const CRC32C_SIMD_CONSTANTS: [u64; 7] =
    CustomCrc32::generate_simd_reflected_constants(CRC32C_POLYNOMIAL_64);

pub struct Crc32C {
    state: u32,
    compute: UpdateFn,
}

impl Crc32C {
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

    fn compute_naive(prev_crc: u32, data: &[u8]) -> u32 {
        CustomCrc32::crc32_naive(prev_crc, CRC32C_POLYNOMIAL, data)
    }

    fn compute_lookup(prev_crc: u32, data: &[u8]) -> u32 {
        CustomCrc32::crc32_lookup(prev_crc, &CRC32C_LOOKUP_TABLE, data)
    }

    #[cfg(feature = "hardware")]
    pub fn compute_hardware(prev_crc: u32, data: &[u8]) -> u32 {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("sse4.2") {
                return compute_crc32c_hardware_x86_64(prev_crc, data);
            }
            #[cfg(target_arch = "x86")]
            if is_x86_feature_detected!("sse4.2") {
                return compute_crc32c_hardware_x86(prev_crc, data);
            }
        }

        Self::compute_lookup(prev_crc, data)
    }

    #[cfg(feature = "hardware")]
    pub fn compute_simd(mut prev_crc: u32, mut data: &[u8]) -> u32 {
        (prev_crc, data) = CustomCrc32::crc32_simd(
            prev_crc,
            CRC32C_SIMD_CONSTANTS,
            REVERSE_CRC32C_POLYNOMIAL_64,
            data,
        );
        //prev_crc
        Self::compute_hardware(prev_crc, data)
    }
}

impl Crc32Digest for Crc32C {
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

    use super::*;

    static EMPTY_DATA: &[u8; 0] = b"";
    static SMALL_DATA_1: &[u8; 9] = b"123456789";
    static SMALL_DATA_2: &[u8; 11] = b"hello-world";
    static LARGE_DATA_1: &[u8; 144] = include_bytes!("../../sample_files/test_data.txt");
    static LARGE_DATA_2: &[u8; 241] = include_bytes!("../../sample_files/test_data_odd_size.txt");

    const EMPTY_DATA_CRC32: u32 = 0;
    const SMALL_DATA_1_CRC32: u32 = 0xE3069283;
    const SMALL_DATA_2_CRC32: u32 = 4099351003;
    const LARGE_DATA_1_CRC32: u32 = 0xD1FABDC4;
    const LARGE_DATA_2_CRC32: u32 = 0xC3FE94BC;

    fn test_naive(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32C::new();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc32c_naive() {
        test_naive(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_naive(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_naive(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_naive(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_naive(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }

    fn test_lookup(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32C::new_lookup();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc32c_lookup() {
        test_lookup(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_lookup(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_lookup(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_lookup(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_lookup(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }

    #[cfg(feature = "hardware")]
    fn test_hardware(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32C::new_hardware();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    #[cfg(feature = "hardware")]
    fn test_crc32c_hardware() {
        test_hardware(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_hardware(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_hardware(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_hardware(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_hardware(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }

    #[cfg(feature = "hardware")]
    fn test_simd(data: &[u8], expected_crc: u32) {
        let mut crc = Crc32C::new_simd();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    #[cfg(feature = "hardware")]
    fn test_crc32c_simd() {
        test_simd(EMPTY_DATA, EMPTY_DATA_CRC32);
        test_simd(SMALL_DATA_1, SMALL_DATA_1_CRC32);
        test_simd(SMALL_DATA_2, SMALL_DATA_2_CRC32);
        test_simd(LARGE_DATA_1, LARGE_DATA_1_CRC32);
        test_simd(LARGE_DATA_2, LARGE_DATA_2_CRC32);
    }
}
