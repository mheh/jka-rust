#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use sp_qshared::shared::{qboolean, vec2_t, vec4_t};

use crate::tr_local::texture_bundle_t::textureBundle_t;

/// Raven `SHADER_MAX_VERTEXES` — max verts a `CQuickSpriteSystem` batch can hold.
///
/// Source: `oracle/code/renderer/tr_local.h`
pub const SHADER_MAX_VERTEXES: usize = 1000;

/// Raven `CQuickSpriteSystem` — batches quad "sprites" (particles, marks, etc.)
/// sharing a texture bundle/GL state into a single flush, with optional fog.
///
/// Type definition source: `oracle/code/renderer/tr_quicksprite.h:16-41`
#[repr(C)]
pub struct CQuickSpriteSystem {
    mTexBundle: *mut textureBundle_t,
    mGLStateBits: c_ulong,
    mFogIndex: i32,
    mUseFog: qboolean,
    mVerts: [vec4_t; SHADER_MAX_VERTEXES],
    // Ideally this would be static, cause it never changes
    mIndexes: [u32; SHADER_MAX_VERTEXES],
    // Ideally this would be static, cause it never changes
    mTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],
    mFogTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],
    mColors: [c_ulong; SHADER_MAX_VERTEXES],
    mNextVert: i32,
    mTurnCullBackOn: qboolean,
}

const _: () = assert!(core::mem::size_of::<CQuickSpriteSystem>() == 44032);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mTexBundle) == 0);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mGLStateBits) == 8);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mFogIndex) == 16);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mUseFog) == 20);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mVerts) == 24);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mIndexes) == 16024);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mTextureCoords) == 20024);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mFogTextureCoords) == 28024);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mColors) == 36024);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mNextVert) == 44024);
const _: () = assert!(core::mem::offset_of!(CQuickSpriteSystem, mTurnCullBackOn) == 44028);
