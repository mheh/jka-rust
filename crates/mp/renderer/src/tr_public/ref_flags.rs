//! `refEntity_t::renderfx` (`RF_*`) and `refdef_t::rdflags` (`RDF_*`) flag
//! masks — the crate's one canonical home for the `tr_types.h` bit constants,
//! replacing the per-file `const RDF_NOWORLDMODEL: i32 = 1;` duplicates the
//! R3 waves each landed independently.

/// Raven `RF_MINLIGHT` — allways have some light (viewmodel, some items).
///
/// Source: `oracle/codemp/cgame/tr_types.h:18`
pub const RF_MINLIGHT: i32 = 0x00001;

/// Raven `RF_FIRST_PERSON` — only draw through eyes (view weapon, damage
/// blood blob).
///
/// Source: `oracle/codemp/cgame/tr_types.h:20`
pub const RF_FIRST_PERSON: i32 = 0x00004;

/// Raven `RF_LIGHTING_ORIGIN` — use `refEntity->lightingOrigin` instead of
/// `refEntity->origin` for lighting.
///
/// Source: `oracle/codemp/cgame/tr_types.h:28`
pub const RF_LIGHTING_ORIGIN: i32 = 0x00080;

/// Raven `RF_FORKED` — override lightning to have forks.
///
/// Source: `oracle/codemp/cgame/tr_types.h:43`
pub const RF_FORKED: i32 = 0x04000;

/// Raven `RF_TAPERED` — lightning tapers.
///
/// Source: `oracle/codemp/cgame/tr_types.h:44`
pub const RF_TAPERED: i32 = 0x08000;

/// Raven `RF_GROW` — lightning grows from start to end during its life.
///
/// Source: `oracle/codemp/cgame/tr_types.h:45`
pub const RF_GROW: i32 = 0x10000;

/// Raven `RDF_NOWORLDMODEL` — used for player configuration screen.
///
/// Source: `oracle/codemp/cgame/tr_types.h:57`
pub const RDF_NOWORLDMODEL: i32 = 1;

/// Raven `RDF_AUTOMAP` — Raven: means this scene is to draw the automap.
///
/// Source: `oracle/codemp/cgame/tr_types.h:63`
pub const RDF_AUTOMAP: i32 = 32;

/// Raven `RDF_NOFOG` — Raven: "no global fog in this scene (but still brush
/// fog)".
///
/// Source: `oracle/codemp/cgame/tr_types.h:64`
pub const RDF_NOFOG: i32 = 64;

/// Raven `RDF_SKYBOXPORTAL` — marks a scene as being a 'portal sky'.
///
/// Source: `oracle/codemp/cgame/tr_types.h:60`
pub const RDF_SKYBOXPORTAL: i32 = 8;

/// Raven `RDF_DRAWSKYBOX` — Raven: the above marks a scene as being a
/// 'portal sky'. this flag says to draw it or not.
///
/// Source: `oracle/codemp/cgame/tr_types.h:61`
pub const RDF_DRAWSKYBOX: i32 = 16;
