#![allow(non_camel_case_types)]

/// Raven `ERMDir` — directions you can proceed from path cells.
///
/// Raven: directions you can proceed from cells.
/// Type definition source: `oracle/oracle/codemp/RMG/RM_Path.h:24-37`
// Raven's `DIR_FIRST = 0` aliases `DIR_N`; Rust enums forbid duplicate
// discriminants, so it is a const alias below.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ERMDir {
    DIR_N = 0,
    DIR_NE,
    DIR_E,
    DIR_SE,
    DIR_S,
    DIR_SW,
    DIR_W,
    DIR_NW,
    DIR_MAX,
    DIR_ALL = 255,
}

pub const DIR_FIRST: ERMDir = ERMDir::DIR_N;

const _: () = assert!(core::mem::size_of::<ERMDir>() == 4);
