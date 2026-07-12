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

const _: () = assert!(core::mem::offset_of!(MD4_CTX, state) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<MD4_CTX>() == 112);
    assert!(core::mem::offset_of!(MD4_CTX, count) == 32);
    assert!(core::mem::offset_of!(MD4_CTX, buffer) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<MD4_CTX>() == 88);
    assert!(core::mem::offset_of!(MD4_CTX, count) == 16);
    assert!(core::mem::offset_of!(MD4_CTX, buffer) == 24);
};
