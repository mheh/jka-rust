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
//! `ViewParms` and `TrRefdef` carry only the subset of fields the world
//! PVS-walk wave (`R_MarkLeaves`/`R_RecursiveWorldNode`) reads or writes; the
//! rest lands with the `tr_main`/`tr_scene` waves. `Poly`, `BackEndCounters`,
//! `SkyParms`, `AutomapWireframe` and `GlStatePlaceholder` are untouched by
//! wave-0 and stay empty.
//!
//! Two exceptions carry a real shape already, because the oracle payload they
//! stand for is itself pointer-free and already ported at the seam: `Vec3` and
//! `PolyVert` alias `mp_qshared`'s `vec3_t`/`polyVert_t` rather than duplicate
//! them.

use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::MAX_MAP_AREA_BYTES;
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::{cplane_t, qhandle_t, vec2_t, vec3_t};

use crate::render_state::image_asset::ImageHandle;
use crate::tr_bsp::{BModel, DShader, Fog, Node, Surface};
use crate::tr_local::mgrid_t::mgrid_t;

/// `FUNCTABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
pub const FUNCTABLE_SIZE: usize = 1024;

/// `FUNCTABLE_SIZE2` — Raven: `log2(FUNCTABLE_SIZE)`, the shift
/// `R_BindAnimatedImage` applies to an `animMap` frame index.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1248`
pub const FUNCTABLE_SIZE2: i32 = 10;

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
    /// `e.hModel` — kept as the oracle's raw index, not a `ModelHandle`:
    /// zero means "unset" here, where `Handle{0,0}` means the registry's live
    /// default entry (A12).
    pub h_model: qhandle_t,
    /// `e.axis[3]` — rotation vectors.
    pub axis: [Vec3; 3],
    /// `e.nonNormalizedAxes` — Raven: axis are not normalized, they have scale.
    /// `R_RotateForEntity` reads it to scale the model-space view origin.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:144`
    pub non_normalized_axes: bool,
    /// `e.origin`.
    pub origin: Vec3,
    /// `e.oldorigin`.
    pub old_origin: Vec3,
    /// `e.customShader` — raw index, same reasoning as `h_model`.
    pub custom_shader: qhandle_t,
    /// `e.shaderRGBA`.
    pub shader_rgba: [u8; 4],
    /// `e.shaderTexCoord` — Raven: texture coordinates used by tcMod entity
    /// modifiers. The `TMOD_ENTITY_TRANSLATE` texmod reads it as a scroll speed.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:120,155`
    pub shader_tex_coord: vec2_t,
    /// `e.radius` — extra sprite information.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:158`
    pub radius: f32,
    /// `e.rotation` — extra sprite information.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:159`
    pub rotation: f32,
    /// `e.shaderTime` — Raven: subtracted from refdef time to control effect
    /// start times. The draw path derives the per-entity shader clock from it.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:162`
    pub shader_time: f32,
    /// `e.frame` — Raven: also used as `MODEL_BEAM`'s diameter, and as the
    /// `Q_random`/`Q_crandom` seed the lightning surfaces step through.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:163`
    pub frame: i32,
    /// `e.oldframe` — Raven: previous frame for MD3 keyframe interpolation.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:179`
    pub old_frame: i32,
    /// `e.backlerp` — Raven: 0.0 is the current frame, 1.0 is the old frame.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:180`
    pub backlerp: f32,
    /// `e.skinNum` — Raven: inline skin index, picks the per-surface MD3
    /// shader when no custom shader or custom skin is set.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:183`
    pub skin_num: i32,
    /// `e.customSkin` — raw index into the skin registry, same reasoning as
    /// `h_model`. Zero means the default skin.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:184`
    pub custom_skin: qhandle_t,
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
    /// The entity's attached Ghoul2 instance list, decoded from the tier-1
    /// `*mut c_void ghoul2` token (`ghoul2_token_decode`, `tr_scene.rs`). Raven
    /// carries a raw `CGhoul2Info_v *`. The render side threads a
    /// `&mut Ghoul2System` and looks the list up by this `Ghoul2Handle`, so no
    /// raw pointer crosses the seam.
    pub ghoul2: Option<Ghoul2Handle>,
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

impl RefEntity {
    /// Raven's `ent->e.ghoul2 != NULL` presence test, now a live-handle test.
    pub fn has_ghoul2(&self) -> bool {
        self.ghoul2.is_some()
    }
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
            non_normalized_axes: false,
            origin: [0.0; 3],
            old_origin: [0.0; 3],
            custom_shader: 0,
            shader_rgba: [0; 4],
            shader_tex_coord: [0.0; 2],
            radius: 0.0,
            rotation: 0.0,
            shader_time: 0.0,
            frame: 0,
            old_frame: 0,
            backlerp: 0.0,
            skin_num: 0,
            custom_skin: 0,
            lighting_origin: [0.0; 3],
            end_time: 0.0,
            saber_length: 0.0,
            angles: [0.0; 3],
            model_scale: [0.0; 3],
            ghoul2: None,
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
/// render-side. Independently shaped from `refdef_t` (`R2-D6`);
/// `areamaskModified` is a `bool` and `text` a `Vec<String>`.
///
/// The four oracle count+pointer pairs (`num_entities`/`entities`,
/// `numPolys`/`polys`, `num_dlights`/`dlights`, `numDrawSurfs`/`drawSurfs`)
/// stay OUT. The render side rebuilds those sets by replaying the
/// `Add*ToScene` `FrameEvent`s, so `TrRefdef` never carries a scene list
/// (DEC-50).
///
/// `skyboxportal`/`drawskyboxportal` are not `trRefdef_t` fields. They are the
/// oracle's `tr_scene.cpp` file-scope statics, carried here so the
/// `FrameEvent::RenderScene` payload can write `FrameState::skyboxportal`/
/// `drawskyboxportal` render-side (DEC-37 A13.3).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:563-598`
#[derive(Clone)]
pub struct TrRefdef {
    /// `x` — Raven: viewport corner, 0-at-the-top y coordinates.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:564`
    pub x: i32,
    /// `y`.
    pub y: i32,
    /// `width`.
    pub width: i32,
    /// `height`.
    pub height: i32,
    /// `fov_x`.
    pub fov_x: f32,
    /// `fov_y`.
    pub fov_y: f32,
    /// `vieworg`.
    pub view_origin: Vec3,
    /// `viewaxis[3]` — the view transformation matrix.
    pub view_axis: [Vec3; 3],
    /// `time` — Raven: time in milliseconds for shader effects and other time
    /// dependent rendering issues.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:569`
    pub time: i32,
    /// `frametime` — the delta from the previous scene's `time`, clamped to
    /// 0-500 ms by `RE_RenderScene`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:570`
    pub frametime: i32,
    /// `rdflags` — Raven: `RDF_NOWORLDMODEL`, etc.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:571`
    pub rdflags: i32,
    /// `areamask[MAX_MAP_AREA_BYTES]` — Raven: "1 bits will prevent the
    /// associated area from rendering at all". Read by `R_MarkLeaves` per
    /// leaf area, written by the scene wave from the refdef event; grown by
    /// the `tr_world` PVS-walk wave that lands `R_MarkLeaves`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:574`
    pub areamask: [u8; MAX_MAP_AREA_BYTES],
    /// `areamaskModified` — Raven: "qtrue if areamask changed since last
    /// scene". `R_MarkLeaves`'s remark trigger; a `bool` per the tier-2
    /// audit's `qboolean` -> `bool` pick.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:575`
    pub areamask_modified: bool,
    /// `floatTime` — Raven: `tr.refdef.time / 1000.0`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:577`
    pub float_time: f32,
    /// `text[MAX_RENDER_STRINGS][MAX_RENDER_STRING_LENGTH]` — Raven: text
    /// messages for deform text shaders. The oracle's fixed byte matrix
    /// becomes owned NUL-terminated Latin-1 strings, one per row.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:580`
    pub text: Vec<String>,
    /// `skyboxportal` — the oracle's sticky file-scope static, carried to
    /// write `FrameState::skyboxportal` render-side. Raven's `int`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:35`
    pub skyboxportal: i32,
    /// `drawskyboxportal` — the oracle's file-scope static, carried to write
    /// `FrameState::drawskyboxportal` render-side. Raven's `int`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:36`
    pub drawskyboxportal: i32,
}

// The refdef is zeroed by `R_Init`'s `Com_Memset(&backEnd, 0, ...)` until the
// scene wave fills it (DEC-42.1). All fields are value types with a zero
// meaning, so a whole-struct zero is the init.
impl Default for TrRefdef {
    fn default() -> TrRefdef {
        TrRefdef {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            fov_x: 0.0,
            fov_y: 0.0,
            view_origin: [0.0; 3],
            view_axis: [[0.0; 3]; 3],
            time: 0,
            frametime: 0,
            rdflags: 0,
            areamask: [0; MAX_MAP_AREA_BYTES],
            areamask_modified: false,
            float_time: 0.0,
            text: Vec::new(),
            skyboxportal: 0,
            drawskyboxportal: 0,
        }
    }
}

/// The owned form of Raven `viewParms_t` — `FrameState::view`. Its
/// `frustum[N]` count is mode-fixed (`FRUSTUM_PLANES` = 4 MP, 5 SP,
/// `R2-D7`(a)) and `isPortal`/`isMirror` become `bool`; fields land with the
/// `tr_main` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:629-644`
#[derive(Clone)]
pub struct ViewParms {
    /// `pvsOrigin` — Raven: "may be different than or.origin for portals".
    /// `R_MarkLeaves` finds the current view cluster from this point. Grown
    /// by the `tr_world` PVS-walk wave that lands `R_MarkLeaves`; the rest of
    /// `viewParms_t` lands with the `tr_main` wave.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:632`
    pub pvs_origin: Vec3,
    /// `frustum[4]` — the four view-frustum clip planes `R_RecursiveWorldNode`
    /// tests each BSP node's bounding box against (`FRUSTUM_PLANES` = 4 MP).
    /// Set up by the `tr_main` wave's `R_SetupFrustum`; read here.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:642`
    pub frustum: [cplane_t; 4],
    /// `visBounds[2]` — the accumulated bounding box of every visible leaf,
    /// grown by `R_RecursiveWorldNode` and cleared by `R_AddWorldSurfaces`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:643`
    pub vis_bounds: [Vec3; 2],
}

// Zeroed by `R_Init` until the `tr_main` wave's per-view setup fills it. The
// frustum planes zero out too (`cplane_t` has no `Default`, so the zero plane
// is written explicitly); `R_SetupFrustum` overwrites them before the walk.
impl Default for ViewParms {
    fn default() -> ViewParms {
        let zero_plane = cplane_t {
            normal: [0.0; 3],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0, 0],
        };
        ViewParms {
            pvs_origin: [0.0; 3],
            frustum: [zero_plane; 4],
            vis_bounds: [[0.0; 3]; 2],
        }
    }
}

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
    /// `surfaces` (`msurface_t *`) — the world's renderable surfaces in BSP
    /// lump order (`numsurfaces` is `surfaces.len()`). DEC-43.1: one flat
    /// index space, so `mark_surfaces` and `BModel`'s
    /// `first_surface`/`num_surfaces` range address it directly.
    pub surfaces: Vec<Surface>,
    /// `marksurfaces` (`msurface_t **`) — surface **indices** into
    /// [`Self::surfaces`], per the tier-2 transition audit's pointer-array
    /// replacement.
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

// The tables are zero until `R_InitFuncTables` fills them, matching `tr`'s
// `Com_Memset(&tr, 0, sizeof(tr))` in `R_Init` (DEC-42.1). `#[derive(Default)]`
// cannot produce this — `[T; N]: Default` stops at N = 32.
// Source: `oracle/codemp/renderer/tr_init.cpp` (`R_Init`, `R_InitFuncTables`)
impl Default for FunctionTables {
    fn default() -> FunctionTables {
        FunctionTables {
            sin_table: [0.0; FUNCTABLE_SIZE],
            square_table: [0.0; FUNCTABLE_SIZE],
            triangle_table: [0.0; FUNCTABLE_SIZE],
            saw_tooth_table: [0.0; FUNCTABLE_SIZE],
            inverse_saw_tooth_table: [0.0; FUNCTABLE_SIZE],
            fog_table: [0.0; FOG_TABLE_SIZE],
        }
    }
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
/// instead of the oracle's `skyParms_t *`. The `image_t *outerbox[6]` becomes
/// `[Option<ImageHandle>; 6]` per the tier-2 transition audit (`skyParms_t`
/// row): the interior-safety law replaces `image_t *` with a handle, and a
/// `NULL` face becomes `None`. `ParseSkyParms` fills both fields.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:449-452`
#[derive(Clone)]
pub struct SkyParms {
    /// `cloudHeight`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:450`
    pub cloud_height: f32,
    /// `outerbox[6]` — the six sky-box face images, one per suffix
    /// (`rt`/`lf`/`bk`/`ft`/`up`/`dn`). `None` marks a face `ParseSkyParms`
    /// left unset, which happens only for the `-` no-outer-box shader.
    /// `ParseSkyParms` reproduces the oracle fallback: a face whose file does
    /// not load takes the previous face's image, and face 0 takes
    /// `tr.defaultImage`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:451`
    pub outerbox: [Option<ImageHandle>; 6],
}

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
