#![allow(non_camel_case_types, non_snake_case)]

/// Raven `VM_MAGIC` — expected `vmHeader_t::vmMagic` for a compiled QVM file.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:25`
pub const VM_MAGIC: i32 = 0x12721444;
