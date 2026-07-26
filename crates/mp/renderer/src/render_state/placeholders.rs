//! Landing placeholders for the names `## Seam definition` lists as "named but
//! not defined here — they land with whichever crate owns them": the
//! CPU/frontend names at R3, the GPU-facing ones at R4.
//!
//! Each type below is an empty shape the R3/R4 wave that owns it fills in and
//! moves to its own file (house one-type-per-file convention); they exist here
//! so the root types this module freezes — `RenderAssets`, `FrameState`,
//! `FrameEvent`, `GpuResources` — compile at R3 skeleton time without any wave
//! inventing their interiors early. Every one of them is bound by the
//! interior-safety law: owned fields, handles and indices only.
//!
//! Two exceptions carry a real shape already, because the oracle payload they
//! stand for is itself pointer-free and already ported at the seam: `Vec3` and
//! `PolyVert` alias `mp_qshared`'s `vec3_t`/`polyVert_t` rather than duplicate
//! them.

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::vec3_t;

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
/// tier-1 `refEntity_t` (its `*mut c_void` ghoul2 tail is forbidden interior);
/// fields land with the `tr_scene` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:94-106`
#[derive(Clone)]
pub struct RefEntity {}

/// The owned form of Raven `trRefdef_t` — the scene description
/// `FrameEvent::RenderScene` carries and `FrameState::refdef` holds
/// render-side. Independently shaped from `refdef_t` (`R2-D6`); the oracle's
/// array pointers become owned `Vec`s on the event, `areamaskModified` a
/// `bool`, `text` a `Vec<String>`. Fields land with the `tr_scene` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:563-598`
#[derive(Clone)]
pub struct TrRefdef {}

/// The owned form of Raven `viewParms_t` — `FrameState::view`. Its
/// `frustum[N]` count is mode-fixed (`FRUSTUM_PLANES` = 4 MP, 5 SP,
/// `R2-D7`(a)) and `isPortal`/`isMirror` become `bool`; fields land with the
/// `tr_main` R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:629-644`
#[derive(Clone)]
pub struct ViewParms {}

/// The owned form of Raven `orientationr_t` — `FrameState::ori` (`R2-D7`(b):
/// Rust spells it `ori` on both modes). Fields land with the `tr_main` R3
/// wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:109-114`
#[derive(Clone)]
pub struct OrientationR {}

/// The owned form of Raven `backEndCounters_t` — `FrameState::counters`
/// (`backEnd.pc`). Fields land with the R4 backend wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1263-1275`
#[derive(Clone)]
pub struct BackEndCounters {}

/// The owned form of Raven `world_t` — `RenderAssets::world` and each entry of
/// `RenderAssets::bsp_models`, replaced wholesale on level load. Every oracle
/// pointer array becomes an owned `Vec` and the node/leaf links become arena
/// indices (tier-2 transition audit, group 1); fields land with the `tr_bsp`
/// R3 wave.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1039-1090`
#[derive(Clone)]
pub struct WorldAsset {}

/// `tr`'s precomputed wave-function tables (`sinTable`, `squareTable`,
/// `triangleTable`, `sawToothTable`, `inverseSawToothTable`, `fogTable`) —
/// `RenderAssets::function_tables`, filled once at renderer init. Fields land
/// with the `tr_init` R3 wave.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1412-1417`
#[derive(Clone)]
pub struct FunctionTables {}

/// The owned form of Raven `glconfig_t` — `RenderAssets::glconfig`,
/// sim-readable because `CG_R_GETREALRES` reads `vidWidth`/`vidHeight`
/// synchronously (B11). Not the tier-1 `glconfig_t` (its `c_char` renderer/
/// vendor/extension strings are forbidden interior — they become `String`s
/// here); fields land with the `tr_init` R3 wave.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:299-325`
#[derive(Clone)]
pub struct GlConfig {}

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
