//! `ShaderAsset` — the shader registry's arena payload, plus its handle alias.

use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;

use crate::render_state::handle::Handle;
use crate::render_state::placeholders::SkyParms;
use crate::render_state::shader_stage::ShaderStage;
use crate::tr_shader::{CullType, FogParms};

/// The owned form of Raven `shader_t` — `RenderAssets::shaders`' element
/// (`R2-D3`), in the shape the tier-2 transition audit assigns (`name` →
/// `String`, `stages`/`deforms` → owned `Vec`s, `remappedShader` →
/// `Handle<ShaderAsset>`, the intrusive `next` chain dissolved). The fields
/// below are real (landed with the `tr_shader` R3 wave-0, `stages` added by
/// the `RB_RotatePic`/`RB_RotatePic2` wave, `fog_parms` by the waves-7-13 fix
/// round, `time_offset`/`remapped_shader` by campaign #41 batch 1);
/// `deforms` and the remaining scalars land with the later `tr_shader` waves
/// that read them.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:459-530`
// `Default` = the zeroed `shader_t` Raven's registry starts every slot from
// (`Com_Memset` in `R_Init`) — the pre-`CreateInternalShaders` slot-0
// placeholder the A12 constructor needs (harness boot support).
#[derive(Clone, Default)]
pub struct ShaderAsset {
    /// `name` — game path, including extension.
    pub name: String,
    /// `lightmapIndex[MAXLIGHTMAPS]`.
    pub lightmap_index: [i32; MAXLIGHTMAPS],
    /// `styles[MAXLIGHTMAPS]`.
    pub styles: [u8; MAXLIGHTMAPS],
    /// `sort` — lower numbered shaders draw before higher numbered.
    pub sort: f32,
    /// `sortedIndex` — this shader's index in `RenderAssets::sorted_shaders`.
    pub sorted_index: i32,
    /// `cullType` — which sides of the surface the renderer culls
    /// (`CT_FRONT_SIDED`/`CT_BACK_SIDED`/`CT_TWO_SIDED`). Read by
    /// `R_CullSurface`'s backface test; captured by `ParseShader`'s `cull`
    /// keyword and copied here by `GeneratePermanentShader`.
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:2507-2530` (parse),
    /// `oracle/codemp/renderer/tr_world.cpp:158,264` (read)
    pub cull_type: CullType,
    /// `surfaceFlags` — if explicitlyDefined, this will have `SURF_*` flags.
    pub surface_flags: i32,
    /// `contentFlags`.
    pub content_flags: i32,
    /// `multitextureEnv` — 0, `GL_MODULATE`, `GL_ADD` (`FUNC_ADD`), or
    /// `GL_DECAL`.
    pub multitexture_env: i32,
    /// `defaultShader` — set if the shader failed to load.
    pub default_shader: bool,
    /// `explicitlyDefined` — set if the shader came from a `.shader` file
    /// rather than a default-generated one.
    pub explicitly_defined: bool,
    /// `numUnfoggedPasses` — Raven's `short`, widened to `i32`: the renderer
    /// interior is layout-free (DEC-37 ruling 1) and the count is bounded by
    /// `MAX_SHADER_STAGES`.
    pub num_unfogged_passes: i32,
    /// `sky` (`skyParms_t *`) — owned inline, `None` when the shader has no
    /// sky parms.
    pub sky: Option<SkyParms>,
    /// `fogParms` (`fogParms_t *`, `Hunk_Alloc`'d in the oracle) — owned
    /// inline, `None` when the shader declared no `fogParms` keyword. Read by
    /// `tr_bsp`'s `R_LoadFogs`.
    ///
    /// Type definition source: `oracle/codemp/renderer/tr_local.h:488`
    pub fog_parms: Option<FogParms>,
    /// `stages[MAX_SHADER_STAGES]` (`shaderStage_t *`, `Hunk_Alloc`ed to
    /// `numUnfoggedPasses` entries) — owned inline as `Vec<ShaderStage>`.
    pub stages: Vec<ShaderStage>,
    /// `timeOffset` — Raven: current time offset for this shader
    /// (`R_RemapShader`'s third argument, parsed with `atof`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:511`
    pub time_offset: f32,
    /// `remappedShader` (`struct shader_s *`) — Raven: current shader this
    /// one is remapped too. The oracle's self-pointer becomes a handle into
    /// the same registry (tier-2 transition audit); `None` for the
    /// overwhelmingly common not-remapped case, where the oracle stores a
    /// self-pointer.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:528`
    pub remapped_shader: Option<ShaderHandle>,
}

/// A generation-counted handle into `RenderAssets::shaders` (A2).
pub type ShaderHandle = Handle<ShaderAsset>;
