//! `FrameEvent` — one ordered per-frame draw/scene command (`R2-D2`/A1).

use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;

use crate::render_state::placeholders::{Poly, PolyVert, RefEntity, TrRefdef, Vec3};
use crate::render_state::shader_asset::ShaderHandle;

/// The typed replacement for the oracle's byte-packed `renderCommandList_t`
/// (`oracle/codemp/renderer/tr_local.h:2180-2250`): the traps that **mutate
/// ordered per-frame draw/scene state**, in trap-call order. Traps that are
/// synchronous, non-event calls (registration, bounds queries, glconfig, PVS,
/// lighting queries, font metrics, light styles, `distanceCull`, the automap
/// wireframe rebuild, the weather-contents-override no-op) are **not** events
/// — per ruling 3 they stay direct calls against `Arc<RenderAssets>` or
/// `RenderAssetsSim` (`### FrameData`).
///
/// Deliberately absent: `CG_R_WEATHER_CONTENTS_OVERRIDE` (retail's handler is
/// a live no-op, `oracle/codemp/client/cl_cgame.cpp:1716-1718`, B9),
/// `AddMiniRefEntityToScene` (no trap call site; ruling 13), and
/// `RC_END_OF_LIST` (the `Vec`'s length is the terminator).
pub enum FrameEvent {
    // --- scene composition (CG_R_CLEARSCENE / UI_R_CLEARSCENE, etc.) ---
    /// `CG_R_/UI_R_CLEARSCENE`.
    ClearScene,
    /// `CG_R_CLEARDECALS` — cgame-only, no UI trap.
    ClearDecals,
    /// `CG_R_/UI_R_ADDREFENTITYTOSCENE`.
    AddRefEntityToScene(RefEntity),
    /// `CG_R_/UI_R_ADDPOLYTOSCENE`.
    ///
    /// Source: `oracle/codemp/renderer/tr_public.h:55`
    AddPolyToScene {
        shader: ShaderHandle,
        verts: Vec<PolyVert>,
        /// `poly_t::fogIndex` — the fog volume the poly falls in, resolved at
        /// trap time by `RE_AddPolyToScene`.
        ///
        /// Source: `oracle/codemp/renderer/tr_scene.cpp:151-179`
        fog_index: i32,
    },
    /// `CG_R_ADDPOLYSTOSCENE` — cgame-only.
    AddPolysToScene {
        shader: ShaderHandle,
        polys: Vec<Poly>,
    },
    /// `CG_R_/UI_R_ADDLIGHTTOSCENE`.
    AddLightToScene {
        org: Vec3,
        intensity: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    /// `CG_R_ADDADDITIVELIGHTTOSCENE` — cgame-only.
    AddAdditiveLightToScene {
        org: Vec3,
        intensity: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    /// `CG_R_ADDDECALTOSCENE` — cgame-only.
    ///
    /// Source: `oracle/codemp/renderer/tr_public.h:56`;
    /// `oracle/codemp/client/cl_cgame.cpp:903-904`
    AddDecalToScene {
        shader: ShaderHandle,
        origin: Vec3,
        dir: Vec3,
        orientation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        alpha_fade: bool,
        radius: f32,
        temporary: bool,
    },
    /// `CG_R_SETRANGEFOG` — table-bypass write to `tr.rangedFog`.
    ///
    /// Source: `oracle/codemp/client/cl_cgame.cpp:943-945`
    SetRangeFog(f32),
    /// `CG_R_SETREFRACTIONPROP` — table-bypass write to
    /// `tr_distortionAlpha`/`Stretch`/`PrePost`/`Negate`.
    ///
    /// Source: `oracle/codemp/client/cl_cgame.cpp:947-952`
    SetRefractionProp {
        alpha: f32,
        stretch: f32,
        pre_post: bool,
        negate: bool,
    },
    /// `CG_R_/UI_R_RENDERSCENE` — seals the accumulated scene. `light_styles`
    /// (A11) is the operational form of A6's snapshot-at-scene-marks: the sim
    /// thread copies `LightStyleTable::colors` into the event so the
    /// render-side consumers read the frame's snapshot, not the live sim-owned
    /// table. R3 caveat (`R2-D5`): snapshot-vs-live timing verifies against
    /// the oracle when the backend consumer lands.
    RenderScene {
        refdef: TrRefdef,
        light_styles: [[u8; 4]; MAX_LIGHT_STYLES],
        /// The oracle's `tr.refdef.num_dlights = 0` disable decision
        /// (`r_dynamiclight->integer == 0 || r_vertexLight->integer == 1`).
        /// `num_dlights` has no `TrRefdef` field because the render side
        /// replays dlights from `AddLightToScene` events, so this flag carries
        /// the disable to the replay, which drops the frame's dlight set.
        ///
        /// Source: `oracle/codemp/renderer/tr_scene.cpp:817-822`
        disable_dynamic_light: bool,
    },

    // --- 2D draw commands (RC_SET_COLOR / RC_STRETCH_PIC / RC_ROTATE_PIC family) ---
    /// `CG_R_/UI_R_SETCOLOR`.
    SetColor([f32; 4]),
    /// `CG_R_/UI_R_DRAWSTRETCHPIC`.
    DrawStretchPic {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        shader: ShaderHandle,
    },
    /// `CG_R_DRAWROTATEPIC` — cgame-only.
    ///
    /// Source: `oracle/codemp/renderer/tr_public.h:67-68`
    DrawRotatePic {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        angle: f32,
        shader: ShaderHandle,
    },
    /// `CG_R_DRAWROTATEPIC2` — cgame-only; same fields as `DrawRotatePic`.
    ///
    /// Source: `oracle/codemp/renderer/tr_public.h:69-70`
    DrawRotatePic2 {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        angle: f32,
        shader: ShaderHandle,
    },
    /// `CG_R_/UI_R_FONT_DRAWSTRING`. `set_index` is the registered font's
    /// handle, `text` the owned Latin-1 string replacing the oracle's
    /// `const char *`, `rgba` its four-float colour by value.
    ///
    /// Source: `oracle/codemp/renderer/tr_public.h:97`
    DrawString {
        ox: i32,
        oy: i32,
        text: String,
        rgba: [f32; 4],
        set_index: i32,
        char_limit: i32,
        scale: f32,
    },

    // --- world-effects / automap tail (RC_WORLD_EFFECTS / RC_AUTO_MAP) ---
    /// `CG_R_WORLDEFFECTCOMMAND` — cgame-only.
    ///
    /// Source: `oracle/codemp/client/cl_cgame.cpp:1720-1722`
    WorldEffectCommand(String),
    /// `CG_R_AUTOMAPELEVADJ` — cgame-only; drives `g_playerHeight`.
    ///
    /// Source: `oracle/codemp/client/cl_cgame.cpp:1075-1077`
    AutomapElevAdj(f32),
}
