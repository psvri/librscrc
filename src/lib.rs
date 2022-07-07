//! # librscrc
//!
//! `librscrc` is a collection of crc32 algorithms with support for various approaches and custom polynomial.
//!
//! # Usage examples
//! ## naive
//! ```
//! use librscrc::prelude::*;
//!
//! // compute crc32
//! let mut crc = Crc32::new();
//! crc.update(b"123456789");
//! assert_eq!(crc.digest(), 0xCBF43926);
//!
//! // compute crc32c
//! let mut crc = Crc32C::new();
//! crc.update(b"123456789");
//! assert_eq!(crc.digest(), 0xE3069283);
//! ```
//!
//! ## simd
//! ```
//! use librscrc::prelude::*;
//!
//! // compute crc32
//! let mut crc = Crc32::new_simd();
//! crc.update(b"123456789");
//! assert_eq!(crc.digest(), 0xCBF43926);
//!
//! //compute crc32c
//! let mut crc = Crc32C::new_simd();
//! crc.update(b"123456789");
//! assert_eq!(crc.digest(), 0xE3069283);
//! ```
//!
//! # Custom polynomial example
//! ```
//! use librscrc::prelude::*;
//!
//! // you can provide a 33 bit polynomial or a 32 bit polynomial.
//! let mut crc = CustomCrc32::new_simd(0x104C11DB7u64);
//! crc.update(b"123456789");
//! assert_eq!(crc.digest(), 0xCBF43926);
//!
//!```

#![cfg_attr(
    all(
        feature = "nightly",
        feature = "hardware",
        any(target_arch = "aarch64")
    ),
    feature(stdsimd)
)]

pub mod check32;
pub mod prelude;
