#![no_main]
use libfuzzer_sys::fuzz_target;
use librscrc::prelude::*;
use std::convert::TryInto;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let polynomial = u32::from_le_bytes(data[..4].try_into().unwrap());
        let data = &data[4..];
        let mut naive = CustomCrc32::new_naive(polynomial);
        let mut lookup = CustomCrc32::new_lookup(polynomial);
        // there is no hardware implementation for a custom polynomial
        // let mut hardware = CustomCrc32::new_hardware(polynomial);
        let mut simd = CustomCrc32::new_simd(polynomial.into());
        naive.update(data);
        lookup.update(data);
        simd.update(data);
        let naive_result = naive.digest();
        assert_eq!(naive_result, lookup.digest());
        assert_eq!(naive_result, simd.digest());
    }
});
