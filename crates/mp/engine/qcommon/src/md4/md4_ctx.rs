#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_uchar;

use super::uint4::UINT4;

/// Raven `MD4_CTX` — MD4 message-digest context (RSA reference implementation).
///
/// Type definition source: `oracle/codemp/qcommon/md4.cpp:34-38`
#[repr(C)]
pub struct MD4_CTX {
    /// state (ABCD)
    pub state: [UINT4; 4],
    /// number of bits, modulo 2^64 (lsb first)
    pub count: [UINT4; 2],
    /// input buffer
    pub buffer: [c_uchar; 64],
}

const _: () = assert!(core::mem::size_of::<MD4_CTX>() == 112);
const _: () = assert!(core::mem::offset_of!(MD4_CTX, state) == 0);
const _: () = assert!(core::mem::offset_of!(MD4_CTX, count) == 32);
const _: () = assert!(core::mem::offset_of!(MD4_CTX, buffer) == 48);
