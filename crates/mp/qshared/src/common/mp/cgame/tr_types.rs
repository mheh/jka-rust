//! MP `tr_types.h` renderfx / refdef flag bits.
//!
//! The renderfx (`RF_*`) and refdef (`RDF_*`) flag constants cgame and ui set
//! on `refEntity_t.renderfx` / `refdef_t.rdflags` before handing a scene to the
//! renderer.
//!
//! Source: `oracle/codemp/cgame/tr_types.h:17-64`

use core::ffi::c_int;

// renderfx flags

/// Raven `RF_MINLIGHT` — allways have some light (viewmodel, some items).
/// Source: `oracle/codemp/cgame/tr_types.h:18`
pub const RF_MINLIGHT: c_int = 0x00001;

/// Raven `RF_THIRD_PERSON` — don't draw through eyes, only mirrors (player
/// bodies, chat sprites).
/// Source: `oracle/codemp/cgame/tr_types.h:19`
pub const RF_THIRD_PERSON: c_int = 0x00002;

/// Raven `RF_FIRST_PERSON` — only draw through eyes (view weapon, damage blood
/// blob).
/// Source: `oracle/codemp/cgame/tr_types.h:20`
pub const RF_FIRST_PERSON: c_int = 0x00004;

/// Raven `RF_DEPTHHACK` — for view weapon Z crunching.
/// Source: `oracle/codemp/cgame/tr_types.h:21`
pub const RF_DEPTHHACK: c_int = 0x00008;

/// Raven `RF_NODEPTH` — No depth at all (seeing through walls).
/// Source: `oracle/codemp/cgame/tr_types.h:22`
pub const RF_NODEPTH: c_int = 0x00010;

/// Raven `RF_VOLUMETRIC` — fake volumetric shading.
/// Source: `oracle/codemp/cgame/tr_types.h:24`
pub const RF_VOLUMETRIC: c_int = 0x00020;

/// Raven `RF_NOSHADOW` — don't add stencil shadows.
/// Source: `oracle/codemp/cgame/tr_types.h:26`
pub const RF_NOSHADOW: c_int = 0x00040;

/// Raven `RF_LIGHTING_ORIGIN` — use `refEntity->lightingOrigin` instead of
/// `refEntity->origin` for lighting.
/// This allows entities to sink into the floor with their origin going solid,
/// and allows all parts of a player to get the same lighting.
/// Source: `oracle/codemp/cgame/tr_types.h:28`
pub const RF_LIGHTING_ORIGIN: c_int = 0x00080;

/// Raven `RF_SHADOW_PLANE` — use `refEntity->shadowPlane`.
/// Source: `oracle/codemp/cgame/tr_types.h:32`
pub const RF_SHADOW_PLANE: c_int = 0x00100;

/// Raven `RF_WRAP_FRAMES` — mod the model frames by the maxframes to allow
/// continuous animation without needing to know the frame count.
/// Source: `oracle/codemp/cgame/tr_types.h:33`
pub const RF_WRAP_FRAMES: c_int = 0x00200;

/// Raven `RF_FORCE_ENT_ALPHA` — override shader alpha settings.
/// Source: `oracle/codemp/cgame/tr_types.h:36`
pub const RF_FORCE_ENT_ALPHA: c_int = 0x00400;

/// Raven `RF_RGB_TINT` — override shader rgb settings.
/// Source: `oracle/codemp/cgame/tr_types.h:37`
pub const RF_RGB_TINT: c_int = 0x00800;

/// Raven `RF_SHADOW_ONLY` — add surfs for shadowing but don't draw them.
/// Source: `oracle/codemp/cgame/tr_types.h:39`
pub const RF_SHADOW_ONLY: c_int = 0x01000;

/// Raven `RF_DISTORTION` — area distortion effect.
/// Source: `oracle/codemp/cgame/tr_types.h:41`
pub const RF_DISTORTION: c_int = 0x02000;

/// Raven `RF_FORKED` — override lightning to have forks.
/// Source: `oracle/codemp/cgame/tr_types.h:43`
pub const RF_FORKED: c_int = 0x04000;

/// Raven `RF_TAPERED` — lightning tapers.
/// Source: `oracle/codemp/cgame/tr_types.h:44`
pub const RF_TAPERED: c_int = 0x08000;

/// Raven `RF_GROW` — lightning grows from start to end during its life.
/// Source: `oracle/codemp/cgame/tr_types.h:45`
pub const RF_GROW: c_int = 0x10000;

/// Raven `RF_DISINTEGRATE1` — does a procedural hole-ripping thing.
/// Source: `oracle/codemp/cgame/tr_types.h:47`
pub const RF_DISINTEGRATE1: c_int = 0x20000;

/// Raven `RF_DISINTEGRATE2` — does a procedural hole-ripping thing with
/// scaling at the ripping point.
/// Source: `oracle/codemp/cgame/tr_types.h:48`
pub const RF_DISINTEGRATE2: c_int = 0x40000;

/// Raven `RF_SETANIMINDEX` — use `backEnd.currentEntity->e.skinNum` for
/// `R_BindAnimatedImage`.
/// Source: `oracle/codemp/cgame/tr_types.h:50`
pub const RF_SETANIMINDEX: c_int = 0x80000;

/// Raven `RF_ALPHA_DEPTH` — depth write on alpha model.
/// Source: `oracle/codemp/cgame/tr_types.h:52`
pub const RF_ALPHA_DEPTH: c_int = 0x100000;

/// Raven `RF_FORCEPOST` — force it to post-render.
/// Source: `oracle/codemp/cgame/tr_types.h:54`
pub const RF_FORCEPOST: c_int = 0x200000;

// refdef flags

/// Raven `RDF_NOWORLDMODEL` — used for player configuration screen.
/// Source: `oracle/codemp/cgame/tr_types.h:56`
pub const RDF_NOWORLDMODEL: c_int = 1;

/// Raven `RDF_HYPERSPACE` — teleportation effect.
/// Source: `oracle/codemp/cgame/tr_types.h:57`
pub const RDF_HYPERSPACE: c_int = 4;

/// Raven `RDF_SKYBOXPORTAL`.
/// Source: `oracle/codemp/cgame/tr_types.h:59`
pub const RDF_SKYBOXPORTAL: c_int = 8;

/// Raven `RDF_DRAWSKYBOX` — the above marks a scene as being a 'portal sky'.
/// this flag says to draw it or not.
/// Source: `oracle/codemp/cgame/tr_types.h:60`
pub const RDF_DRAWSKYBOX: c_int = 16;

/// Raven `RDF_AUTOMAP` — means this scene is to draw the automap.
/// Source: `oracle/codemp/cgame/tr_types.h:63`
pub const RDF_AUTOMAP: c_int = 32;

/// Raven `RDF_NOFOG` — no global fog in this scene (but still brush fog).
/// Source: `oracle/codemp/cgame/tr_types.h:64`
pub const RDF_NOFOG: c_int = 64;
