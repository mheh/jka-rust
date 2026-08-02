#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

/// Raven `POINT` (`windows.h` `tagPOINT`) - a 2D vertex, `x`/`y` as 32-bit
/// signed longs. `cm_draw.cpp`'s scan converter is the only qcommon consumer,
/// so this is a local stand-in rather than a shared windows-typedef module.
///
/// Source: `oracle/codemp/qcommon/cm_draw.cpp:1082` (declaration site);
/// the type itself is the Win32 `POINT` (`{ LONG x, y; }`).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}
