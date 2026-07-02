#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mdxmSurface_t` — per-surface geometry offsets within an `.mdxm`.
///
/// Type definition source: `oracle/oracle/codemp/renderer/../ghoul2/../renderer/mdx_format.h:219-243`
#[repr(C)]
pub struct mdxmSurface_t {
    /// this one field at least should be kept, since the game-engine may switch-case (but currently=0 in carcass)
    pub ident: i32,
    /// 0...mdxmHeader_t->numSurfaces-1 (because of how ingame renderer works)
    pub thisSurfaceIndex: i32,
    /// this will be a negative number, pointing back to main header
    pub ofsHeader: i32,
    pub numVerts: i32,
    pub ofsVerts: i32,
    pub numTriangles: i32,
    pub ofsTriangles: i32,
    // Bone references are a set of ints representing all the bones
    // present in any vertex weights for this surface.  This is
    // needed because a model may have surfaces that need to be
    // drawn at different sort times, and we don't want to have
    // to re-interpolate all the bones for each surface.
    pub numBoneReferences: i32,
    pub ofsBoneReferences: i32,
    /// next surface follows
    pub ofsEnd: i32,
}

const _: () = assert!(core::mem::size_of::<mdxmSurface_t>() == 40);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, thisSurfaceIndex) == 4);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ofsHeader) == 8);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, numVerts) == 12);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ofsVerts) == 16);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, numTriangles) == 20);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ofsTriangles) == 24);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, numBoneReferences) == 28);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ofsBoneReferences) == 32);
const _: () = assert!(core::mem::offset_of!(mdxmSurface_t, ofsEnd) == 36);
