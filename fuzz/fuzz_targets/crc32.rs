#![no_main]
use libfuzzer_sys::fuzz_target;
use librscrc::prelude::*;

fuzz_target!(|data: &[u8]| {
    let mut naive = Crc32::new_naive();
    let mut lookup = Crc32::new_lookup();
    let mut hardware = Crc32::new_hardware();
    let mut simd = Crc32::new_simd();
    naive.update(data);
    lookup.update(data);
    hardware.update(data);
    simd.update(data);
    let naive_result = naive.digest();
    assert_eq!(naive_result, lookup.digest());
    assert_eq!(naive_result, hardware.digest());
    assert_eq!(naive_result, simd.digest());
});
