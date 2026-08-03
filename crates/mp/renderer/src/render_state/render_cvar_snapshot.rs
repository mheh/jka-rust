//! `RenderCvarSnapshot` — the render cvar values one frame reads (DEC-37
//! A13.1, R4 render-thread snapshot).

use mp_engine_qcommon::common::common::Common;

use crate::render_state::renderer_cvars::RendererCvars;

/// The resolved render cvar values that one executor frame reads.
///
/// `RendererCvars` holds the live `cvar_t*` handles. A13.1 deferred the
/// render-thread snapshot the backend needs. This struct is that snapshot: the
/// executor caller resolves each handle against the live engine cvar table
/// once, then passes this small copy by value into `FrameExecutor::
/// execute_frame` and `Pipeline3d::draw`. A caller without a live cvar table
/// (a harness or a golden test) uses [`RenderCvarSnapshot::default`], which
/// carries the retail cvar defaults.
///
/// The fields keep Raven's `->integer`/`->value` widths, so a gate compares the
/// same way the oracle does.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:105-109` (`r_skipBackEnd`),
/// `oracle/codemp/renderer/tr_backend.cpp:523-529` (`r_fastsky`),
/// `oracle/codemp/renderer/tr_surface.cpp:1555-1563` (`r_lodCurveError`)
#[derive(Clone, Copy)]
pub struct RenderCvarSnapshot {
    /// `r_skipBackEnd->integer`. A non-zero value skips the whole backend
    /// replay, so the executor draws nothing this frame. Retail default `0`.
    pub skip_back_end: i32,
    /// `r_fastsky->integer`. A non-zero value skips the sky draw and clears the
    /// world to black. Retail default `0`.
    pub fastsky: i32,
    /// `r_lodCurveError->value`. The grid LOD arm divides the view-space error
    /// by this to pick the row and column step. Retail default `250`.
    pub lod_curve_error: f32,
    /// The shader backend the world pass draws with (DEC-37 ruling 5). A zero
    /// value picks the faithful backend, the parity reference. A non-zero value
    /// picks the PBR uber-shader. Retail default `0`. The harness rides an F9
    /// toggle and a `--pbr` boot flag on this field.
    pub pbr: i32,
}

impl RenderCvarSnapshot {
    /// Reads the live cvar table once, at the frame boundary.
    ///
    /// `pbr` has no cvar handle yet, so it keeps its retail default. The
    /// harness drives that field from its own F9 toggle instead.
    pub fn from_cvars(cvars: &RendererCvars, common: &Common) -> RenderCvarSnapshot {
        RenderCvarSnapshot {
            skip_back_end: common.cvar(cvars.r_skipBackEnd).integer,
            fastsky: common.cvar(cvars.r_fastsky).integer,
            lod_curve_error: common.cvar(cvars.r_lodCurveError).value,
            pbr: 0,
        }
    }
}

impl Default for RenderCvarSnapshot {
    /// The retail cvar defaults, for a caller without a live cvar table.
    fn default() -> RenderCvarSnapshot {
        RenderCvarSnapshot {
            skip_back_end: 0,
            fastsky: 0,
            lod_curve_error: 250.0,
            pbr: 0,
        }
    }
}
