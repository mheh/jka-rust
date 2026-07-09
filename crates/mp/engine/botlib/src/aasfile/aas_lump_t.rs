#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_lump_t` — offset and length of a lump within an AAS file.
///
/// Raven: none.
/// Type definition source: `oracle/codemp/botlib/aasfile.h:210-214`
#[repr(C)]
pub struct aas_lump_t {
    pub fileofs: i32,
    pub filelen: i32,
}

const _: () = assert!(core::mem::size_of::<aas_lump_t>() == 8);
const _: () = assert!(core::mem::offset_of!(aas_lump_t, fileofs) == 0);
const _: () = assert!(core::mem::offset_of!(aas_lump_t, filelen) == 4);
