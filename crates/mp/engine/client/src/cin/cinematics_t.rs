#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_long, c_short, c_uint};

use native_types::byte;

use super::cin_consts::{DEFAULT_CIN_HEIGHT, DEFAULT_CIN_WIDTH};

/// Raven `linbuf` length — `DEFAULT_CIN_WIDTH * DEFAULT_CIN_HEIGHT * 4 * 2`, the
/// two-screen 32-bit decode surface.
const LINBUF_LEN: usize = (DEFAULT_CIN_WIDTH * DEFAULT_CIN_HEIGHT * 4 * 2) as usize;

/// Raven `cinematics_t` — the single RoQ decode scratch surface that every
/// cinematic handle shares. Internal to the client, so it never crosses the ABI
/// seam and carries no layout asserts.
///
/// Type definition source: `oracle/codemp/client/cl_cin.cpp:67-76`
#[repr(C)]
pub struct cinematics_t {
    pub linbuf: [byte; LINBUF_LEN],
    pub file: [byte; 65536],
    pub sqrTable: [c_short; 256],

    pub mcomp: [c_uint; 256],
    pub qStatus: [[*mut byte; 32768]; 2],

    pub oldXOff: c_long,
    pub oldYOff: c_long,
    pub oldysize: c_uint,
    pub oldxsize: c_uint,
}

// Every field is a scalar array or a null-valid pointer array, and Raven's `cin`
// is a zero-filled file static, so the all-zero image is a valid inhabitant.
// The 2.6 MB mass builds heap-first through `zeroed_box` (STATE-D9).
unsafe impl native_platform::ZeroValid for cinematics_t {}
