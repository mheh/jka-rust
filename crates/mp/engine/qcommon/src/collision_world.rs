//! `CollisionWorld` — the `cmg` + `SubBSP` collision state (state-ownership STATE-D2).

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::cvar::cvar_t;
use mp_qshared::shared::limits::MAX_SUB_BSP;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::surface_flags::MATERIAL_LAST;
use mp_qshared::shared::MAX_QPATH;

use crate::cm::cbrush_s::cbrush_t;
use crate::cm::ccmshader::CCMShader;
use crate::cm::clip_map_t::clipMap_t;
use crate::cm::cmodel_s::cmodel_t;
use crate::cm_load::CRMManager;
use crate::cm_terrain::CmLandScape;

/// Raven `infoParm_t` — a `svInfoParms` row: keyword name plus the
/// surface/content flag bits it ORs (and the solid-clearing mask it ANDs) into
/// a shader.
///
/// Type definition source: `oracle/codemp/qcommon/cm_shader.cpp:220-224`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InfoParm {
    pub name: *const c_char,
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

    /// Raven `int c_pointcontents` — trace optimize counter.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:221`
    pub c_pointcontents: c_int,

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
    /// curve clipping.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:223-225`
    pub cm_noAreas: *mut cvar_t,
    pub cm_noCurves: *mut cvar_t,
    pub cm_playerCurveClip: *mut cvar_t,

    /// Raven `cvar_t *com_terrainPhysics` — gates terrain-brush trace
    /// handling (`CM_TraceThroughTerrain`); named `cm_terrainPhysics` at the
    /// `CollisionWorld` call sites (`cm_trace.rs`) rather than the Raven
    /// global's own `com_` prefix.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:267`
    pub cm_terrainPhysics: *mut cvar_t,

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
    /// `mp_qshared::shared::surface_flags::MATERIALS`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:285-287`
    ///
    /// WRITELIST(cm.svMaterialNames): real string pointers, not zero — needs
    /// `Engine::new` to `.write(CollisionWorld::init_svMaterialNames())`.
    pub svMaterialNames: [*const c_char; MATERIAL_LAST as usize],

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
        use mp_qshared::shared::surface_flags::{
            CONTENTS_ABSEIL, CONTENTS_BOTCLIP, CONTENTS_DETAIL, CONTENTS_FOG, CONTENTS_INSIDE,
            CONTENTS_LADDER, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_NODROP, CONTENTS_OPAQUE,
            CONTENTS_OUTSIDE, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SOLID,
            CONTENTS_TERRAIN, CONTENTS_TRANSLUCENT, CONTENTS_TRIGGER, CONTENTS_WATER,
            SURF_NODAMAGE, SURF_NODLIGHT, SURF_NODRAW, SURF_NOIMPACT, SURF_NOMARKS, SURF_NOSTEPS,
            SURF_SKY, SURF_SLICK,
        };
        const fn p(
            name: *const c_char,
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
            p(c"sky".as_ptr(), -1, SURF_SKY, 0),
            p(c"slick".as_ptr(), -1, SURF_SLICK, 0),
            p(c"nodamage".as_ptr(), -1, SURF_NODAMAGE, 0),
            p(c"noimpact".as_ptr(), -1, SURF_NOIMPACT, 0),
            p(c"nomarks".as_ptr(), -1, SURF_NOMARKS, 0),
            p(c"nodraw".as_ptr(), -1, SURF_NODRAW, 0),
            p(c"nosteps".as_ptr(), -1, SURF_NOSTEPS, 0),
            p(c"nodlight".as_ptr(), -1, SURF_NODLIGHT, 0),
            p(c"nonsolid".as_ptr(), !CONTENTS_SOLID, 0, 0),
            p(c"nonopaque".as_ptr(), !CONTENTS_OPAQUE, 0, 0),
            p(c"lava".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_LAVA),
            p(c"water".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_WATER),
            p(c"fog".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_FOG),
            p(
                c"playerclip".as_ptr(),
                !CONTENTS_SOLID,
                0,
                CONTENTS_PLAYERCLIP,
            ),
            p(
                c"monsterclip".as_ptr(),
                !CONTENTS_SOLID,
                0,
                CONTENTS_MONSTERCLIP,
            ),
            p(c"botclip".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_BOTCLIP),
            p(c"shotclip".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_SHOTCLIP),
            p(c"trigger".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_TRIGGER),
            p(c"nodrop".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_NODROP),
            p(c"terrain".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_TERRAIN),
            p(c"ladder".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_LADDER),
            p(c"abseil".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_ABSEIL),
            p(c"outside".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_OUTSIDE),
            p(c"inside".as_ptr(), !CONTENTS_SOLID, 0, CONTENTS_INSIDE),
            p(c"detail".as_ptr(), -1, 0, CONTENTS_DETAIL),
            p(c"trans".as_ptr(), -1, 0, CONTENTS_TRANSLUCENT),
        ]
    }

    /// Raven `svMaterialNames[MATERIAL_LAST]` initializer (the `MATERIALS`
    /// X-macro expansion, as C-string pointers) — see the `svMaterialNames`
    /// field doc's writelist note.
    ///
    /// Source: `oracle/codemp/qcommon/cm_shader.cpp:285-287`; `game/surfaceflags.h:90-123`
    pub fn init_svMaterialNames() -> [*const c_char; MATERIAL_LAST as usize] {
        [
            c"none".as_ptr(),
            c"solidwood".as_ptr(),
            c"hollowwood".as_ptr(),
            c"solidmetal".as_ptr(),
            c"hollowmetal".as_ptr(),
            c"shortgrass".as_ptr(),
            c"longgrass".as_ptr(),
            c"dirt".as_ptr(),
            c"sand".as_ptr(),
            c"gravel".as_ptr(),
            c"glass".as_ptr(),
            c"concrete".as_ptr(),
            c"marble".as_ptr(),
            c"water".as_ptr(),
            c"snow".as_ptr(),
            c"ice".as_ptr(),
            c"flesh".as_ptr(),
            c"mud".as_ptr(),
            c"bpglass".as_ptr(),
            c"dryleaves".as_ptr(),
            c"greenleaves".as_ptr(),
            c"fabric".as_ptr(),
            c"canvas".as_ptr(),
            c"rock".as_ptr(),
            c"rubber".as_ptr(),
            c"plastic".as_ptr(),
            c"tiles".as_ptr(),
            c"carpet".as_ptr(),
            c"plaster".as_ptr(),
            c"shatterglass".as_ptr(),
            c"armor".as_ptr(),
            c"computer".as_ptr(),
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
