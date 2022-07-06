#[cfg(feature = "hardware")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::check32::platform::x86::compute_crc;

#[cfg(feature = "hardware")]
#[cfg(target_arch = "aarch64")]
use crate::check32::platform::arm::compute_crc;

#[cfg(feature = "hardware")]
#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

pub struct CustomCrc32 {}

impl CustomCrc32 {
    pub const fn crc32_naive(prev_crc: u32, polynomial: u32, data: &[u8]) -> u32 {
        let mut crc = !prev_crc;
        let polynomial = polynomial.reverse_bits();
        let mut i = 0;
        let mut j = 0;
        while i < data.len() {
            crc ^= data[i] as u32;

            while j < 8 {
                if crc & 1u32 == 1u32 {
                    crc = crc >> 1 ^ polynomial;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            j = 0;
            i += 1;
        }

        !crc
    }

    pub fn crc32_lookup(prev_crc: u32, lookup_table: &[[u32; 256]; 16], mut data: &[u8]) -> u32 {
        let mut crc: u32 = !prev_crc;

        while data.len() >= 16 {
            //crc ^= u32::from_le_bytes(data[..4].try_into().unwrap());
            crc = lookup_table[0][data[15] as usize]
                ^ lookup_table[1][data[14] as usize]
                ^ lookup_table[2][data[13] as usize]
                ^ lookup_table[3][data[12] as usize]
                ^ lookup_table[4][data[11] as usize]
                ^ lookup_table[5][data[10] as usize]
                ^ lookup_table[6][data[9] as usize]
                ^ lookup_table[7][data[8] as usize]
                ^ lookup_table[8][data[7] as usize]
                ^ lookup_table[9][data[6] as usize]
                ^ lookup_table[10][data[5] as usize]
                ^ lookup_table[11][data[4] as usize]
                ^ lookup_table[12][data[3] as usize ^ ((crc >> 24) & 0xFF) as usize]
                ^ lookup_table[13][data[2] as usize ^ ((crc >> 16) & 0xFF) as usize]
                ^ lookup_table[14][data[1] as usize ^ ((crc >> 8) & 0xFF) as usize]
                ^ lookup_table[15][data[0] as usize ^ ((crc) & 0xFF) as usize];
            data = &data[16..];
        }
        for &b in data {
            crc = lookup_table[0][((crc as u8) ^ b) as usize] ^ (crc >> 8);
        }

        !crc
    }

    #[cfg(feature = "hardware")]
    pub(super) fn crc32_simd(
        mut prev_crc: u32,
        constants: [u64; 7],
        rev_polynomial: u64,
        mut data: &[u8],
    ) -> (u32, &[u8]) {
        unsafe {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if is_x86_feature_detected!("sse4.2") && is_x86_feature_detected!("pclmulqdq") {
                (prev_crc, data) = compute_crc(prev_crc, constants, rev_polynomial, data);
            }
            #[cfg(target_arch = "aarch64")]
            if is_aarch64_feature_detected!("neon") && is_aarch64_feature_detected!("aes") {
                (prev_crc, data) = compute_crc(prev_crc, constants, rev_polynomial, data);
            }
        }
        (prev_crc, data)
    }

    pub(super) const fn generate_lookup_table_16(polynomial: u32) -> [[u32; 256]; 16] {
        let mut table = [[0; 256]; 16];

        table[0] = Self::generate_lookup_table(polynomial);
        let mut length = 0;
        let mut j = 1;

        while length < 256 {
            let mut crc = table[0][length];
            while j < 16 {
                crc = (crc >> 8) ^ table[0][crc as u8 as usize];
                table[j][length] = crc;
                j += 1;
            }
            j = 1;
            length += 1;
        }

        table
    }

    pub(super) const fn generate_lookup_table(mut polynomial: u32) -> [u32; 256] {
        let mut table = [0; 256];
        polynomial = polynomial.reverse_bits();
        let mut length = 0;
        let mut crc;
        let mut j = 0;

        while length < 256 {
            crc = length;
            while j < 8 {
                if crc & 1u32 == 1u32 {
                    crc = (crc >> 1) ^ polynomial;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[length as usize] = crc;
            j = 0;
            length += 1;
        }

        table
    }

    pub(super) const fn generate_simd_constants(polynomial: u64) -> [u64; 7] {
        let x32 = Self::division(0x100000000, polynomial).1;
        let x64 = Self::division(Self::carry_less_mul(x32, x32), polynomial).1;
        let x96 = Self::division(Self::carry_less_mul(x64, x32), polynomial).1;
        let x128 = Self::division(Self::carry_less_mul(x64, x64), polynomial).1;
        let x192 = Self::division(Self::carry_less_mul(x128, x64), polynomial).1;
        let x256 = Self::division(Self::carry_less_mul(x128, x128), polynomial).1;
        let x512 = Self::division(Self::carry_less_mul(x256, x256), polynomial).1;
        let x576 = Self::division(Self::carry_less_mul(x512, x64), polynomial).1;

        let u = 0x1_0000_0000 | Self::division(polynomial << 32, polynomial).0;

        [x576, x512, x192, x128, x96, x64, u]
    }

    pub(super) const fn generate_simd_reflected_constants(polynomial: u64) -> [u64; 7] {
        // x32
        let x32 = Self::division(1 << 32, polynomial).1;

        //x64
        let x64 = Self::division(Self::carry_less_mul(x32, x32), polynomial).1;

        //x(128 - 32) and x(128 + 32)
        let x96 = Self::division(Self::carry_less_mul(x64, x32), polynomial).1;
        let x160 = Self::division(Self::carry_less_mul(x96, x64), polynomial).1;

        //x(4*128 - 32) and x(4*128 + 32)
        let x224 = Self::division(Self::carry_less_mul(x160, x64), polynomial).1;
        let x256 = Self::division(Self::carry_less_mul(x224, x32), polynomial).1;
        let x480 = Self::division(Self::carry_less_mul(x256, x224), polynomial).1;
        let x544 = Self::division(Self::carry_less_mul(x480, x64), polynomial).1;

        let u = 0x1_0000_0000 | Self::division(polynomial << 32, polynomial).0;

        // [k1', k2', k3', k4', k5', k6', u']
        let mut constants = [0; 7];
        constants[0] = Self::reverse_constant(x544);
        constants[1] = Self::reverse_constant(x480);
        constants[2] = Self::reverse_constant(x160);
        constants[3] = Self::reverse_constant(x96);
        constants[4] = Self::reverse_constant(x64);
        constants[5] = Self::reverse_constant(x32);

        constants[6] = Self::reverse_constant(u);

        constants
    }

    const fn reverse_constant(mut constant: u64) -> u64 {
        let mut reversed_constant = 0;

        let mut count = 0;

        while count < 33 {
            reversed_constant = (reversed_constant << 1) ^ (constant & 1);
            constant >>= 1;
            count += 1;
        }
        reversed_constant
    }

    const fn division(dividend: u64, polynomial: u64) -> (u64, u64) {
        let mut remainder = dividend;
        let mut quotient = 0;
        let mut count = 0;
        while count < 32 {
            let msb = remainder >> 63;
            quotient = (quotient << 1) ^ msb;
            remainder = (remainder << 1) ^ (msb * (polynomial << 32));
            count += 1;
        }
        (quotient, remainder >> 32)
    }

    const fn carry_less_mul(a: u64, b: u64) -> u64 {
        let mut result = 0;
        let mut count = 0;
        while count < 32 {
            if (b >> count) & 1 == 1 {
                result ^= a << count
            }
            count += 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLYNOMIAL: u64 = 0x104C11DB7u64;

    #[test]
    fn test_crc32() {
        assert_eq!(CustomCrc32::crc32_naive(0, POLYNOMIAL as u32, b""), 0);
        assert_eq!(
            CustomCrc32::crc32_naive(0, POLYNOMIAL as u32, b"123456789"),
            0xCBF43926
        );
        assert_eq!(
            CustomCrc32::crc32_naive(0, POLYNOMIAL as u32, b"hello-world"),
            2983461467
        );
    }

    #[test]
    fn test_simd_constant() {
        let constants = CustomCrc32::generate_simd_constants(POLYNOMIAL);

        assert_eq!(constants[0], 0x8833794C);
        assert_eq!(constants[1], 0xE6228B11);
        assert_eq!(constants[2], 0xC5B9CD4C);
        assert_eq!(constants[3], 0xE8A45605);
        assert_eq!(constants[4], 0xF200AA66);
        assert_eq!(constants[5], 0x490D678D);

        assert_eq!(constants[6], 0x104D101DF);
    }

    #[test]
    fn test_simd_reflected_constant() {
        let constants = CustomCrc32::generate_simd_reflected_constants(POLYNOMIAL);

        assert_eq!(constants[0], 0x154442bd4);
        assert_eq!(constants[1], 0x1c6e41596);
        assert_eq!(constants[2], 0x1751997d0);
        assert_eq!(constants[3], 0x0ccaa009e);
        assert_eq!(constants[4], 0x163cd6124);
        assert_eq!(constants[5], 0x1db710640);
        assert_eq!(constants[6], 0x1F7011641);
    }
}
