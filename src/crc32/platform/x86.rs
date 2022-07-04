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
/// in https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/zlib/crc32_simd.c
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

/*#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn get_simd_128(a: &mut &[u8]) -> __m128i {
    //debug_assert!(a.len() >= 16);
    let r = _mm_loadu_si128(a.as_ptr() as *const __m128i);
    *a = &a[16..];
    return r;
}*/

/*#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;

const K1: i64 = 0x154442bd4;
const K2: i64 = 0x1c6e41596;
const K3: i64 = 0x1751997d0;
const K4: i64 = 0x0ccaa009e;
const K5: i64 = 0x163cd6124;
const K6: i64 = 0x1db710640;

const P_X: i64 = 0x1DB710641;
const U_PRIME: i64 = 0x1F7011641;

unsafe fn debug(s: &str, a: arch::__m128i) -> arch::__m128i {
    if false {
        union A {
            a: arch::__m128i,
            b: [u8; 16],
        }
        let x = A { a }.b;
        print!(" {:20} | ", s);
        for x in x.iter() {
            print!("{:02x} ", x);
        }
        println!();
    }
    return a;
}

#[target_feature(enable = "pclmulqdq", enable = "sse2", enable = "sse4.1")]
unsafe fn calculate(crc: u32, mut data: &[u8]) -> u32 {
    // In theory we can accelerate smaller chunks too, but for now just rely on
    // the fallback implementation as it's too much hassle and doesn't seem too
    // beneficial.
    if data.len() < 128 {
        return crc;
    }

    // Step 1: fold by 4 loop
    let mut x3 = get(&mut data);
    let mut x2 = get(&mut data);
    let mut x1 = get(&mut data);
    let mut x0 = get(&mut data);

    // fold in our initial value, part of the incremental crc checksum
    x3 = arch::_mm_xor_si128(x3, arch::_mm_cvtsi32_si128(!crc as i32));

    let k1k2 = arch::_mm_set_epi64x(K2, K1);
    println!("others {:?}", k1k2);

    while data.len() >= 64 {
        x3 = reduce128(x3, get(&mut data), k1k2);
        x2 = reduce128(x2, get(&mut data), k1k2);
        x1 = reduce128(x1, get(&mut data), k1k2);
        x0 = reduce128(x0, get(&mut data), k1k2);
        break;
    }

    let k3k4 = arch::_mm_set_epi64x(K4, K3);
    let mut x = reduce128(x3, x2, k3k4);
    x = reduce128(x, x1, k3k4);
    x = reduce128(x, x0, k3k4);

    println!("others {:?}", x);

    // Step 2: fold by 1 loop
    while data.len() >= 16 {
        x = reduce128(x, get(&mut data), k3k4);
    }

    println!("others {:?}", x);

    debug("128 > 64 init", x);

    // Perform step 3, reduction from 128 bits to 64 bits. This is
    // significantly different from the paper and basically doesn't follow it
    // at all. It's not really clear why, but implementations of this algorithm
    // in Chrome/Linux diverge in the same way. It is beyond me why this is
    // different than the paper, maybe the paper has like errata or something?
    // Unclear.
    //
    // It's also not clear to me what's actually happening here and/or why, but
    // algebraically what's happening is:
    //
    // x = (x[0:63] • K4) ^ x[64:127]           // 96 bit result
    // x = ((x[0:31] as u64) • K5) ^ x[32:95]   // 64 bit result
    //
    // It's... not clear to me what's going on here. The paper itself is pretty
    // vague on this part but definitely uses different constants at least.
    // It's not clear to me, reading the paper, where the xor operations are
    // happening or why things are shifting around. This implementation...
    // appears to work though!
    drop(K6);
    let x = arch::_mm_xor_si128(
        arch::_mm_clmulepi64_si128(x, k3k4, 0x10),
        arch::_mm_srli_si128(x, 8),
    );
    println!("others {:?}", x);
    let x = arch::_mm_xor_si128(
        arch::_mm_clmulepi64_si128(
            arch::_mm_and_si128(x, arch::_mm_set_epi32(0, 0, 0, !0)),
            arch::_mm_set_epi64x(0, K5),
            0x00,
        ),
        arch::_mm_srli_si128(x, 4),
    );
    println!("others {:?}", x);
    debug("128 > 64 xx", x);

    // Perform a Barrett reduction from our now 64 bits to 32 bits. The
    // algorithm for this is described at the end of the paper, and note that
    // this also implements the "bit reflected input" variant.
    let pu = arch::_mm_set_epi64x(U_PRIME, P_X);
    println!("others pu {:?}", pu);

    // T1(x) = ⌊(R(x) % x^32)⌋ • μ
    let t1 = arch::_mm_clmulepi64_si128(
        arch::_mm_and_si128(x, arch::_mm_set_epi32(0, 0, 0, !0)),
        pu,
        0x10,
    );

    // T2(x) = ⌊(T1(x) % x^32)⌋ • P(x)
    let t2 = arch::_mm_clmulepi64_si128(
        arch::_mm_and_si128(t1, arch::_mm_set_epi32(0, 0, 0, !0)),
        pu,
        0x00,
    );
    // We're doing the bit-reflected variant, so get the upper 32-bits of the
    // 64-bit result instead of the lower 32-bits.
    //
    // C(x) = R(x) ^ T2(x) / x^32

    println!("others t1 {:?}", arch::_mm_extract_epi32(arch::_mm_xor_si128(x, t2), 1) as u32);
    arch::_mm_extract_epi32(arch::_mm_xor_si128(x, t2), 1) as u32
}

unsafe fn reduce128(a: arch::__m128i, b: arch::__m128i, keys: arch::__m128i) -> arch::__m128i {
    let t1 = arch::_mm_clmulepi64_si128(a, keys, 0x00);
    let t2 = arch::_mm_clmulepi64_si128(a, keys, 0x11);
    arch::_mm_xor_si128(arch::_mm_xor_si128(b, t1), t2)
}

unsafe fn get(a: &mut &[u8]) -> arch::__m128i {
    debug_assert!(a.len() >= 16);
    let r = arch::_mm_loadu_si128(a.as_ptr() as *const arch::__m128i);
    *a = &a[16..];
    return r;
}

#[test]
fn get_128_test() {
    let mut data = b"12345678901234567890";
    unsafe {
        let mut data = b"12345678901234567890";
        let mine = get_simd_128(&mut &data[0..]);
        println!("{:?}", mine);
        let mut data = b"12345678901234567890";
        let others = get(&mut &data[0..]);
        println!("{:?}", others);
    }
}

#[test]
fn fold_128_test() {
    let mut data = b"12345678901234567890";
    unsafe {
        let mut data = b"12345678901234567890";
        let mine = get_simd_128(&mut &data[0..]);
        println!("{:?}", mine);
        let mut data = b"12345678901234567890";
        let others = get(&mut &data[0..]);
        println!("{:?}", others);
    }
}

#[test]
fn compare_crc_implementation() {
    unsafe {
        let data = include_bytes!("../../../sample_files/test_data.txt");
        calculate(0, data);
        let data = include_bytes!("../../../sample_files/test_data.txt");
        const CRC32_SIMD_CONSTANTS: [u64; 7] = [
            0x154442bd4u64,
            0x1c6e41596u64,
            0x1751997d0u64,
            0x0ccaa009eu64,
            0x163cd6124u64,
            0x1db710640u64,
            0x1F7011641u64,
        ];
        let others = compute_crc(0, CRC32_SIMD_CONSTANTS, 0x104C11DB7u64, data);
    }
}*/
