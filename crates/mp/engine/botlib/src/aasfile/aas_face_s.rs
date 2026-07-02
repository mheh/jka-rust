#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_face_s` — an AAS face bounding an area, with plane and edge-index info.
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:174-182`
#[repr(C)]
pub struct aas_face_t {
    /// number of the plane this face is in
    pub planenum: i32,
    /// face flags (no use to create face settings for just this field)
    pub faceflags: i32,
    /// number of edges in the boundary of the face
    pub numedges: i32,
    /// first edge in the edge index
    pub firstedge: i32,
    /// area at the front of this face
    pub frontarea: i32,
    /// area at the back of this face
    pub backarea: i32,
}

pub type aas_face_s = aas_face_t;

const _: () = assert!(core::mem::size_of::<aas_face_t>() == 24);
const _: () = assert!(core::mem::offset_of!(aas_face_t, planenum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_face_t, faceflags) == 4);
const _: () = assert!(core::mem::offset_of!(aas_face_t, numedges) == 8);
const _: () = assert!(core::mem::offset_of!(aas_face_t, firstedge) == 12);
const _: () = assert!(core::mem::offset_of!(aas_face_t, frontarea) == 16);
const _: () = assert!(core::mem::offset_of!(aas_face_t, backarea) == 20);
