#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use native_types::MAX_QPATH;

/// Raven `MAX_PARSEFILES`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:2655`
pub const MAX_PARSEFILES: usize = 16;

/// Raven `parseData_t` (`parseData_s`) — SP text-parser file state.
///
/// SP-only (not present in MP `q_shared.h`).
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2656-2662`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct parseData_t {
    /// Name of current file being read in.
    pub fileName: [c_char; MAX_QPATH],
    /// Number of lines read in.
    pub com_lines: c_int,
    /// Start address of buffer holding data that was read in.
    pub bufferStart: *const c_char,
    /// Where data is currently being parsed from buffer.
    pub bufferCurrent: *const c_char,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<parseData_t>() == 88);
    assert!(offset_of!(parseData_t, com_lines) == 64);
    assert!(offset_of!(parseData_t, bufferStart) == 72);
    assert!(offset_of!(parseData_t, bufferCurrent) == 80);
};
