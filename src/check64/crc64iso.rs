use crate::check64::custom_crc64::CustomCrc64;
use crate::check64::{Crc64Digest, UpdateFn};

const CRC64_POLYNOMIAL: u64 = 0x000000000000001B;
const CRC64_LOOKUP_TABLE: [[u64; 256]; 16] =
    CustomCrc64::generate_lookup_table_16(CRC64_POLYNOMIAL);

#[cfg(feature = "hardware")]
const CRC64_POLYNOMIAL_REV: u64 = 0xD800000000000000;

pub struct Crc64ISO {
    state: u64,
    compute: UpdateFn,
}

impl Crc64ISO {
    /// Creates a new `Crc64` using naive approach
    pub fn new_naive() -> Self {
        Self {
            state: 0,
            compute: Self::compute_naive,
        }
    }

    /// Creates a new `Crc64` using a table lookup approach
    pub fn new_lookup() -> Self {
        Self {
            state: 0,
            compute: Self::compute_lookup,
        }
    }

    fn compute_naive(prev_crc: u64, data: &[u8]) -> u64 {
        CustomCrc64::crc64_naive(prev_crc, CRC64_POLYNOMIAL, data)
    }

    fn compute_lookup(prev_crc: u64, data: &[u8]) -> u64 {
        CustomCrc64::crc64_lookup(prev_crc, &CRC64_LOOKUP_TABLE, data)
    }
}

impl Crc64Digest for Crc64ISO {
    fn update(&mut self, data: &[u8]) {
        self.state = (self.compute)(self.state, data);
    }

    fn digest(&self) -> u64 {
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

    const EMPTY_DATA_CRC64: u64 = 0;
    const SMALL_DATA_1_CRC64: u64 = 0xB90956C775A41001;
    const SMALL_DATA_2_CRC64: u64 = 0xB9CF3F572AD35C3E;
    const LARGE_DATA_1_CRC64: u64 = 0x36CFFD7478289526;
    const LARGE_DATA_2_CRC64: u64 = 0xC103BAD1BE6AF43A;

    fn test_naive(data: &[u8], expected_crc: u64) {
        let mut crc = Crc64ISO::new_naive();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc64iso_naive() {
        test_naive(EMPTY_DATA, EMPTY_DATA_CRC64);
        test_naive(SMALL_DATA_1, SMALL_DATA_1_CRC64);
        test_naive(SMALL_DATA_2, SMALL_DATA_2_CRC64);
        test_naive(LARGE_DATA_1, LARGE_DATA_1_CRC64);
        test_naive(LARGE_DATA_2, LARGE_DATA_2_CRC64);
    }

    fn test_lookup(data: &[u8], expected_crc: u64) {
        let mut crc = Crc64ISO::new_lookup();
        crc.update(data);
        assert_eq!(crc.digest(), expected_crc);
    }

    #[test]
    fn test_crc64iso_lookup() {
        test_lookup(EMPTY_DATA, EMPTY_DATA_CRC64);
        test_lookup(SMALL_DATA_1, SMALL_DATA_1_CRC64);
        test_lookup(SMALL_DATA_2, SMALL_DATA_2_CRC64);
        test_lookup(LARGE_DATA_1, LARGE_DATA_1_CRC64);
        test_lookup(LARGE_DATA_2, LARGE_DATA_2_CRC64);
    }
}