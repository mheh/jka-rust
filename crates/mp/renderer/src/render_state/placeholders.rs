//! Landing placeholders for the names `## Seam definition` lists as "named but
//! not defined here — they land with whichever crate owns them": the
//! CPU/frontend names at R3, the GPU-facing ones at R4.
//!
//! Each type below is a shape the R3/R4 wave that owns it fills in and moves
//! to its own file (house one-type-per-file convention); they exist here so
//! the root types this module freezes — `RenderAssets`, `FrameState`,
//! `FrameEvent`, `GpuResources` — compile at R3 skeleton time without any wave
//! inventing their interiors early. Every one of them is bound by the
//! interior-safety law: owned fields, handles and indices only.
//!
//! Quarantine still holds, but several are no longer empty: the R3 wave-0
//! transcription landed the fields it reads on `RefEntity`, `TrRefdef`,
//! `WorldAsset`, `FunctionTables` and `GlConfig` (each type's doc comment says
//! which set is real and which wave lands the rest), and campaign #41 batch 1
//! filled `OrientationR` outright (all four oracle fields are value arrays).
//! `Poly`, `ViewParms`, `BackEndCounters`, `SkyParms`, `AutomapWireframe` and
//! `GlStatePlaceholder` are untouched by wave-0 and stay empty.
//!
//! Two exceptions carry a real shape already, because the oracle payload they
//! stand for is itself pointer-free and already ported at the seam: `Vec3` and
//! `PolyVert` alias `mp_qshared`'s `vec3_t`/`polyVert_t` rather than duplicate
//! them.

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::{cplane_t, qhandle_t, vec3_t};

use crate::tr_bsp::{BModel, DShader, Fog, Node};
use crate::tr_local::mgrid_t::mgrid_t;

/// `FUNCTABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
pub(crate) const FUNCTABLE_SIZE: usize = 1024;

/// `FOG_TABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1246`
pub(crate) const FOG_TABLE_SIZE: usize = 256;

/// Raven `vec3_t` under the design's `Vec3` spelling — the vector payload
/// `FrameEvent`'s light/decal variants carry.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:530-537`
pub type Vec3 = vec3_t;

/// Raven `polyVert_t` — the vertex payload `FrameEvent`'s poly/decal variants
/// carry, owned in a `Vec` instead of the oracle's `polyVert_t *`.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:71-75`
pub type PolyVert = polyVert_t;

/// One polygon of a `CG_R_ADDPOLYSTOSCENE` batch — the owned form of Raven
/// `poly_t`, its `verts` pointer replaced by an owned vertex list and its
/// `hShader` hoisted onto the event variant (`FrameEvent::AddPolysToScene`).
/// Fields land with the `tr_scene` R3 wave.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:77-81`
#[derive(Clone)]
pub struct Poly {}

/// The owned form of Raven `trRefEntity_t`'s scene-entity payload — what
/// `FrameEvent::AddRefEntityToScene` carries and what `FrameState`'s
/// `current_entity`/`entity_2d` hold by value (`## Seam definition`). Not the
/// tier-1 `refEntity_t` (its `*mut c_void` ghoul2 tail is forbidden interior).
///
/// The embedded `refEntity_t e` is flattened onto this struct rather than
/// nested under an `.e` sub-field. The fields below are real (landed with the
/// `tr_scene`/`tr_light`/`tr_shade_calc` R3 wave-0); the rest of
/// `refEntity_t`/`trRefEntity_t` lands with the waves that read it.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:94-106`
/// (`refEntity_t`: `oracle/codemp/cgame/tr_types.h:135-231`)
#[derive(Clone)]
pub struct RefEntity {
    /// `e.reType`.
    pub re_type: refEntityType_t,
    /// `e.renderfx`.
    pub renderfx: i32,
    /// `e.hModel` — kept as the oracle's raw index, not `Handle<ModelAsset>`:
    /// zero means "unset" here, where `Handle{0,0}` means the registry's live
    /// default entry (A12).
    pub h_model: qhandle_t,
    /// `e.axis[3]` — rotation vectors.
    pub axis: [Vec3; 3],
    /// `e.origin`.
    pub origin: Vec3,
    /// `e.oldorigin`.
    pub old_origin: Vec3,
    /// `e.customShader` — raw index, same reasoning as `h_model`.
    pub custom_shader: qhandle_t,
    /// `e.shaderRGBA`.
    pub shader_rgba: [u8; 4],
    /// `e.radius` — extra sprite information.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:158`
    pub radius: f32,
    /// `e.rotation` — extra sprite information.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:159`
    pub rotation: f32,
    /// `e.frame` — Raven: also used as `MODEL_BEAM`'s diameter, and as the
    /// `Q_random`/`Q_crandom` seed the lightning surfaces step through.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:163`
    pub frame: i32,
    /// `e.lightingOrigin`.
    pub lighting_origin: Vec3,
    /// `e.endTime`.
    pub end_time: f32,
    /// `e.saberLength`.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:238`
    pub saber_length: f32,
    /// `e.angles` — Raven: rotation angles - used for Ghoul2.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:243`
    pub angles: Vec3,
    /// `e.modelScale` — Raven: axis scale for models.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:245`
    pub model_scale: Vec3,
    /// `e.ghoul2 != NULL` — a presence flag, not the pointer: the tier-1
    /// `*mut c_void` tail is forbidden interior.
    pub has_ghoul2: bool,
    /// `needDlights` — Raven: true for bmodels that touch a dlight.
    pub need_dlights: bool,
    /// `lightingCalculated`.
    pub lighting_calculated: bool,
    /// `lightDir` — normalized direction towards light.
    pub light_dir: Vec3,
    /// `ambientLight` — color normalized to 0-255.
    pub ambient_light: Vec3,
    /// `ambientLightInt` — Raven: 32 bit rgba packed. Retail writes it
    /// through `((byte *)&ent->ambientLightInt)[N]`, a reinterpret-cast the
    /// interior-safety law forbids; carried as the unpacked `color4ub_t` the
    /// byte writes address instead.
    pub ambient_light_int: [u8; 4],
    /// `directedLight`.
    pub directed_light: Vec3,
    /// `dlightBits`.
    pub dlight_bits: i32,
}

// `refEntityType_t` has no `Default`, so `RefEntity`'s cannot be derived;
// `RT_MODEL = 0` is the zero-initialized oracle value.
impl Default for RefEntity {
    fn default() -> Self {
        Self {
            re_type: refEntityType_t::RT_MODEL,
            renderfx: 0,
            h_model: 0,
            axis: [[0.0; 3]; 3],
            origin: [0.0; 3],
            old_origin: [0.0; 3],
            custom_shader: 0,
            shader_rgba: [0; 4],
            radius: 0.0,
            rotation: 0.0,
            frame: 0,
            lighting_origin: [0.0; 3],
            end_time: 0.0,
            saber_length: 0.0,
            angles: [0.0; 3],
            model_scale: [0.0; 3],
            has_ghoul2: false,
            need_dlights: false,
            lighting_calculated: false,
            light_dir: [0.0; 3],
            ambient_light: [0.0; 3],
            ambient_light_int: [0; 4],
            directed_light: [0.0; 3],
            dlight_bits: 0,
        }
    }
}

/// The owned form of Raven `trRefdef_t` — the scene description
/// `FrameEvent::RenderScene` carries and `FrameState::refdef` holds
/// render-side. Independently shaped from `refdef_t` (`R2-D6`); the oracle's
/// array pointers become owned `Vec`s on the event, `areamaskModified` a
/// `bool`, `text` a `Vec<String>`. The fields below are real (landed with the
/// `tr_backend` R3 wave-0); the rest lands with the `tr_scene` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:563-598`
#[derive(Clone)]
pub struct TrRefdef {
    /// `fov_x`.
    pub fov_x: f32,
    /// `fov_y`.
    pub fov_y: f32,
    /// `vieworg`.
    pub view_origin: Vec3,
    /// `viewaxis[3]` — the view transformation matrix.
    pub view_axis: [Vec3; 3],
}

/// The owned form of Raven `viewParms_t` — `FrameState::view`. Its
/// `frustum[N]` count is mode-fixed (`FRUSTUM_PLANES` = 4 MP, 5 SP,
/// `R2-D7`(a)) and `isPortal`/`isMirror` become `bool`; fields land with the
/// `tr_main` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:629-644`
#[derive(Clone)]
pub struct ViewParms {}

/// The owned form of Raven `orientationr_t` — `FrameState::ori` (`R2-D7`(b):
/// Rust spells it `ori` on both modes). All four oracle fields are plain
/// value arrays, so the whole struct lands owned.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:109-114`
#[derive(Clone, Default)]
pub struct OrientationR {
    /// `origin` — Raven: in world coordinates.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:110`
    pub origin: Vec3,
    /// `axis[3]` — Raven: orientation in world.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:111`
    pub axis: [Vec3; 3],
    /// `viewOrigin` — Raven: `viewParms->or.origin` in local coordinates.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:112`
    pub view_origin: Vec3,
    /// `modelMatrix[16]`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:113`
    pub model_matrix: [f32; 16],
}

/// The owned form of Raven `backEndCounters_t` — `FrameState::counters`
/// (`backEnd.pc`). Fields land with the R4 backend wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1263-1275`
#[derive(Clone)]
pub struct BackEndCounters {}

/// The owned form of Raven `world_t` — `RenderAssets::world` and each entry of
/// `RenderAssets::bsp_models`, replaced wholesale on level load. Every oracle
/// pointer array becomes an owned `Vec` and the node/leaf links become arena
/// indices (tier-2 transition audit, group 1). The fields below are real
/// (landed with the `tr_bsp`/`tr_light` R3 wave-0); the node/surface/bmodel/
/// fog arrays land with the rest of the `tr_bsp`/`tr_world` waves.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1039-1090`
#[derive(Clone, Default)]
pub struct WorldAsset {
    /// `name` — ie: `maps/tim_dm2.bsp`.
    pub name: String,
    /// `shaders` (`dshader_t *`) — the on-disk shader-reference lump.
    pub shaders: Vec<DShader>,
    /// `bmodels` (`bmodel_t *`).
    pub bmodels: Vec<BModel>,
    /// `planes` (`cplane_t *`).
    pub planes: Vec<cplane_t>,
    /// `numDecisionNodes`.
    pub num_decision_nodes: i32,
    /// `nodes` (`mnode_t *`) — the node/leaf arena `Node`'s `parent`/
    /// `children` indices point into (`numnodes` is `nodes.len()`).
    pub nodes: Vec<Node>,
    /// `marksurfaces` (`msurface_t **`) — surface **indices**, per the tier-2
    /// transition audit's pointer-array replacement.
    pub mark_surfaces: Vec<u32>,
    /// `lightGridOrigin`.
    pub light_grid_origin: Vec3,
    /// `lightGridSize`.
    pub light_grid_size: Vec3,
    /// `lightGridInverseSize`.
    pub light_grid_inverse_size: Vec3,
    /// `lightGridBounds[3]`.
    pub light_grid_bounds: [i32; 3],
    /// `lightGridData` (`mgrid_t *`) — `None` when the lump size mismatches
    /// the grid bounds (`R_LoadLightGridArray`'s warning path).
    pub light_grid_data: Option<Vec<mgrid_t>>,
    /// `lightGridArray` (`word *`).
    pub light_grid_array: Vec<u16>,
    /// `numGridArrayElements`.
    pub num_grid_array_elements: i32,
    /// `numClusters`.
    pub num_clusters: i32,
    /// `clusterBytes`.
    pub cluster_bytes: i32,
    /// `vis` (`const byte *`) — owned here rather than shared with
    /// `CM_LoadMap`'s buffer.
    pub vis: Vec<u8>,
    /// `novis` (`byte *`) — `clusterBytes` of `0xff`.
    pub novis: Vec<u8>,
    /// `entityString` (`char *`).
    pub entity_string: String,
    /// `entityParsePoint` (`char *`) — a byte offset into `entity_string`,
    /// not a pointer into it (interior-safety law).
    pub entity_parse_point: usize,
    /// `baseName` — ie: `tim_dm2`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:987`
    pub base_name: String,
    /// `globalFog` — index into [`Self::fogs`], `-1` when the map has none.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1012`
    pub global_fog: i32,
    /// `fogs` (`fog_t *`) — `numfogs` collapses to `fogs.len()` (wave-8
    /// field merge; see `tr_bsp.rs`'s WAVE 8 ADDITIONS note).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1010-1013`
    pub fogs: Vec<Fog>,
}

/// `tr`'s precomputed wave-function tables (`sinTable`, `squareTable`,
/// `triangleTable`, `sawToothTable`, `inverseSawToothTable`, `fogTable`) —
/// `RenderAssets::function_tables`, filled once at renderer init. All six
/// fields are real (landed with the `tr_image`/`tr_light`/`tr_shade_calc` R3
/// wave-0).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1412-1417`
#[derive(Clone)]
pub struct FunctionTables {
    /// `sinTable`.
    pub sin_table: [f32; FUNCTABLE_SIZE],
    /// `squareTable`.
    pub square_table: [f32; FUNCTABLE_SIZE],
    /// `triangleTable`.
    pub triangle_table: [f32; FUNCTABLE_SIZE],
    /// `sawToothTable`.
    pub saw_tooth_table: [f32; FUNCTABLE_SIZE],
    /// `inverseSawToothTable`.
    pub inverse_saw_tooth_table: [f32; FUNCTABLE_SIZE],
    /// `fogTable`.
    pub fog_table: [f32; FOG_TABLE_SIZE],
}

/// The owned form of Raven `glconfig_t` — `RenderAssets::glconfig`,
/// sim-readable because `CG_R_GETREALRES` reads `vidWidth`/`vidHeight`
/// synchronously (B11). Not the tier-1 `glconfig_t` (its `c_char` renderer/
/// vendor/extension strings are forbidden interior — they become `String`s
/// here, its `qboolean`s become `bool`s). The full `glconfig_t` field set is
/// real (landed with the `tr_init`/`tr_image`/`tr_backend` R3 wave-0); the
/// tier-1 struct the `CG_GETGLCONFIG`/`UI_GETGLCONFIG` traps marshal is
/// `mp_qshared::common::mp::cgame::glconfig_t::glconfig_t`, converted at the
/// seam, never held here.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:299-325`
#[derive(Clone)]
pub struct GlConfig {
    /// `renderer_string`.
    pub renderer_string: String,
    /// `vendor_string`.
    pub vendor_string: String,
    /// `version_string`.
    pub version_string: String,
    /// `extensions_string`.
    pub extensions_string: String,
    /// `maxTextureSize` — queried from GL.
    pub max_texture_size: i32,
    /// `maxActiveTextures` — multitexture ability.
    pub max_active_textures: i32,
    /// `maxTextureFilterAnisotropy`.
    pub max_texture_filter_anisotropy: f32,
    /// `colorBits`.
    pub color_bits: i32,
    /// `depthBits`.
    pub depth_bits: i32,
    /// `stencilBits`.
    pub stencil_bits: i32,
    /// `deviceSupportsGamma`.
    pub device_supports_gamma: bool,
    /// `textureCompression`.
    pub texture_compression: textureCompression_t,
    /// `textureEnvAddAvailable`.
    pub texture_env_add_available: bool,
    /// `clampToEdgeAvailable`.
    pub clamp_to_edge_available: bool,
    /// `vidWidth`.
    pub vid_width: i32,
    /// `vidHeight`.
    pub vid_height: i32,
    /// `displayFrequency`.
    pub display_frequency: i32,
    /// `isFullscreen`.
    ///
    /// Raven: synonymous with "does rendering consume the entire screen?".
    pub is_fullscreen: bool,
    /// `stereoEnabled`.
    pub stereo_enabled: bool,
}

// `textureCompression_t` has no `Default`, so `GlConfig`'s cannot be derived;
// `TC_NONE = 0` is the zero-initialized oracle value.
impl Default for GlConfig {
    fn default() -> Self {
        Self {
            renderer_string: String::new(),
            vendor_string: String::new(),
            version_string: String::new(),
            extensions_string: String::new(),
            max_texture_size: 0,
            max_active_textures: 0,
            max_texture_filter_anisotropy: 0.0,
            color_bits: 0,
            depth_bits: 0,
            stencil_bits: 0,
            device_supports_gamma: false,
            texture_compression: textureCompression_t::TC_NONE,
            texture_env_add_available: false,
            clamp_to_edge_available: false,
            vid_width: 0,
            vid_height: 0,
            display_frequency: 0,
            is_fullscreen: false,
            stereo_enabled: false,
        }
    }
}

/// The owned form of Raven `skyParms_t` — `ShaderAsset::sky`, owned inline
/// instead of the oracle's `skyParms_t *`. Its `outerbox` becomes
/// `[ImageHandle; 6]` per the tier-2 transition audit (`skyParms_t` row);
/// fields land with the `tr_sky` R3 wave, the first that reads one — wave-0
/// `tr_shader` only tests the option for presence.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:449-452`
#[derive(Clone)]
pub struct SkyParms {}

/// The wireframe automap surface list — `RenderAssets::automap_wireframe`,
/// rebuilt by `RenderAssetsSim::rebuild_automap_wireframe` (A10/`R2-D10`),
/// replacing `g_autoMapFrame`/`g_autoMapValid`. Fields land with the first
/// automap R3/R4 wave, which also settles `RC_AUTO_MAP`'s command shape
/// (`R2-D8`).
///
/// Source: `oracle/codemp/renderer/tr_world.cpp:782,784`
#[derive(Clone)]
pub struct AutomapWireframe {}

/// `GpuResources::gl_state` — the named placeholder standing in for Raven
/// `glstate_t` (B6/`R2-D1`): the GL binding cache has no meaning under wgpu,
/// so it holds nothing until R4 defines the real pipeline/bind-group cache.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1253-1260`
#[derive(Clone)]
pub struct GlStatePlaceholder {}
