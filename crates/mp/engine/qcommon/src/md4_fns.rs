//! `md4.cpp` free functions — RSA MD4 message-digest algorithm.
//!
//! Source: `oracle/codemp/qcommon/md4.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uchar, c_uint};

use crate::common::common::Common;
use crate::common_fns::{Com_Memcpy, Com_Memset};
use crate::md4::md4_cpp_consts::{S11, S12, S13, S14, S21, S22, S23, S24, S31, S32, S33, S34};
use crate::md4::md4_ctx::MD4_CTX;
use crate::md4::uint4::UINT4;

/// Raven `PADDING` — MD4 padding buffer (first byte 0x80, rest zero).
///
/// Source: `oracle/codemp/qcommon/md4.cpp:78-80`
static PADDING: [c_uchar; 64] = {
    let mut a = [0u8; 64];
    a[0] = 0x80;
    a
};

// Raven's `UINT4` is 32-bit on every retail platform; `c_ulong` widens to 8
// bytes on LP64, so the round math truncates to `u32` — digests match the
// 32-bit retail client's bit-for-bit on every host.

/// Raven `F` — basic MD4 function.
/// Source: `oracle/codemp/qcommon/md4.cpp:83`
#[inline(always)]
fn F(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

/// Raven `G` — basic MD4 function.
/// Source: `oracle/codemp/qcommon/md4.cpp:84`
#[inline(always)]
fn G(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (x & z) | (y & z)
}

/// Raven `H` — basic MD4 function.
/// Source: `oracle/codemp/qcommon/md4.cpp:85`
#[inline(always)]
fn H(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

/// Raven `ROTATE_LEFT`.
/// Source: `oracle/codemp/qcommon/md4.cpp:88`
#[inline(always)]
fn ROTATE_LEFT(x: u32, n: u32) -> u32 {
    (x << n) | (x >> (32 - n))
}

/// Raven `FF` — transformation for round 1.
/// Source: `oracle/codemp/qcommon/md4.cpp:92`
#[inline(always)]
fn FF(a: UINT4, b: UINT4, c: UINT4, d: UINT4, x: UINT4, s: u32) -> UINT4 {
    let a = (a as u32)
        .wrapping_add(F(b as u32, c as u32, d as u32))
        .wrapping_add(x as u32);
    ROTATE_LEFT(a, s) as UINT4
}

/// Raven `GG` — transformation for round 2.
/// Source: `oracle/codemp/qcommon/md4.cpp:94`
#[inline(always)]
fn GG(a: UINT4, b: UINT4, c: UINT4, d: UINT4, x: UINT4, s: u32) -> UINT4 {
    let a = (a as u32)
        .wrapping_add(G(b as u32, c as u32, d as u32))
        .wrapping_add(x as u32)
        .wrapping_add(0x5a827999);
    ROTATE_LEFT(a, s) as UINT4
}

/// Raven `HH` — transformation for round 3.
/// Source: `oracle/codemp/qcommon/md4.cpp:96`
#[inline(always)]
fn HH(a: UINT4, b: UINT4, c: UINT4, d: UINT4, x: UINT4, s: u32) -> UINT4 {
    let a = (a as u32)
        .wrapping_add(H(b as u32, c as u32, d as u32))
        .wrapping_add(x as u32)
        .wrapping_add(0x6ed9eba1);
    ROTATE_LEFT(a, s) as UINT4
}

/// Raven `MD4Init` — MD4 initialization. Begins an MD4 operation, writing a new context.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:100-109`
pub fn MD4Init(context: *mut MD4_CTX) {
    unsafe {
        (*context).count[0] = 0;
        (*context).count[1] = 0;

        /* Load magic initialization constants.*/
        (*context).state[0] = 0x67452301;
        (*context).state[1] = 0xefcdab89;
        (*context).state[2] = 0x98badcfe;
        (*context).state[3] = 0x10325476;
    }
}

/// Raven `Encode` — encodes input (UINT4) into output (unsigned char), assuming len is a
/// multiple of 4.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:243-253`
pub fn Encode(output: *mut c_uchar, input: *mut UINT4, len: c_uint) {
    unsafe {
        let mut i: c_uint = 0;
        let mut j: c_uint = 0;
        while j < len {
            let word = *input.offset(i as isize);
            *output.offset(j as isize) = (word & 0xff) as c_uchar;
            *output.offset(j as isize + 1) = ((word >> 8) & 0xff) as c_uchar;
            *output.offset(j as isize + 2) = ((word >> 16) & 0xff) as c_uchar;
            *output.offset(j as isize + 3) = ((word >> 24) & 0xff) as c_uchar;
            i += 1;
            j += 4;
        }
    }
}

/// Raven `Decode` — decodes input (unsigned char) into output (UINT4), assuming len is a
/// multiple of 4.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:257-263`
pub fn Decode(output: *mut UINT4, input: *const c_uchar, len: c_uint) {
    unsafe {
        let mut i: c_uint = 0;
        let mut j: c_uint = 0;
        while j < len {
            let b0 = *input.offset(j as isize) as UINT4;
            let b1 = *input.offset(j as isize + 1) as UINT4;
            let b2 = *input.offset(j as isize + 2) as UINT4;
            let b3 = *input.offset(j as isize + 3) as UINT4;
            *output.offset(i as isize) = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
            i += 1;
            j += 4;
        }
    }
}

/// Raven `MD4Transform` — basic MD4 step. Transforms state based on block.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:172-239`
pub fn MD4Transform(state: *mut UINT4, block: *const c_uchar) {
    unsafe {
        let mut a = *state.offset(0);
        let mut b = *state.offset(1);
        let mut c = *state.offset(2);
        let mut d = *state.offset(3);
        let mut x = [0 as UINT4; 16];

        Decode(x.as_mut_ptr(), block, 64);

        /* Round 1 */
        a = FF(a, b, c, d, x[0], S11 as u32); /* 1 */
        d = FF(d, a, b, c, x[1], S12 as u32); /* 2 */
        c = FF(c, d, a, b, x[2], S13 as u32); /* 3 */
        b = FF(b, c, d, a, x[3], S14 as u32); /* 4 */
        a = FF(a, b, c, d, x[4], S11 as u32); /* 5 */
        d = FF(d, a, b, c, x[5], S12 as u32); /* 6 */
        c = FF(c, d, a, b, x[6], S13 as u32); /* 7 */
        b = FF(b, c, d, a, x[7], S14 as u32); /* 8 */
        a = FF(a, b, c, d, x[8], S11 as u32); /* 9 */
        d = FF(d, a, b, c, x[9], S12 as u32); /* 10 */
        c = FF(c, d, a, b, x[10], S13 as u32); /* 11 */
        b = FF(b, c, d, a, x[11], S14 as u32); /* 12 */
        a = FF(a, b, c, d, x[12], S11 as u32); /* 13 */
        d = FF(d, a, b, c, x[13], S12 as u32); /* 14 */
        c = FF(c, d, a, b, x[14], S13 as u32); /* 15 */
        b = FF(b, c, d, a, x[15], S14 as u32); /* 16 */

        /* Round 2 */
        a = GG(a, b, c, d, x[0], S21 as u32); /* 17 */
        d = GG(d, a, b, c, x[4], S22 as u32); /* 18 */
        c = GG(c, d, a, b, x[8], S23 as u32); /* 19 */
        b = GG(b, c, d, a, x[12], S24 as u32); /* 20 */
        a = GG(a, b, c, d, x[1], S21 as u32); /* 21 */
        d = GG(d, a, b, c, x[5], S22 as u32); /* 22 */
        c = GG(c, d, a, b, x[9], S23 as u32); /* 23 */
        b = GG(b, c, d, a, x[13], S24 as u32); /* 24 */
        a = GG(a, b, c, d, x[2], S21 as u32); /* 25 */
        d = GG(d, a, b, c, x[6], S22 as u32); /* 26 */
        c = GG(c, d, a, b, x[10], S23 as u32); /* 27 */
        b = GG(b, c, d, a, x[14], S24 as u32); /* 28 */
        a = GG(a, b, c, d, x[3], S21 as u32); /* 29 */
        d = GG(d, a, b, c, x[7], S22 as u32); /* 30 */
        c = GG(c, d, a, b, x[11], S23 as u32); /* 31 */
        b = GG(b, c, d, a, x[15], S24 as u32); /* 32 */

        /* Round 3 */
        a = HH(a, b, c, d, x[0], S31 as u32); /* 33 */
        d = HH(d, a, b, c, x[8], S32 as u32); /* 34 */
        c = HH(c, d, a, b, x[4], S33 as u32); /* 35 */
        b = HH(b, c, d, a, x[12], S34 as u32); /* 36 */
        a = HH(a, b, c, d, x[2], S31 as u32); /* 37 */
        d = HH(d, a, b, c, x[10], S32 as u32); /* 38 */
        c = HH(c, d, a, b, x[6], S33 as u32); /* 39 */
        b = HH(b, c, d, a, x[14], S34 as u32); /* 40 */
        a = HH(a, b, c, d, x[1], S31 as u32); /* 41 */
        d = HH(d, a, b, c, x[9], S32 as u32); /* 42 */
        c = HH(c, d, a, b, x[5], S33 as u32); /* 43 */
        b = HH(b, c, d, a, x[13], S34 as u32); /* 44 */
        a = HH(a, b, c, d, x[3], S31 as u32); /* 45 */
        d = HH(d, a, b, c, x[11], S32 as u32); /* 46 */
        c = HH(c, d, a, b, x[7], S33 as u32); /* 47 */
        b = HH(b, c, d, a, x[15], S34 as u32); /* 48 */

        *state.offset(0) = (*state.offset(0)).wrapping_add(a);
        *state.offset(1) = (*state.offset(1)).wrapping_add(b);
        *state.offset(2) = (*state.offset(2)).wrapping_add(c);
        *state.offset(3) = (*state.offset(3)).wrapping_add(d);

        /* Zeroize sensitive information.*/
        Com_Memset(x.as_mut_ptr() as *mut (), 0, core::mem::size_of_val(&x));
    }
}

/// Raven `MD4Update` — MD4 block update operation. Continues an MD4 message-digest
/// operation, processing another message block, and updating the context.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:112-143`
pub fn MD4Update(context: *mut MD4_CTX, input: *const c_uchar, inputLen: c_uint) {
    unsafe {
        /* Compute number of bytes mod 64 */
        let mut index = (((*context).count[0] >> 3) & 0x3F) as c_uint;

        /* Update number of bits */
        let add_bits = (inputLen as UINT4) << 3;
        let new_count0 = (*context).count[0].wrapping_add(add_bits);
        if new_count0 < add_bits {
            (*context).count[1] += 1;
        }
        (*context).count[0] = new_count0;

        (*context).count[1] += (inputLen as UINT4) >> 29;

        let partLen: c_uint = 64 - index;

        let mut i: c_uint;
        /* Transform as many times as possible.*/
        if inputLen >= partLen {
            Com_Memcpy(
                (*context).buffer.as_mut_ptr().offset(index as isize) as *mut (),
                input as *const (),
                partLen as usize,
            );
            MD4Transform((*context).state.as_mut_ptr(), (*context).buffer.as_ptr());

            i = partLen;
            while i + 63 < inputLen {
                MD4Transform((*context).state.as_mut_ptr(), input.offset(i as isize));
                i += 64;
            }

            index = 0;
        } else {
            i = 0;
        }

        /* Buffer remaining input */
        Com_Memcpy(
            (*context).buffer.as_mut_ptr().offset(index as isize) as *mut (),
            input.offset(i as isize) as *const (),
            (inputLen - i) as usize,
        );
    }
}

/// Raven `MD4Final` — MD4 finalization. Ends an MD4 message-digest operation, writing the
/// message digest and zeroizing the context.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:147-168`
pub fn MD4Final(common: &mut Common, digest: *mut c_uchar, context: *mut MD4_CTX) {
    let _ = common;
    unsafe {
        let mut bits: [c_uchar; 8] = [0; 8];

        /* Save number of bits */
        Encode(bits.as_mut_ptr(), (*context).count.as_mut_ptr(), 8);

        /* Pad out to 56 mod 64.*/
        let index = (((*context).count[0] >> 3) & 0x3f) as c_uint;
        let padLen: c_uint = if index < 56 { 56 - index } else { 120 - index };
        MD4Update(context, PADDING.as_ptr(), padLen);

        /* Append length (before padding) */
        MD4Update(context, bits.as_ptr(), 8);

        /* Store state in digest */
        Encode(digest, (*context).state.as_mut_ptr(), 16);

        /* Zeroize sensitive information.*/
        Com_Memset(context as *mut (), 0, core::mem::size_of::<MD4_CTX>());
    }
}

/// Raven `Com_BlockChecksum`.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:267-280`
pub fn Com_BlockChecksum(common: &mut Common, buffer: *const (), length: c_int) -> c_uint {
    unsafe {
        let mut digest: [c_int; 4] = [0; 4];
        let mut ctx: MD4_CTX = core::mem::zeroed();

        MD4Init(&mut ctx);
        MD4Update(&mut ctx, buffer as *const c_uchar, length as c_uint);
        MD4Final(common, digest.as_mut_ptr() as *mut c_uchar, &mut ctx);

        let val: c_uint = (digest[0] ^ digest[1] ^ digest[2] ^ digest[3]) as c_uint;

        val
    }
}

/// Raven `Com_BlockChecksumKey`.
///
/// Source: `oracle/codemp/qcommon/md4.cpp:282-296`
pub fn Com_BlockChecksumKey(
    common: &mut Common,
    buffer: *mut (),
    length: c_int,
    key: c_int,
) -> c_uint {
    unsafe {
        let mut digest: [c_int; 4] = [0; 4];
        let mut ctx: MD4_CTX = core::mem::zeroed();
        let mut key = key;

        MD4Init(&mut ctx);
        MD4Update(&mut ctx, &mut key as *mut c_int as *const c_uchar, 4);
        MD4Update(&mut ctx, buffer as *const c_uchar, length as c_uint);
        MD4Final(common, digest.as_mut_ptr() as *mut c_uchar, &mut ctx);

        let val: c_uint = (digest[0] ^ digest[1] ^ digest[2] ^ digest[3]) as c_uint;

        val
    }
}
