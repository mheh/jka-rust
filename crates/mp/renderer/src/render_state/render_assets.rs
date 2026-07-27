//! `RenderAssets` — the CPU-side, `Arc`-shared, sim-readable registry root
//! (`R2-D1`/`R2-D3`/`R2-D4`).

use std::collections::HashMap;

use crate::render_state::arena::Arena;
use crate::render_state::image_asset::{ImageAsset, ImageHandle};
use crate::render_state::model_asset::{ModelAsset, ModelHandle};
use crate::render_state::placeholders::{AutomapWireframe, FunctionTables, GlConfig, WorldAsset};
use crate::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use crate::render_state::skin_asset::{SkinAsset, SkinHandle};

/// The registries `trGlobals_t` used to hold, plus the session state a
/// synchronous trap has to reach: CPU-only, immutable-after-publish,
/// `Arc`-shared, sim-readable (ruling 3). Mutation goes through
/// `RenderAssetsSim` (A9).
///
/// `#[derive(Clone)]` — required by `Arc::make_mut(&mut RenderAssetsSim
/// ::published)` (A9/NB-1); every field type derives `Clone` in turn.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1309-1423,1434`
#[derive(Clone)]
pub struct RenderAssets {
    /// Unbounded (A5) — mirrors `tr_image.cpp`'s `AllocatedImages` std::map
    /// backing store; no `MAX_DRAWIMAGES` soft-cap, no slot-0 reservation.
    pub images: Arena<ImageAsset>,
    /// The oracle's `AllocatedImages` name→ptr map, keyed by the lower-cased,
    /// extension-stripped name (`GenerateImageMappingName`,
    /// `oracle/codemp/renderer/tr_image.cpp:1287-1289`).
    pub image_names: HashMap<String, ImageHandle>,
    /// `tr.lightmaps[MAX_LIGHTMAPS]` — `image_t*` in the oracle, folded into
    /// `images` rather than a fifth arena; this is the **positional** index
    /// `R_FindShader` reads by small integer
    /// (`oracle/codemp/renderer/tr_shader.cpp:3543`), populated at level load
    /// in lightmap order (`R2-D4`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1364`
    pub lightmaps: Vec<ImageHandle>,
    /// Soft-capped at `MAX_SHADERS = 16384`; slot 0 pre-populated with
    /// `tr.defaultShader`; overflow warns and returns `Handle{0,0}` — that
    /// same default (A5, A5 amendment, A12).
    pub shaders: Arena<ShaderAsset>,
    /// The `IsShader` bucket walk (`oracle/codemp/renderer/
    /// tr_shader.cpp:3373-3398`): a stripped name maps to every candidate
    /// sharing it, compared per-entry against the full
    /// `lightmapIndex`/`styles` arrays with the `if (!sh->defaultShader)`
    /// short-circuit (`:3382`) — a plain name→handle map cannot represent it
    /// (`R2-D4`).
    pub shader_lookup: HashMap<String, Vec<ShaderHandle>>,
    /// `tr.sortedShaders[MAX_SHADERS]` — shaders in draw order, maintained by
    /// `SortNewShader` and walked by `R_ShaderList_f`'s `Cmd_Argc() > 1`
    /// branch. Owned handles, not the oracle's `shader_t *` array.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1407`
    pub sorted_shaders: Vec<ShaderHandle>,
    /// `s_shaderText` — the concatenated `.shader` file text, parsed on demand
    /// by `FindShaderInShaderText` (A13.4).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp` (`s_shaderText`)
    pub shader_text: String,
    /// `shaderTextHashTable[MAX_SHADERTEXT_HASH]` — one bucket per shader-name
    /// hash, each holding byte offsets into `shader_text` where a shader's
    /// text begins (the oracle's `char **` bucket of pointers into
    /// `s_shaderText`, indices per the interior-safety law) (A13.4).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp` (`shaderTextHashTable`)
    pub shader_text_hash_table: Vec<Vec<usize>>,
    /// `deferLoad` — the shader-load deferral flag riding with the shader-text
    /// cache (A13.4).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp` (`deferLoad`)
    pub defer_load: bool,
    /// Soft-capped at `MAX_SKINS = 1024`; slot 0 pre-populated with
    /// `"<default skin>"`; overflow warns and returns `Handle{0,0}` (A5, A5
    /// amendment, A12).
    pub skins: Arena<SkinAsset>,
    /// `RE_RegisterSkin`'s name walk (`oracle/codemp/renderer/
    /// tr_image.cpp:3128-3136`) compares the full name only — plain
    /// name→handle (`R2-D4`).
    pub skin_lookup: HashMap<String, SkinHandle>,
    /// Soft-capped at `MAX_MOD_KNOWN = 1024`; slot 0 pre-populated with
    /// `MOD_BAD`; overflow is silent in retail, this port adds a marked
    /// warning, returns `Handle{0,0}` (A5, A5 amendment, A12).
    pub models: Arena<ModelAsset>,
    /// `RE_RegisterModel`'s `mhHashTable` walk (`oracle/codemp/renderer/
    /// tr_model.cpp:1211-1215`) — plain name→handle, same shape as
    /// `skin_lookup`, kept separate for per-kind handle typing (`R2-D4`).
    pub model_lookup: HashMap<String, ModelHandle>,
    /// `tr.world` — replaced wholesale on level load.
    pub world: Option<WorldAsset>,
    /// `tr.bspModels[MAX_SUB_BSP]` — sub-BSP worlds, homed beside `world`
    /// rather than a fifth arena.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1399`
    pub bsp_models: Vec<WorldAsset>,
    /// `tr`'s wave-function tables.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1412-1417`
    pub function_tables: FunctionTables,
    /// `tr.distanceCull` — sim-readable because `CG_R_GETDISTANCECULL` reads
    /// it synchronously (B11).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1420`
    pub distance_cull: f32,
    /// `tr.distanceCullSquared`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1420`
    pub distance_cull_squared: f32,
    /// `glConfig` — sim-readable because `CG_R_GETREALRES` reads
    /// `vidWidth`/`vidHeight` synchronously (B11).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1435`
    pub glconfig: GlConfig,
    /// `tr.registered` ("cleared at shutdown, set at beginRegistration") — the
    /// guard every `RE_Add*ToScene` reads first
    /// (`oracle/codemp/renderer/tr_scene.cpp:195-197`). A session flag, not
    /// per-frame scratch (`R2-D2`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1310`
    pub registered: bool,
    /// `max_polys` — the per-frame poly append bound, `r_maxpolys`' value
    /// (default `MAX_POLYS = 600`). Session/capacity state, sim-side
    /// (`### FrameData`'s append-validation principle).
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1285`;
    /// `oracle/codemp/renderer/tr_local.h:2256`
    pub max_polys: usize,
    /// `max_polyverts` — the per-frame poly-vertex append bound,
    /// `r_maxpolyverts`' value (default `MAX_POLYVERTS = 3000`).
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1289`;
    /// `oracle/codemp/renderer/tr_local.h:2257`
    pub max_polyverts: usize,
    /// Wireframe automap data — rebuilt sim-side by
    /// `RenderAssetsSim::rebuild_automap_wireframe` (A10/`R2-D10`), replacing
    /// `g_autoMapFrame`/`g_autoMapValid`.
    ///
    /// Source: `oracle/codemp/renderer/tr_world.cpp:782,784,1205-1231`
    pub automap_wireframe: AutomapWireframe,
}
