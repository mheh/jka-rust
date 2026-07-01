//! MP `tr_types.h` GL configuration snapshot.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use crate::common::mp::cgame::texture_compression_t::textureCompression_t;
use crate::shared::qboolean;

/// Raven `glconfig_t` — renderer/GL capabilities the engine fills for modules
/// (`CG_GETGLCONFIG` / `UI_GETGLCONFIG` copy one across the ABI seam).
///
/// Type definition source: `oracle/oracle/codemp/cgame/tr_types.h:299-325`
#[repr(C)]
pub struct glconfig_t {
    pub renderer_string: *const c_char,
    pub vendor_string: *const c_char,
    pub version_string: *const c_char,
    pub extensions_string: *const c_char,

    pub maxTextureSize: i32,   // queried from GL
    pub maxActiveTextures: i32, // multitexture ability
    pub maxTextureFilterAnisotropy: f32,

    pub colorBits: i32,
    pub depthBits: i32,
    pub stencilBits: i32,

    pub deviceSupportsGamma: qboolean,
    pub textureCompression: textureCompression_t,
    pub textureEnvAddAvailable: qboolean,
    pub clampToEdgeAvailable: qboolean,

    pub vidWidth: i32,
    pub vidHeight: i32,

    pub displayFrequency: i32,

    // Raven: synonymous with "does rendering consume the entire screen?", therefore
    // a Voodoo or Voodoo2 will have this set to TRUE, as will a Win32 ICD that
    // used CDS.
    pub isFullscreen: qboolean,
    pub stereoEnabled: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<glconfig_t>() == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, renderer_string) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, vendor_string) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, version_string) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, extensions_string) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, maxTextureSize) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, maxActiveTextures) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, maxTextureFilterAnisotropy) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, colorBits) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, depthBits) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, stencilBits) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, deviceSupportsGamma) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, textureCompression) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, textureEnvAddAvailable) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, clampToEdgeAvailable) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, vidWidth) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, vidHeight) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, displayFrequency) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, isFullscreen) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(glconfig_t, stereoEnabled) == 88);
