//! `refEntity_t::renderfx` (`RF_*`) and `refdef_t::rdflags` (`RDF_*`) flag
//! masks — the crate's one canonical home for the `tr_types.h` bit constants,
//! replacing the per-file `const RDF_NOWORLDMODEL: i32 = 1;` duplicates the
//! R3 waves each landed independently.

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
