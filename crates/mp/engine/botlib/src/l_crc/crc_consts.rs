#![allow(non_camel_case_types, non_snake_case)]

/// Raven `CRC_INIT_VALUE` — initial CRC accumulator value.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:30`
pub const CRC_INIT_VALUE: ::core::ffi::c_ushort = 0xffff;

/// Raven `CRC_XOR_VALUE` — final CRC XOR mask.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:31`
pub const CRC_XOR_VALUE: ::core::ffi::c_ushort = 0x0000;
