#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MEM_ID` — magic tag stamped on allocated memory blocks.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:23`
pub const MEM_ID: ::core::ffi::c_long = 0x12345678;

/// Raven `HUNK_ID` — magic tag stamped on hunk-allocated memory blocks.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:24`
pub const HUNK_ID: ::core::ffi::c_long = 0x87654321;
