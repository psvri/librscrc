pub struct CustomCrc64 {}

impl CustomCrc64 {
    pub(crate) fn crc64_lookup(
        prev_crc: u64,
        lookup_table: &[[u64; 256]; 16],
        mut data: &[u8],
    ) -> u64 {
        let mut crc: u64 = !prev_crc;

        while data.len() >= 16 {
            crc = lookup_table[0][data[15] as usize]
                ^ lookup_table[1][data[14] as usize]
                ^ lookup_table[2][data[13] as usize]
                ^ lookup_table[3][data[12] as usize]
                ^ lookup_table[4][data[11] as usize]
                ^ lookup_table[5][data[10] as usize]
                ^ lookup_table[6][data[9] as usize]
                ^ lookup_table[7][data[8] as usize]
                ^ lookup_table[8][data[7] as usize ^ ((crc >> 56) & 0xFF) as usize]
                ^ lookup_table[9][data[6] as usize ^ ((crc >> 48) & 0xFF) as usize]
                ^ lookup_table[10][data[5] as usize ^ ((crc >> 40) & 0xFF) as usize]
                ^ lookup_table[11][data[4] as usize ^ ((crc >> 32) & 0xFF) as usize]
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

    pub(super) const fn generate_lookup_table_16(polynomial: u64) -> [[u64; 256]; 16] {
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

    pub(super) const fn generate_lookup_table(mut polynomial: u64) -> [u64; 256] {
        let mut table = [0; 256];
        polynomial = polynomial.reverse_bits();
        let mut length = 0;
        let mut crc;
        let mut j = 0;

        while length < 256 {
            crc = length;
            while j < 8 {
                if crc & 1u64 == 1 {
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

    pub(crate) fn crc64_naive(prev_crc: u64, polynomial: u64, data: &[u8]) -> u64 {
        let mut crc = !prev_crc;
        let polynomial = polynomial.reverse_bits();
        let mut i = 0;
        let mut j = 0;

        while i < data.len() {
            crc ^= data[i] as u64;

            while j < 8 {
                if crc & 1u64 == 1 {
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

    pub(super) const fn generate_simd_reflected_constants(mut polynomial: u64) -> [u64; 7] {
        let polynomial = (polynomial as u128) & 1 << 64;
        // x32
        let x32 = Self::division(1 << 32, polynomial).1;

        //x64
        let x64 = Self::division(Self::carry_less_mul(x32, x32), polynomial).1;

        //x(128) and x(192)
        let x128 = Self::division(Self::carry_less_mul(x64, x64), polynomial).1;
        let x192 = Self::division(Self::carry_less_mul(x64, x32), polynomial).1;
        let x96 = Self::division(Self::carry_less_mul(x64, x32), polynomial).1;
        let x160 = Self::division(Self::carry_less_mul(x96, x64), polynomial).1;

        //x(4*128 - 32) and x(4*128 + 32)
        let x224 = Self::division(Self::carry_less_mul(x160, x64), polynomial).1;
        let x256 = Self::division(Self::carry_less_mul(x224, x32), polynomial).1;
        let x480 = Self::division(Self::carry_less_mul(x256, x224), polynomial).1;
        let x544 = Self::division(Self::carry_less_mul(x480, x64), polynomial).1;

        let u = 0x1_0000_0000_0000_0000 | Self::division(polynomial << 64, polynomial).0;

        // [k1', k2', k3', k4', k5', k6', u']
        let mut constants = [0; 7];
        constants[0] = Self::reverse_constant(x544);
        constants[1] = Self::reverse_constant(x480);
        constants[2] = Self::reverse_constant(x160);
        constants[3] = Self::reverse_constant(x96);
        constants[4] = Self::reverse_constant(x192);
        constants[5] = Self::reverse_constant(x128);

        constants[6] = Self::reverse_constant(u);

        constants
    }

    const fn reverse_constant(mut constant: u128) -> u64 {
        let mut reversed_constant = 0;

        let mut count = 0;

        while count < 65 {
            reversed_constant = (reversed_constant << 1) ^ (constant & 1);
            constant >>= 1;
            count += 1;
        }
        reversed_constant as u64
    }

    pub const fn division(dividend: u128, polynomial: u128) -> (u128, u128) {
        let mut remainder = dividend;
        let mut quotient = 0;
        let mut count = 0;
        while count < 64 {
            let msb = remainder >> 127;
            quotient = (quotient << 1) ^ msb;
            remainder = (remainder << 1) ^ (msb * (polynomial << 64));
            count += 1;
        }
        (quotient, (remainder >> 64))
    }

    pub const fn carry_less_mul(a: u128, b: u128) -> u128 {
        let mut result = 0;
        let mut count = 0;
        while count < 64 {
            if (b >> count) & 1 == 1 {
                result ^= (a as u128) << count
            }
            count += 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::CustomCrc64;

    const POLYNOMIAL: u64 = 0x42F0E1EBA9EA3693;

    #[test]
    fn test_carry_less_mul() {
        assert_eq!(
            CustomCrc64::carry_less_mul(0x5a2d_8244_0f1e_3e50, 0xcae9_00d5_fed9_262f),
            0x39ca_c5ca_fc66_6bf3_25bc_9dd4_c0f3_6330,
        )
    }
    
    fn test_simd_constants() {
        let constants = CustomCrc64::generate_simd_reflected_constants(POLYNOMIAL);

        println!("{:0x}", constants[6]);
        println!("{:0x}", 0xdabe_95af_c787_5f40u64);
        assert_eq!(constants[6], 0xdabe_95af_c787_5f40)
    }
}