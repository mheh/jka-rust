//! `WalkWarnings` — the once-per-process latches the render-side walk prints
//! through (W2-F1).

/// One latch per diagnostic the world and entity walk used to print through
/// `Com_Printf`/`Com_DPrintf`.
///
/// W2-F1 moved the walk to the render thread, which holds no `Common`, so each
/// of these prints once through `eprintln!` and then latches. The oracle's own
/// prints are either developer-gated or per-occurrence, and a per-frame repeat
/// on the render thread would flood the console (`frame_exec`'s warn-once
/// precedent).
#[derive(Default)]
pub struct WalkWarnings {
    /// `R_CullSurface`'s roof cull needs three collision traces the render
    /// thread cannot run, so W2-F2 leaves the feature inert and says so.
    pub roof_cull: bool,
    /// `R_GetShaderByHandle`'s out-of-range diagnostic.
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:3800-3810`
    pub shader_handle: bool,
    /// `R_AddMD3Surfaces`' "no such frame" diagnostic.
    ///
    /// Source: `oracle/codemp/renderer/tr_mesh.cpp:311-318`
    pub md3_bad_frame: bool,
    /// `R_AddMD3Surfaces`' "no shader for surface in skin" diagnostic.
    ///
    /// Source: `oracle/codemp/renderer/tr_mesh.cpp:365-368`
    pub md3_skin_surface: bool,
    /// `R_AddMD3Surfaces`' "shader in skin not found" diagnostic.
    ///
    /// Source: `oracle/codemp/renderer/tr_mesh.cpp:370-373`
    pub md3_skin_shader: bool,
    /// `R_MirrorViewBySurface`'s recursive-portal refusal.
    ///
    /// Source: `oracle/codemp/renderer/tr_main.cpp:1112-1113`
    pub recursive_portal: bool,
}
