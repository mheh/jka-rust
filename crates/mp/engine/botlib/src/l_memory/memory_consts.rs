#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

/// Raven `MEM_ID` — magic tag stamped on allocated memory blocks.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:23`
pub const MEM_ID: c_long = 0x12345678;

/// Raven `HUNK_ID` — magic tag stamped on hunk-allocated memory blocks.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:24`
// 0x87654321 exceeds i32::MAX; on ILP32 it wraps to the same negative value
// Raven's `unsigned int -> long` conversion produced.
pub const HUNK_ID: c_long = 0x87654321u32 as c_long;
