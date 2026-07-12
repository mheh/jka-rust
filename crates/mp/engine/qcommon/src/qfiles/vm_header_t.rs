#![allow(non_camel_case_types, non_snake_case)]

/// Raven `vmHeader_t` — compiled QVM bytecode file header.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:26-38`
#[repr(C)]
pub struct vmHeader_t {
    pub vmMagic: i32,

    pub instructionCount: i32,

    pub codeOffset: i32,
    pub codeLength: i32,

    pub dataOffset: i32,
    pub dataLength: i32,
    pub litLength: i32, // ( dataLength - litLength ) should be byteswapped on load
    pub bssLength: i32, // zero filled memory appended to datalength
}

const _: () = assert!(core::mem::size_of::<vmHeader_t>() == 32);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, vmMagic) == 0);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, instructionCount) == 4);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, codeOffset) == 8);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, codeLength) == 12);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, dataOffset) == 16);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, dataLength) == 20);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, litLength) == 24);
const _: () = assert!(core::mem::offset_of!(vmHeader_t, bssLength) == 28);
