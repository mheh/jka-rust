//! `CollisionWorld` — the `cmg` + `SubBSP` collision state (state-ownership STATE-D2).

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::cvar::CvarHandle;
use mp_qshared::shared::limits::MAX_SUB_BSP;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::surface_flags::MATERIAL_LAST;
use mp_qshared::shared::surface_flags::{
    CONTENTS_ABSEIL, CONTENTS_BOTCLIP, CONTENTS_DETAIL, CONTENTS_FOG, CONTENTS_INSIDE,
    CONTENTS_LADDER, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_NODROP, CONTENTS_OPAQUE,
    CONTENTS_OUTSIDE, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SOLID, CONTENTS_TERRAIN,
    CONTENTS_TRANSLUCENT, CONTENTS_TRIGGER, CONTENTS_WATER, SURF_NODAMAGE, SURF_NODLIGHT,
    SURF_NODRAW, SURF_NOIMPACT, SURF_NOMARKS, SURF_NOSTEPS, SURF_SKY, SURF_SLICK,
};
use mp_qshared::shared::MAX_QPATH;

use crate::cm::cbrush_s::cbrush_t;
use crate::cm::ccmshader::CCMShader;
use crate::cm::clip_map_t::clipMap_t;
use crate::cm::cm_patch_h_consts::MAX_PATCH_PLANES;
use crate::cm::cmodel_s::cmodel_t;
use crate::cm::patch_plane_t::patchPlane_t;
use crate::cm_load::CRMManager;
use crate::cm_terrain::CmLandScape;

/// Raven `infoParm_t` — a `svInfoParms` row: keyword name plus the
/// surface/content flag bits it ORs (and the solid-clearing mask it ANDs) into
/// a shader.
///
/// Internal-only table row (never crosses the ABI seam — read solely by
/// `SV_ParseSurfaceParm`), so `name` is an idiomatic `&'static str` rather than
/// Raven's `const char *`.
/// Type definition source: `oracle/codemp/qcommon/cm_shader.cpp:220-224`
#[derive(Clone, Copy)]
pub struct InfoParm {
    pub name: &'static str,
    pub clearSolid: c_int,
    pub surfaceFlags: c_int,
    pub contents: c_int,
}

/// The number of `svInfoParms` rows (`oracle/codemp/qcommon/cm_shader.cpp:226-259`).
const NUM_SV_INFO_PARMS: usize = 26;

/// Faithful-shape stand-in for Raven's `CHash<T>` name-keyed hash table
/// (porting-rules §F — idiomatic reimplementation, not byte-faithful).
/// `insert`/`clear`/`count` are real; by-name lookup (`Index`) always misses
/// (returns the null/default `T`) — the same "unblock the build, semantics
/// land with the cm-shader wave" contract as `cm_get_shader_info` below
/// (that wave owns `cmShaderTable`/`shaderTextTable`'s real `Hunk_Alloc`-backed
/// table).
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:28-30`
pub struct CmHashTable<T> {
    entries: Vec<T>,
    miss: T,
}

impl<T: Default> Default for CmHashTable<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            miss: T::default(),
        }
    }
}

impl<T: Default> CmHashTable<T> {
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn insert(&mut self, item: T) {
        self.entries.push(item);
    }
}

impl<T: Default> core::ops::Index<*const c_char> for CmHashTable<T> {
    type Output = T;
    fn index(&self, _key: *const c_char) -> &T {
        &self.miss
    }
}

/// The `Engine.cm` field: `cmg`/`SubBSP[32]`/`NumSubBSP`/trace counters. An
/// instance-shaped value (STATE-D2), zero/Default-initialized by `Engine::new`
/// (mirroring Raven's static zero-init of `clipMap_t cmg`), populated in place by
/// `CM_LoadMap`. Internals are subsystem detail (non-goal), placeheld here so the
/// frozen `Engine` struct can name the field.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:37,60-61`
pub struct CollisionWorld {
    /// Raven `clipMap_t cmg` — the live clipmap.
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:37`
    pub cmg: clipMap_t,
    /// Raven `clipMap_t SubBSP[MAX_SUB_BSP]` / `int NumSubBSP, TotalSubModels`
    /// — the sub-BSP instancing table.
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:60-61`
    pub SubBSP: [clipMap_t; MAX_SUB_BSP as usize],
    pub NumSubBSP: c_int,
    pub TotalSubModels: c_int,

    /// Raven `c_pointcontents` / `c_traces` / `c_brush_traces` /
    /// `c_patch_traces` — trace optimize/statistics counters.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:220-221`
    pub c_pointcontents: c_int,
    pub c_traces: c_int,
    pub c_brush_traces: c_int,
    pub c_patch_traces: c_int,

    /// Raven `cm_polylib.cpp` winding debug counters reached through the
    /// clipmap receiver (`c_removed` free-side, `c_active_windings`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_polylib.cpp:12-15`
    pub c_removed: c_int,
    pub c_active_windings: c_int,
    pub c_peak_windings: c_int,
    pub c_winding_allocs: c_int,
    pub c_winding_points: c_int,

    /// Raven `int c_totalPatchBlocks` — running total of grid blocks across every
    /// generated patch collide (a load-time statistic accumulated by
    /// `CM_GeneratePatchCollide`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_patch.cpp:85`
    pub c_totalPatchBlocks: c_int,

    /// Raven `static int numPlanes` / `static patchPlane_t planes[MAX_PATCH_PLANES]`
    /// — the `cm_patch.cpp` plane-dedup scratch built while a patch collide is
    /// generated (`CM_FindPlane`/`CM_FindPlane2`), then copied out into the
    /// per-patch `patchCollide_s`. Threaded on the clipmap receiver (ruling 2)
    /// rather than kept as file statics; zero-valid, so the `alloc_zeroed`
    /// `Engine` mass initializes them (no boot writelist entry needed).
    ///
    /// Source: `oracle/codemp/qcommon/cm_patch.cpp:430-431`
    pub numPlanes: c_int,
    pub planes: [patchPlane_t; MAX_PATCH_PLANES],

    /// Raven `byte *cmod_base` — the current lump's read cursor into the
    /// loaded BSP file image.
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:42`
    pub cmod_base: *mut u8,

    /// Raven `cmodel_t box_model` / `cplane_t *box_planes` /
    /// `cbrush_t *box_brush` — the synthetic box/capsule trace hull, carved
    /// out of the tail of `cmg`'s plane/brush/brushside arrays.
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:50-52`
    pub box_model: cmodel_t,
    pub box_planes: *mut cplane_t,
    pub box_brush: *mut cbrush_t,

    /// Raven `cvar_t *cm_noAreas` / `*cm_noCurves` / `*cm_playerCurveClip` —
    /// cheat cvars gating area-portal culling, patch collision, and player
    /// curve clipping (cached registration handles into `Common.cvar_indexes`;
    /// `None` = Raven's not-yet-registered null).
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:223-225`
    pub cm_noAreas: Option<CvarHandle>,
    pub cm_noCurves: Option<CvarHandle>,
    pub cm_playerCurveClip: Option<CvarHandle>,

    /// Raven `void *gpvCachedMapDiskImage` / `char gsCachedMapDiskImage[MAX_QPATH]`
    /// / `qboolean gbUsingCachedMapDataRightNow` — the cached-map-diskimage
    /// lifecycle (kept across a failed `Z_Malloc` recovery attempt).
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:568-570`
    pub gpvCachedMapDiskImage: *mut (),
    pub gsCachedMapDiskImage: [c_char; MAX_QPATH],
    pub gbUsingCachedMapDataRightNow: qboolean,

    /// Raven `static unsigned last_checksum` — `CM_LoadMap`'s hoisted
    /// function-static, the last computed BSP checksum (reused when the
    /// caller re-requests the same already-loaded map).
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:610`
    pub last_checksum: c_uint,

    /// Raven `CRMManager *TheRandomMissionManager` — the RMG mission
    /// manager singleton, `delete`d on `CM_ClearMap`/map change.
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.h:60`
    pub TheRandomMissionManager: *mut CRMManager,

    /// Raven `char *shaderText` — the concatenated raw text of every loaded
    /// `.shader` file, and `CHash<CCMShaderText> shaderTextTable` — the
    /// per-label index into it.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:28-29`
    pub shaderText: *mut c_char,
    // Raven `CHash<CCMShaderText> shaderTextTable`. `CCMShaderText` is a tiny
    // vendored-container element (name + `const char *mData` into `shaderText`);
    // per porting-rules §17 it collapses to an idiomatic map of shader name →
    // byte offset of the text block within `shaderText` (`shaderText` is a raw
    // `Z_Malloc` buffer, never realloc'd after load, so offsets stay stable).
    // Source: `oracle/codemp/qcommon/cm_shader.cpp:9-33`
    //
    // WRITELIST(cm.shaderTextTable): needs `Engine::new` to
    // `.write(Default::default())`.
    pub shaderTextTable: std::collections::BTreeMap<String, usize>,

    /// Raven `CHash<CCMShader> cmShaderTable` — the by-name `CCMShader`
    /// lookup/registration table.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:30`
    ///
    /// WRITELIST(cm.cmShaderTable): `Vec`-backed, not zero-valid — needs
    /// `Engine::new` to `.write(Default::default())`.
    pub cmShaderTable: CmHashTable<*mut CCMShader>,

    /// Raven `infoParm_t svInfoParms[]` — the `surfaceparm` keyword table
    /// (`SV_ParseSurfaceParm`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:220-259`
    ///
    /// WRITELIST(cm.svInfoParms): real name-pointer/flag data, not zero —
    /// needs `Engine::new` to `.write(CollisionWorld::init_svInfoParms())`.
    pub svInfoParms: [InfoParm; NUM_SV_INFO_PARMS],

    /// Raven `const char *svMaterialNames[MATERIAL_LAST]` — the `material`
    /// keyword table (`SV_ParseMaterial`), parallel to
    /// `mp_qshared::shared::surface_flags::MATERIALS`. Internal-only (read solely
    /// by `SV_ParseMaterial`, never crosses the seam), so the entries are
    /// idiomatic `&'static str` rather than Raven's `const char *`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:285-287`
    ///
    /// WRITELIST(cm.svMaterialNames): real string data, not zero — needs
    /// `Engine::new` to `.write(CollisionWorld::init_svMaterialNames())`.
    pub svMaterialNames: [&'static str; MATERIAL_LAST as usize],

    /// Raven `cmg.landScape` — the nullable terrain landscape, `Some` only on an
    /// RMG terrain map. Set by `register_terrain` / the Wave-20 `G_RMG_INIT` arm;
    /// `None` before (Raven's `CCMLandScape *landScape` NULL pointer, faithful as
    /// `Option`, RMG-D1 / ruling 28). The `CmLandScape` struct definition lands at
    /// the wave-0–4 collision consumers (RMG-D5 struct-definition-lands-early), so
    /// this field type-checks before the wave-16 construction bodies. The
    /// `terrain_*` forwarders (Seam §C, ruling 38) — inherent `impl CollisionWorld`
    /// methods that resolve `self.land_scape` internally — are transcribed in
    /// `cm_terrain.rs` (Files roster, class `CCMLandScape`), not here.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:135`; `cm_local.h:155`
    pub land_scape: Option<CmLandScape>,
}

impl CollisionWorld {
    /// Raven `svInfoParms[]` initializer (the real data, not zero — see the
    /// `svInfoParms` field doc's writelist note).
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:226-259`
    pub fn init_svInfoParms() -> [InfoParm; NUM_SV_INFO_PARMS] {
        const fn p(
            name: &'static str,
            clear_solid: c_int,
            surface_flags: c_int,
            contents: c_int,
        ) -> InfoParm {
            InfoParm {
                name,
                clearSolid: clear_solid,
                surfaceFlags: surface_flags,
                contents,
            }
        }
        [
            p("sky", -1, SURF_SKY, 0),
            p("slick", -1, SURF_SLICK, 0),
            p("nodamage", -1, SURF_NODAMAGE, 0),
            p("noimpact", -1, SURF_NOIMPACT, 0),
            p("nomarks", -1, SURF_NOMARKS, 0),
            p("nodraw", -1, SURF_NODRAW, 0),
            p("nosteps", -1, SURF_NOSTEPS, 0),
            p("nodlight", -1, SURF_NODLIGHT, 0),
            p("nonsolid", !CONTENTS_SOLID, 0, 0),
            p("nonopaque", !CONTENTS_OPAQUE, 0, 0),
            p("lava", !CONTENTS_SOLID, 0, CONTENTS_LAVA),
            p("water", !CONTENTS_SOLID, 0, CONTENTS_WATER),
            p("fog", !CONTENTS_SOLID, 0, CONTENTS_FOG),
            p("playerclip", !CONTENTS_SOLID, 0, CONTENTS_PLAYERCLIP),
            p("monsterclip", !CONTENTS_SOLID, 0, CONTENTS_MONSTERCLIP),
            p("botclip", !CONTENTS_SOLID, 0, CONTENTS_BOTCLIP),
            p("shotclip", !CONTENTS_SOLID, 0, CONTENTS_SHOTCLIP),
            p("trigger", !CONTENTS_SOLID, 0, CONTENTS_TRIGGER),
            p("nodrop", !CONTENTS_SOLID, 0, CONTENTS_NODROP),
            p("terrain", !CONTENTS_SOLID, 0, CONTENTS_TERRAIN),
            p("ladder", !CONTENTS_SOLID, 0, CONTENTS_LADDER),
            p("abseil", !CONTENTS_SOLID, 0, CONTENTS_ABSEIL),
            p("outside", !CONTENTS_SOLID, 0, CONTENTS_OUTSIDE),
            p("inside", !CONTENTS_SOLID, 0, CONTENTS_INSIDE),
            p("detail", -1, 0, CONTENTS_DETAIL),
            p("trans", -1, 0, CONTENTS_TRANSLUCENT),
        ]
    }

    /// Raven `svMaterialNames[MATERIAL_LAST]` initializer (the `MATERIALS`
    /// X-macro expansion, as C-string pointers) — see the `svMaterialNames`
    /// field doc's writelist note.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:285-287`; `game/surfaceflags.h:90-123`
    pub fn init_svMaterialNames() -> [&'static str; MATERIAL_LAST as usize] {
        [
            "none",
            "solidwood",
            "hollowwood",
            "solidmetal",
            "hollowmetal",
            "shortgrass",
            "longgrass",
            "dirt",
            "sand",
            "gravel",
            "glass",
            "concrete",
            "marble",
            "water",
            "snow",
            "ice",
            "flesh",
            "mud",
            "bpglass",
            "dryleaves",
            "greenleaves",
            "fabric",
            "canvas",
            "rock",
            "rubber",
            "plastic",
            "tiles",
            "carpet",
            "plaster",
            "shatterglass",
            "armor",
            "computer",
        ]
    }

    /// `CCMShader *CM_GetShaderInfo( const char *name )` — decl `cm_local.h:303`,
    /// def `cm_shader.cpp:498`. `&mut self` because a miss registers a fresh
    /// `CCMShader` into `cmShaderTable` (`Hunk_Alloc` + `CM_ParseShader`,
    /// `cm_shader.cpp:512-522`), so the call mutates the clipmap;
    /// `Option<&CCMShader>` mirrors the returned pointer. Frozen extern binding
    /// per `docs/subsystems/rmg-terrain.md` (ruling 41 / RMG-D5), reached by
    /// `CmLandScape::load_terrain_def`'s `altitudetexture`/`water` cases.
    ///
    /// **Owned by the `cm` C-track qcommon packet (the wider-clipmap shader
    /// machinery — `cmShaderTable`/`Hunk_Alloc`/`CM_GetShaderText`/
    /// `CM_ParseShader`, `cm_shader.cpp:28-30,475-522`), landing with the `cm`
    /// waves.** That lane has not landed in this tree (its `CollisionWorld`
    /// fields are the sibling `//TODO: Port CollisionWorld fields` above), so
    /// this is a callable no-op that returns `None` (no shader registered) to
    /// unblock the terrain lane build; `load_terrain_def` is exercised by no
    /// runnable golden (golden #4 is a reported blocker), so the no-op is
    /// observationally inert here. Reconciled — full port + faithful table
    /// lookup — when the cm-shader wave lands.
    //TODO: Port CM_GetShaderInfo
    // Source: oracle/codemp/qcommon/cm_shader.cpp:498-524
    pub fn cm_get_shader_info(&mut self, name: &str) -> Option<&CCMShader> {
        let _ = name;
        None
    }
}
