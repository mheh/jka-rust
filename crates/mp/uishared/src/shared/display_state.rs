//! `DisplayState` — the data tail of Raven's `displayContextDef_t`.

use core::ffi::c_int;
use core::ptr::null;

use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::{qfalse, qhandle_t};

use super::cached_assets_t::CachedAssets;

/// The scalar/asset tail of Raven's `displayContextDef_t` — everything after
/// the ~52 function pointers that [`DisplayContext`] replaces (DEC-36 D3).
///
/// The split is by kind, not by convenience: the function pointers were the
/// host's *behavior* and became trait methods; these fields are the host's
/// *data* (frame timing, cursor position, virtual-screen scale, the registered
/// asset bag and the engine's `glconfig`), so they stay a plain owned struct
/// the host holds — `UiWorld::uiDC`, later `CgWorld`'s `cgDC` — and the
/// framework reaches through [`DisplayContext::display`].
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:460-476`
// `glconfig_t` is the frozen ABI struct (raw `*const c_char` strings the
// engine owns), so it carries no derives; `DisplayState` inherits that and
// derives none either.
#[allow(non_snake_case)]
pub struct DisplayState {
    pub yscale: f32,
    pub xscale: f32,
    pub bias: f32,
    pub realTime: c_int,
    pub frameTime: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,
    pub debug: bool,

    pub Assets: CachedAssets,

    /// Engine-filled through `trap_GetGlconfig`; stays the frozen `#[repr(C)]`
    /// ABI struct (Class B — the bytes cross the seam by copy).
    pub glconfig: glconfig_t,
    pub whiteShader: qhandle_t,
    pub gradientImage: qhandle_t,
    pub cursor: qhandle_t,
    pub FPS: f32,
}

impl Default for DisplayState {
    /// Raven zero-initialized `uiInfo.uiDC` with the rest of `uiInfo` (a
    /// file-scope struct) and filled it in `_UI_Init`.
    fn default() -> Self {
        DisplayState {
            yscale: 0.0,
            xscale: 0.0,
            bias: 0.0,
            realTime: 0,
            frameTime: 0,
            cursorx: 0,
            cursory: 0,
            debug: false,
            Assets: CachedAssets::default(),
            glconfig: glconfig_t {
                renderer_string: null(),
                vendor_string: null(),
                version_string: null(),
                extensions_string: null(),
                maxTextureSize: 0,
                maxActiveTextures: 0,
                maxTextureFilterAnisotropy: 0.0,
                colorBits: 0,
                depthBits: 0,
                stencilBits: 0,
                deviceSupportsGamma: qfalse,
                textureCompression: textureCompression_t::TC_NONE,
                textureEnvAddAvailable: qfalse,
                clampToEdgeAvailable: qfalse,
                vidWidth: 0,
                vidHeight: 0,
                displayFrequency: 0,
                isFullscreen: qfalse,
                stereoEnabled: qfalse,
            },
            whiteShader: 0,
            gradientImage: 0,
            cursor: 0,
            FPS: 0.0,
        }
    }
}
