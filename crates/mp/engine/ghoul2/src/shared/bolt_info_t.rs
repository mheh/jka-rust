#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::mdxaBone_t;

/// Raven `boltInfo_t` — per-bolt attachment info for a Ghoul2 model instance.
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/ghoul2/ghoul2_shared.h:170-182`
#[repr(C)]
pub struct boltInfo_t {
    /// bone number bolt attaches to
    pub boneNumber: i32,
    /// surface number bolt attaches to
    pub surfaceNumber: i32,
    /// if we attach to a surface, this tells us if it is an original surface or a generated one - doesn't go across the network
    pub surfaceType: i32,
    /// nor does this
    pub boltUsed: i32,
    /// this does not go across the network
    pub position: mdxaBone_t,
}

const _: () = assert!(core::mem::size_of::<boltInfo_t>() == 64);
const _: () = assert!(core::mem::offset_of!(boltInfo_t, boneNumber) == 0);
const _: () = assert!(core::mem::offset_of!(boltInfo_t, surfaceNumber) == 4);
const _: () = assert!(core::mem::offset_of!(boltInfo_t, surfaceType) == 8);
const _: () = assert!(core::mem::offset_of!(boltInfo_t, boltUsed) == 12);
const _: () = assert!(core::mem::offset_of!(boltInfo_t, position) == 16);
