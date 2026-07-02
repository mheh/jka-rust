#![allow(non_camel_case_types, non_snake_case)]

/// Raven `surfaceInfo_t` — per-surface override info for a Ghoul2 model instance.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/codemp/ghoul2/ghoul2_shared.h:38-56`
#[repr(C)]
pub struct surfaceInfo_t {
    /// what the flags are for this model
    pub offFlags: i32,
    /// index into array held inside the model definition of pointers to the actual surface data loaded in - used by both client and game
    pub surface: i32,
    /// point 0 barycentric coors
    pub genBarycentricJ: f32,
    /// point 1 barycentric coors - point 2 is 1 - point0 - point1
    pub genBarycentricI: f32,
    /// used to point back to the original surface and poly if this is a generated surface
    pub genPolySurfaceIndex: i32,
    /// used to determine original lod of original surface and poly hit location
    pub genLod: i32,
}

const _: () = assert!(core::mem::size_of::<surfaceInfo_t>() == 24);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, offFlags) == 0);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, surface) == 4);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, genBarycentricJ) == 8);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, genBarycentricI) == 12);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, genPolySurfaceIndex) == 16);
const _: () = assert!(core::mem::offset_of!(surfaceInfo_t, genLod) == 20);
