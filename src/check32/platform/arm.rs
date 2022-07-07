#[cfg(all(feature = "nightly", feature = "hardware", target_arch = "aarch64"))]
use core::arch::aarch64::{__crc32b, __crc32cb, __crc32cd, __crc32cw, __crc32d, __crc32w};

#[cfg(all(feature = "hardware", target_arch = "aarch64"))]
use core::arch::aarch64::{
    uint64x2_t, vandq_u64, vdupq_n_u32, vdupq_n_u8, veorq_u64, vextq_u8, vgetq_lane_u32,
    vgetq_lane_u64, vld1q_u32, vld1q_u64, vld1q_u8, vmull_p64, vreinterpretq_u32_u64,
    vreinterpretq_u64_p128, vreinterpretq_u64_u32, vreinterpretq_u64_u8, vreinterpretq_u8_u64,
    vsetq_lane_u32,
};

#[cfg(all(feature = "hardware", target_arch = "aarch64"))]
use std::arch::asm;

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

/// This function computes the crc values based on the implementation of chromiums zlib
/// https://github.com/chromium/chromium/commit/a0771caebe87477558454cc6d793562e3afe74ac
#[target_feature(enable = "neon", enable = "aes")]
#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn compute_crc(
    prev_crc: u32,
    constants: [u64; 7],
    rev_polynomial: u64,
    mut data: &[u8],
) -> (u32, &[u8]) {
    if data.len() < 128 {
        return (prev_crc, data);
    }

    // this is safe since we already validated we have at least 128 bytes
    let mut x3 = get_simd_128(&mut data);
    let mut x2 = get_simd_128(&mut data);
    let mut x1 = get_simd_128(&mut data);
    let mut x0 = get_simd_128(&mut data);

    let prev_crc_vec = vreinterpretq_u64_u32(vsetq_lane_u32(!prev_crc, vdupq_n_u32(0), 0));

    x3 = veorq_u64(x3, prev_crc_vec);

    let k1k2 = vld1q_u64([constants[0], constants[1]].as_ptr());

    while data.len() >= 64 {
        x3 = fold_128(x3, get_simd_128(&mut data), k1k2);
        x2 = fold_128(x2, get_simd_128(&mut data), k1k2);
        x1 = fold_128(x1, get_simd_128(&mut data), k1k2);
        x0 = fold_128(x0, get_simd_128(&mut data), k1k2);
    }

    //fold into 128 bits
    let k3k4 = vld1q_u64([constants[2], constants[3]].as_ptr());
    let mut x = fold_128(x3, x2, k3k4);
    x = fold_128(x, x1, k3k4);
    x = fold_128(x, x0, k3k4);

    // fold 1*128 bits
    while data.len() >= 16 {
        x = fold_128(x, get_simd_128(&mut data), k3k4);
    }

    // fold 128 bits to 64 bits
    const MASK: [u32; 4] = [!0, 0, !0, 0];
    let mut x2 = pmull_01(x, k3k4);
    x = vreinterpretq_u64_u8(vextq_u8(vreinterpretq_u8_u64(x), vdupq_n_u8(0), 8));
    let x3 = vreinterpretq_u64_u32(vld1q_u32(MASK.as_ptr()));
    x = veorq_u64(x, x2);

    let k5k6 = vld1q_u64([constants[4], constants[5]].as_ptr());

    x2 = vreinterpretq_u64_u8(vextq_u8(vreinterpretq_u8_u64(x), vdupq_n_u8(0), 4));
    x = vandq_u64(x, x3);
    x = pmull_00(x, k5k6);
    x = veorq_u64(x, x2);

    /*
     * Barret reduce to 32-bits.
     */
    let pu = vld1q_u64([rev_polynomial, constants[6]].as_ptr());

    x2 = vandq_u64(x, x3);
    x2 = pmull_01(x2, pu);
    x2 = vandq_u64(x2, x3);
    x2 = pmull_00(x2, pu);
    x = veorq_u64(x, x2);

    (!(vgetq_lane_u32(vreinterpretq_u32_u64(x), 1)), data)
}

/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_01(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    //let mut xmm0 = b;
    let result = vmull_p64(vgetq_lane_u64(a, 0), vgetq_lane_u64(b, 1));
    vreinterpretq_u64_p128(result)
}

/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_00(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    //let mut xmm0 = b;
    let result = vmull_p64(vgetq_lane_u64(a, 0), vgetq_lane_u64(b, 0));
    vreinterpretq_u64_p128(result)
}

/*/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_11(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    //let mut xmm0 = b;
    let result = vmull_p64(vgetq_lane_u64(a, 1), vgetq_lane_u64(b, 1));
    vreinterpretq_u64_p128(result)
}*/

/*/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_01(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    let result: uint64x2_t;
    asm!(
        "pmull  {q0}.1q, {v1}.1d, {v2}.1d",
        q0 = out(vreg) result,
        v1 = in(vreg) a,
        v2 = in(vreg) vgetq_lane_u64(b, 1),
    );
    result
}*/

/*/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_00(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    let result: uint64x2_t;
    asm!(
        "pmull  {q0}.1q, {v1}.1d, {v2}.1d",
        q0 = out(vreg) result,
        v1 = in(vreg) a,
        v2 = in(vreg) b,
    );
    result
}*/

/// performing the equivalent of _mm_clmulepi64_si128(a, b, 0x00);
/// I had to use asm macro here since this was generating a lot of fmov.
/// I wasn't the only one who faced it, tikvs crc64 implementation also had the same issue
/// <(https://github.com/tikv/crc64fast/blob/master/src/pclmulqdq/aarch64.rs)>
#[target_feature(enable = "aes")]
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn pmull_11(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    let result: uint64x2_t;
    asm!(
        "pmull2  {q0}.1q, {v1}.2d, {v2}.2d",
        q0 = out(vreg) result,
        v1 = in(vreg) a,
        v2 = in(vreg) b,
    );
    result
}

/// fold 128 bits
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn fold_128(a: uint64x2_t, mut b: uint64x2_t, constant: uint64x2_t) -> uint64x2_t {
    //let mut xmm0 = b;
    let xmm1 = pmull_00(a, constant);
    let xmm2 = pmull_11(a, constant);
    b = veorq_u64(b, xmm1);
    b = veorq_u64(b, xmm2);
    b
}

/// read 128bits
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn get_simd_128(data: &mut &[u8]) -> uint64x2_t {
    let x1 = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let x2 = u64::from_le_bytes(data[8..16].try_into().unwrap());
    *data = &data[16..];
    vld1q_u64([x1, x2].as_ptr())
}
