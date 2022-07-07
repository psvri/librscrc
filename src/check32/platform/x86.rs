#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_clmulepi64_si128, _mm_crc32_u32, _mm_crc32_u64, _mm_crc32_u8,
    _mm_cvtsi32_si128, _mm_extract_epi32, _mm_set_epi64x, _mm_setr_epi32, _mm_srli_si128,
    _mm_xor_si128,
};

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_and_si128, _mm_clmulepi64_si128, _mm_crc32_u32, _mm_crc32_u8, _mm_cvtsi32_si128,
    _mm_extract_epi32, _mm_set_epi64x, _mm_setr_epi32, _mm_srli_si128, _mm_xor_si128,
};

#[target_feature(enable = "sse4.2")]
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn compute_crc32c_hardware_x86_64(prev_crc: u32, data: &[u8]) -> u32 {
    let mut crc = (!prev_crc) as u64;
    let mut chunk_iter = data.chunks_exact(8);

    for chunk in chunk_iter.by_ref() {
        crc = _mm_crc32_u64(crc, u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    let mut crc = crc as u32;
    chunk_iter = chunk_iter.remainder().chunks_exact(4);

    for chunk in chunk_iter.by_ref() {
        crc = _mm_crc32_u32(crc, u32::from_le_bytes(chunk.try_into().unwrap()));
    }
    for reminder in chunk_iter.remainder() {
        crc = _mm_crc32_u8(crc, *reminder);
    }

    !crc
}

#[target_feature(enable = "sse4.2")]
#[cfg(target_arch = "x86")]
pub(crate) unsafe fn compute_crc32c_hardware_x86(prev_crc: u32, data: &[u8]) -> u32 {
    let mut crc = !prev_crc;
    let mut chunk_iter = data.chunks_exact(4);

    for chunk in chunk_iter.by_ref() {
        crc = _mm_crc32_u32(crc, u32::from_le_bytes(chunk.try_into().unwrap()));
    }
    for reminder in chunk_iter.remainder() {
        crc = _mm_crc32_u8(crc, *reminder);
    }

    !crc
}

/// This function computes the crc values based on the implementation on chromiums zlib code present
/// in <https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/zlib/crc32_simd.c>
#[target_feature(enable = "sse4.2", enable = "pclmulqdq", enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn compute_crc(
    prev_crc: u32,
    constants: [u64; 7],
    rev_polynomial: u64,
    mut data: &[u8],
) -> (u32, &[u8]) {
    if data.len() < 128 {
        return (prev_crc, data);
    }
    //dbg!("using simd implementation");

    // this is safe since we already validated we have at least 128 bytes
    let mut x3 = get_simd_128(&mut data);
    let mut x2 = get_simd_128(&mut data);
    let mut x1 = get_simd_128(&mut data);
    let mut x0 = get_simd_128(&mut data);

    x3 = _mm_xor_si128(x3, _mm_cvtsi32_si128(!prev_crc as i32));

    let k1k2 = _mm_set_epi64x(constants[1] as i64, constants[0] as i64);
    // fold 4*128 bits
    while data.len() >= 64 {
        x3 = fold_128(x3, get_simd_128(&mut data), k1k2);
        x2 = fold_128(x2, get_simd_128(&mut data), k1k2);
        x1 = fold_128(x1, get_simd_128(&mut data), k1k2);
        x0 = fold_128(x0, get_simd_128(&mut data), k1k2);
    }

    //fold into 128 bits
    let k3k4 = _mm_set_epi64x(constants[3] as i64, constants[2] as i64);
    let mut x = fold_128(x3, x2, k3k4);
    x = fold_128(x, x1, k3k4);
    x = fold_128(x, x0, k3k4);

    // fold 1*128 bits
    while data.len() >= 16 {
        x = fold_128(x, get_simd_128(&mut data), k3k4);
    }

    // fold 128 bits to 64 bits
    let mut x2 = _mm_clmulepi64_si128(x, k3k4, 0x10);
    let x3 = _mm_setr_epi32(!0, 0, !0, 0);
    x = _mm_srli_si128(x, 8);
    x = _mm_xor_si128(x, x2);

    let k5k6 = _mm_set_epi64x(constants[5] as i64, constants[4] as i64);

    x2 = _mm_srli_si128(x, 4);
    x = _mm_and_si128(x, x3);
    x = _mm_clmulepi64_si128(x, k5k6, 0x00);
    x = _mm_xor_si128(x, x2);

    // Barret reduce to 32-bits
    let pu = _mm_set_epi64x(constants[6] as i64, rev_polynomial as i64);

    x2 = _mm_and_si128(x, x3);
    x2 = _mm_clmulepi64_si128(x2, pu, 0x10);

    x2 = _mm_and_si128(x2, x3);
    x2 = _mm_clmulepi64_si128(x2, pu, 0x00);
    x = _mm_xor_si128(x, x2);

    (!(_mm_extract_epi32(x, 1) as u32), data)
}

#[target_feature(enable = "pclmulqdq,sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn fold_128(a: __m128i, mut b: __m128i, constant: __m128i) -> __m128i {
    //let mut xmm0 = b;
    let xmm1 = _mm_clmulepi64_si128(a, constant, 0x00);
    let xmm2 = _mm_clmulepi64_si128(a, constant, 0x11);
    b = _mm_xor_si128(b, xmm1);
    b = _mm_xor_si128(b, xmm2);
    b
}

#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn get_simd_128(data: &mut &[u8]) -> __m128i {
    let x1 = i64::from_le_bytes(data[0..8].try_into().unwrap());
    let x2 = i64::from_le_bytes(data[8..16].try_into().unwrap());
    *data = &data[16..];
    _mm_set_epi64x(x2, x1)
}
