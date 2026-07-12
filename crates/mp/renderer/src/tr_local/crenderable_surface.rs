#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

use crate::mdx_format::mdxm_surface_t::mdxmSurface_t;

/// Raven `CRenderableSurface` — a Ghoul2 render-side surface: ident, bone
/// cache, and (under `_G2_GORE`) alternate gore texcoords/chain/scale/fade.
///
/// Raven: ident of this surface - required so the materials renderer knows
/// what sort of surface this refers to.
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2047-2101`
#[repr(C)]
pub struct CRenderableSurface {
    pub ident: c_int,
    //TODO: Port CBoneCache
    // Source: oracle/codemp/renderer/tr_local.h:2055
    pub boneCache: *mut core::ffi::c_void,
    // pointer to surface data loaded into file - only used by client
    // renderer DO NOT USE IN GAME SIDE - if there is a vid restart this
    // will be out of wack on the game
    pub surfaceData: *mut mdxmSurface_t,
    // alternate texture coordinates.
    pub alternateTex: *mut f32,
    pub goreChain: *mut core::ffi::c_void,

    pub scale: f32,
    pub fade: f32,
    // this is a number between 0 and 1 that dictates the progression of the
    // bullet impact
    pub impactTime: f32,
}

const _: () = assert!(core::mem::offset_of!(CRenderableSurface, ident) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<CRenderableSurface>() == 56);
    assert!(core::mem::offset_of!(CRenderableSurface, boneCache) == 8);
    assert!(core::mem::offset_of!(CRenderableSurface, surfaceData) == 16);
    assert!(core::mem::offset_of!(CRenderableSurface, alternateTex) == 24);
    assert!(core::mem::offset_of!(CRenderableSurface, goreChain) == 32);
    assert!(core::mem::offset_of!(CRenderableSurface, scale) == 40);
    assert!(core::mem::offset_of!(CRenderableSurface, fade) == 44);
    assert!(core::mem::offset_of!(CRenderableSurface, impactTime) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<CRenderableSurface>() == 32);
    assert!(core::mem::offset_of!(CRenderableSurface, boneCache) == 4);
    assert!(core::mem::offset_of!(CRenderableSurface, surfaceData) == 8);
    assert!(core::mem::offset_of!(CRenderableSurface, alternateTex) == 12);
    assert!(core::mem::offset_of!(CRenderableSurface, goreChain) == 16);
    assert!(core::mem::offset_of!(CRenderableSurface, scale) == 20);
    assert!(core::mem::offset_of!(CRenderableSurface, fade) == 24);
    assert!(core::mem::offset_of!(CRenderableSurface, impactTime) == 28);
};
