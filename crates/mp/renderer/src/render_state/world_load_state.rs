//! `WorldLoadState` — the `tr` fields the sim writes at load and the render
//! side only reads (W2-F3).

use crate::render_state::placeholders::Vec3;

/// The sim-written half of the old `FrameState`.
///
/// Every field here is written on the sim thread, at image init, at BSP load,
/// or once per frame in `RE_BeginFrame`, and read on the render thread during
/// the walk and the draw. W2-F3 splits them out so the render-resident view
/// state carries nothing the sim writes. A copy rides on each `FramePackage`,
/// which is why the struct stays small and `Copy`.
///
/// `tr.externalVisData` is not here: `RE_SetWorldVisData` writes it and
/// `R_LoadVisibility` reads it, both at BSP load on the sim thread, so it
/// lives on `RenderAssets` beside the world it belongs to.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1309-1423`
#[derive(Clone, Copy)]
pub struct WorldLoadState {
    /// `tr.frameCount` — Raven: incremented every frame.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1313`
    pub frame_count: i32,
    /// `tr.identityLight` — `1.0 / ( 1 << overbrightBits )`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1374`
    pub identity_light: f32,
    /// `tr.identityLightByte` — `identityLight * 255`, truncated to `int` by
    /// the oracle's assignment.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1375`
    pub identity_light_byte: i32,
    /// `tr.overbrightBits` — the lightmap/vertex-color shift
    /// `R_ColorShiftLightingBytes` applies at BSP load.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1376`
    pub overbright_bits: i32,
    /// `tr.sunDirection`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1385`
    pub sun_direction: Vec3,
    /// `tr.sunAmbient` — Raven: "from the sky shader (only used for John's
    /// terrain system)".
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1387`
    pub sun_ambient: Vec3,
}

impl Default for WorldLoadState {
    /// The values Raven's loader-zeroed `tr` starts at, with `identityLight`
    /// at its no-overbright default so a frame drawn before
    /// `R_SetColorMappings` runs is not black.
    fn default() -> WorldLoadState {
        WorldLoadState {
            frame_count: 0,
            identity_light: 1.0,
            identity_light_byte: 255,
            overbright_bits: 0,
            sun_direction: Vec3::default(),
            sun_ambient: Vec3::default(),
        }
    }
}
